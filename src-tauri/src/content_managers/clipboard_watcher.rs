use super::db::DbConnection;
use super::settings::SettingsEntry;
use crate::content_managers::message_bus::AppMessage;
use crate::content_managers::message_bus::MessageBus;
use crate::error::{with_error_event, AppError, AppResult};
use anyhow::Context;
use arboard::Clipboard;
use arboard::ImageData;
use chrono::Utc;
use image::codecs::png::{PngDecoder, PngEncoder};
use image::ImageDecoder;
use image::ImageEncoder;
use regex::Regex;
use sqlx::Row;
use sqlx::SqlitePool;
use std::env::temp_dir;
use std::fs::File;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tauri::{self, async_runtime, AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClipboardEventKind {
    Text,
    Image,
}

impl ClipboardEventKind {
    fn as_str(&self) -> &str {
        match self {
            ClipboardEventKind::Text => "text",
            ClipboardEventKind::Image => "image",
        }
    }

    fn from_str(s: &str) -> AppResult<Self> {
        match s {
            "text" => Ok(ClipboardEventKind::Text),
            "image" => Ok(ClipboardEventKind::Image),
            _ => Err(AppError::DBERROR(format!(
                "Unexpected clipboard event kind in database: {}",
                s
            ))),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ClipboardEvent {
    id: String,
    entry: Vec<u8>,
    kind: ClipboardEventKind,
    timestamp: String,
}

pub struct ClipboardWatcher {
    running: bool,
    app_handle: AppHandle,
    last_text: String,
    last_image: u64,
    filters: Vec<Regex>,
    history_limit: u32,
    pool: Arc<Mutex<SqlitePool>>,
}

fn buffer_hash(buffer: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    buffer.hash(&mut hasher);
    hasher.finish()
}

fn image_to_png(image: ImageData) -> AppResult<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    PngEncoder::new(&mut buffer)
        .write_image(
            &image.bytes,
            image.width as u32,
            image.height as u32,
            image::ExtendedColorType::Rgba8,
        )
        .context("Unable to encode image to PNG")?;
    Ok(buffer)
}

impl ClipboardWatcher {
    fn notify_clipboard_updated(&self) {
        if self.app_handle.emit("clipboard_updated", ()).is_err() {
            log::error!("Unable to emit: clipboard_updated");
        }
    }

    fn is_filtered(filters: &[Regex], text: &str) -> bool {
        filters.iter().any(|regex| regex.is_match(text))
    }

    pub async fn new(
        db: Arc<DbConnection>,
        bus: MessageBus,
        app_handle: AppHandle,
        settings: SettingsEntry,
        initial_filters: Vec<Regex>,
    ) -> Arc<Mutex<Self>> {
        let mut last_text = String::from("");
        let mut last_image = 0;
        let history_limit = settings.clipboard_history_size;

        // try to assign last text to last clipboard entry
        match Clipboard::new() {
            Ok(mut clipboard) => {
                let text_value = clipboard.get_text();
                let image_value = clipboard.get_image();

                if let Ok(text) = text_value {
                    last_text = text;
                }

                if let Ok(image) = image_value {
                    last_image = buffer_hash(&image.bytes);
                }
            }
            Err(error) => {
                log::warn!("Clipboard unavailable during watcher bootstrap: {}", error);
            }
        }

        log::info!("Clipboard watcher loaded {} filters", initial_filters.len());
        log::info!("Clipboard watcher history limit set to {}", history_limit);

        let state = Arc::new(Mutex::new(Self {
            running: true,
            app_handle: app_handle.clone(),
            last_text,
            last_image,
            filters: initial_filters.clone(),
            history_limit,
            pool: Arc::clone(&db.pool),
        }));

        let refresh_state = Arc::clone(&state);
        let refresh_bus = bus.clone();
        async_runtime::spawn(async move {
            let mut receiver = refresh_bus.subscribe();

            loop {
                match receiver.recv().await {
                    Ok(AppMessage::SettingsUpdated(settings)) => {
                        let mut watcher = refresh_state.lock().await;
                        watcher.history_limit = settings.clipboard_history_size;
                        log::info!(
                            "Clipboard watcher updated history limit: {}",
                            watcher.history_limit
                        );
                    }
                    Ok(AppMessage::FiltersUpdated(updated_filters)) => {
                        let mut watcher = refresh_state.lock().await;
                        watcher.filters = updated_filters.filter_regexes;
                        log::info!(
                            "Clipboard watcher refreshed filters: {}",
                            watcher.filters.len()
                        );
                    }
                    Ok(AppMessage::SetClipboardText(text)) => {
                        let mut watcher = refresh_state.lock().await;
                        watcher.last_text.clone_from(&text);
                        drop(watcher);

                        match Clipboard::new() {
                            Ok(mut clipboard) => {
                                if let Err(err) = clipboard.set_text(text) {
                                    log::error!(
                                        "Unable to set clipboard text from message bus: {}",
                                        err
                                    );
                                }
                            }
                            Err(err) => {
                                log::error!("Unable to access clipboard from message bus: {}", err);
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        log::warn!("Clipboard watcher lagged and skipped {} messages", skipped);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        log::error!("Message bus closed for clipboard watcher");
                        break;
                    }
                }
            }
        });

        // watcher
        let cloned_state = Arc::clone(&state);
        async_runtime::spawn(async move {
            let mut clipboard = loop {
                match Clipboard::new() {
                    Ok(clipboard) => break clipboard,
                    Err(error) => {
                        log::error!("Clipboard access failed, retrying: {}", error);
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                }
            };
            log::info!("Clipboard watcher started");

            loop {
                let value = clipboard.get_text();
                // if value received
                if let Ok(text) = value {
                    let mut app_state = cloned_state.lock().await;
                    // if running and text is not same (we also discard empty text)
                    if !text.trim().is_empty() && app_state.running && text != app_state.last_text {
                        app_state.last_text.clone_from(&text);

                        if Self::is_filtered(&app_state.filters, &text) {
                            log::info!("Clipboard text skipped by active filters");
                            continue;
                        }

                        let entry = ClipboardEvent {
                            id: Uuid::new_v4().to_string(),
                            entry: text.as_bytes().to_vec(),
                            kind: ClipboardEventKind::Text,
                            timestamp: Utc::now().to_rfc3339(),
                        };
                        log::info!("Clipboard text changed: {:#?}", entry.id);
                        if app_state
                            .app_handle
                            .emit("clipboard_entry_added", entry.clone())
                            .is_err()
                        {
                            log::error!("Unable to emit: clipboard_entry_added");
                        }
                        if app_state.save(entry).await.is_err() {
                            log::error!("Unable to save: clipboard_entry_added");
                        }
                        if bus
                            .send(AppMessage::AddedToClipboard(text.clone()))
                            .is_err()
                        {
                            log::error!("Unable to send message: AddedToClipboard");
                        }
                    }
                };
                let value = clipboard.get_image();
                // if value received
                if let Ok(image) = value {
                    let mut app_state = cloned_state.lock().await;
                    let hash = buffer_hash(&image.bytes);
                    // if running and image is not same
                    if app_state.running && hash != app_state.last_image {
                        app_state.last_image = hash;

                        let entry = ClipboardEvent {
                            id: Uuid::new_v4().to_string(),
                            entry: match image_to_png(image) {
                                Ok(png) => png,
                                Err(error) => {
                                    log::error!("Unable to encode clipboard image: {}", error);
                                    continue;
                                }
                            },
                            kind: ClipboardEventKind::Image,
                            timestamp: Utc::now().to_rfc3339(),
                        };
                        log::info!("Clipboard image changed: Image hash - {:#?}", hash);
                        if app_state
                            .app_handle
                            .emit("clipboard_entry_added", entry.clone())
                            .is_err()
                        {
                            log::error!("Unable to emit: clipboard_entry_added");
                        }
                        if app_state.save(entry).await.is_err() {
                            log::error!("Unable to save: clipboard_entry_added");
                        }
                    }
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });

        state
    }

    pub fn pause(&mut self) {
        self.running = false;
        log::info!("Clipboard watcher paused");
    }

    pub fn resume(&mut self) {
        self.running = true;
        log::info!("Clipboard watcher resumed");
    }

    pub async fn read(&self, count: u32) -> AppResult<Vec<ClipboardEvent>> {
        let pool: tokio::sync::MutexGuard<'_, sqlx::Pool<sqlx::Sqlite>> = self.pool.lock().await;
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM clipboard
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(count)
        .fetch_all(&*pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            events.push(ClipboardEvent {
                id: row.get("id"),
                entry: row.get("entry"),
                kind: ClipboardEventKind::from_str(row.get("kind"))?,
                timestamp: row.get("timestamp"),
            });
        }
        log::info!("Read clipboard entries: {:#?}", events.len());
        Ok(events)
    }

    pub async fn read_one(&self, id: String) -> AppResult<ClipboardEvent> {
        log::info!("Read clipboard entry: {:#?}", id);
        let pool = self.pool.lock().await;
        let row = sqlx::query(
            r#"
            SELECT *
            FROM clipboard
            WHERE id = ?
            "#,
        )
        .bind(&id)
        .fetch_one(&*pool)
        .await?;

        Ok(ClipboardEvent {
            id: row.get("id"),
            entry: row.get("entry"),
            kind: ClipboardEventKind::from_str(row.get("kind"))?,
            timestamp: row.get("timestamp"),
        })
    }

    pub async fn delete_one(&self, id: String) -> Result<(), sqlx::Error> {
        log::info!("Deleted clipboard entry: {:#?}", id);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            DELETE FROM clipboard
            WHERE id = ?
            "#,
        )
        .bind(&id)
        .execute(&*pool)
        .await?;
        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE item_kind = 'clipboard' AND item_id = ?
            "#,
        )
        .bind(id)
        .execute(&*pool)
        .await?;
        self.notify_clipboard_updated();
        Ok(())
    }

    pub async fn delete_all(&self) -> Result<(), sqlx::Error> {
        log::info!("Deleted all clipboard entries");
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            DELETE FROM clipboard
            "#,
        )
        .execute(&*pool)
        .await?;
        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE item_kind = 'clipboard'
            "#,
        )
        .execute(&*pool)
        .await?;
        self.notify_clipboard_updated();
        log::info!("Deleted all clipboard entries");
        Ok(())
    }

    async fn save(&self, event: ClipboardEvent) -> Result<(), sqlx::Error> {
        log::info!("Saved clipboard entry: {:#?}", event.id);

        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            INSERT INTO clipboard (id, entry, kind, timestamp)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(event.id)
        .bind(event.entry)
        .bind(event.kind.as_str())
        .bind(event.timestamp)
        .execute(&*pool)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM clipboard
            WHERE id in 
                (
                SELECT id FROM clipboard 
                ORDER BY timestamp DESC 
                LIMIT -1 OFFSET ?
                )
            "#,
        )
        .bind(self.history_limit)
        .execute(&*pool)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE item_kind = 'clipboard'
              AND item_id NOT IN (SELECT id FROM clipboard)
            "#,
        )
        .execute(&*pool)
        .await?;

        Ok(())
    }
}

#[tauri::command]
pub async fn clipboard_pause_watcher(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        let mut clipboard_watcher = state.lock().await;
        clipboard_watcher.pause();
        log::info!("CMD:clipboard_pause_watcher");
        app_handle.emit("clipboard_status_changed", false)?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn clipboard_resume_watcher(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        // reset last text to prevent reading previous clipboard entry
        let mut clipboard_watcher = state.lock().await;
        let mut clipboard = Clipboard::new()?;
        if let Ok(value) = clipboard.get_text() {
            clipboard_watcher.last_text.clone_from(&value);
        } else {
            clipboard_watcher.last_text.clear();
        }
        clipboard_watcher.resume();
        log::info!("CMD:clipboard_resume_watcher");
        app_handle.emit("clipboard_status_changed", true)?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn clipboard_add_entry(
    app_handle: tauri::AppHandle,
    id: String,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:clipboard_add_entry: {:#?}", id);
        let mut clipboard_watcher = state.lock().await;
        let mut clipboard = Clipboard::new()?;
        let entry = clipboard_watcher.read_one(id).await?;
        match entry.kind {
            ClipboardEventKind::Text => {
                let text = String::from_utf8(entry.entry)?;
                clipboard_watcher.last_text.clone_from(&text);
                clipboard.set_text(text)?;
            }
            ClipboardEventKind::Image => {
                let decoder = PngDecoder::new(BufReader::new(Cursor::new(entry.entry)))
                    .context("Unable to decode PNG clipboard entry")?;
                let (width, height) = decoder.dimensions();
                let mut buffer = vec![0; (width * height * 4) as usize];
                decoder
                    .read_image(&mut buffer)
                    .context("Unable to read decoded PNG image")?;
                let hash = buffer_hash(&buffer);

                clipboard.set_image(arboard::ImageData {
                    width: width as usize,
                    height: height as usize,
                    bytes: std::borrow::Cow::from(buffer),
                })?;
                clipboard_watcher.last_image = hash;
            }
        }
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn clipboard_read_entries(
    app_handle: tauri::AppHandle,
    count: u32,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> AppResult<Vec<ClipboardEvent>> {
    with_error_event(&app_handle, async {
        log::info!("CMD:clipboard_read_entries: {}", count);
        let entries = state.lock().await.read(count).await?;
        Ok(entries)
    })
    .await
}

#[tauri::command]
pub async fn clipboard_delete_one_entry(
    app_handle: tauri::AppHandle,
    id: String,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:clipboard_delete_one_entry: {:#?}", id);
        state.lock().await.delete_one(id).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn clipboard_delete_all_entries(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:clipboard_delete_all_entries");
        state.lock().await.delete_all().await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn clipboard_open_entry(
    app_handle: tauri::AppHandle,
    id: String,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:clipboard_open_entry: {:#?}", id);
        let clipboard_watcher = state.lock().await;
        let entry = clipboard_watcher.read_one(id.clone()).await?;
        if let ClipboardEventKind::Image = entry.kind {
            let image = entry.entry;

            let mut temp_file_path = temp_dir();
            temp_file_path.push(format!("{}.png", id));

            let mut temp_file = File::create(&temp_file_path)?;
            temp_file.write_all(&image)?;
            temp_file.flush()?;

            let image_path_str = temp_file_path
                .to_str()
                .ok_or_else(|| AppError::IOERROR("Invalid temp file path".to_string()))?;

            app_handle
                .opener()
                .open_path(image_path_str, None::<&str>)
                .map_err(|error| AppError::RUNTIMEERROR(error.to_string()))?;

            log::info!("Image opened successfully");
        }
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn clipboard_read_status(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> AppResult<bool> {
    with_error_event(&app_handle, async {
        log::info!("CMD:clipboard_read_status");
        let clipboard_watcher = state.lock().await;
        Ok(clipboard_watcher.running)
    })
    .await
}
