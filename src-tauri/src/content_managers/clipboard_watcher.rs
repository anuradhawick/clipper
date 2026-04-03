use super::db::DbConnection;
use super::filters_manager::FiltersManager;
use super::settings::SettingsManager;
use crate::content_managers::message_bus::AppMessage;
use crate::content_managers::message_bus::MessageBus;
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
use tauri::{self, async_runtime, AppHandle, Manager, State};
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

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "text" => Ok(ClipboardEventKind::Text),
            "image" => Ok(ClipboardEventKind::Image),
            _ => Err(()),
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
    history_limit: u32,
    pool: Arc<Mutex<SqlitePool>>,
    filters: Vec<Regex>,
}

fn buffer_hash(buffer: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    buffer.hash(&mut hasher);
    hasher.finish()
}

fn image_to_png(image: ImageData) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    PngEncoder::new(&mut buffer)
        .write_image(
            &image.bytes,
            image.width as u32,
            image.height as u32,
            image::ExtendedColorType::Rgba8,
        )
        .expect("Unable to encode image to PNG");
    buffer
}

impl ClipboardWatcher {
    fn notify_clipboard_updated(&self) {
        if self.app_handle.emit("clipboard_updated", ()).is_err() {
            log::error!("Unable to emit: clipboard_updated");
        }
    }

    async fn load_filters(app_handle: &AppHandle) -> Vec<Regex> {
        let filters_manager = app_handle
            .state::<Arc<Mutex<FiltersManager>>>()
            .inner()
            .clone();

        let filters = {
            let manager = filters_manager.lock().await;
            match manager.read_filter_regexes().await {
                Ok(filters) => filters,
                Err(err) => {
                    log::error!("Unable to load clipboard filters: {}", err);
                    return Vec::new();
                }
            }
        };

        filters
            .into_iter()
            .filter_map(|filter| match Regex::new(&filter) {
                Ok(regex) => Some(regex),
                Err(err) => {
                    log::warn!("Skipping invalid filter regex '{}': {}", filter, err);
                    None
                }
            })
            .collect()
    }

    fn is_filtered(&self, text: &str) -> bool {
        self.filters.iter().any(|regex| regex.is_match(text))
    }

    pub async fn new(
        db: Arc<DbConnection>,
        bus: MessageBus,
        app_handle: AppHandle,
        settings_manager: Arc<Mutex<SettingsManager>>,
    ) -> Arc<Mutex<Self>> {
        let mut last_text = String::from("");
        let mut last_image = 0;
        let mut history_limit = 100;
        let pool = db.pool.lock().await;

        // create table if not exist for clipboard entries
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS clipboard (
                id TEXT PRIMARY KEY,
                entry BLOB NOT NULL,
                kind TEXT NOT NULL,
                timestamp TEXT
            );
            "#,
        )
        .execute(&*pool)
        .await
        .expect("Unable to execute SQL!");
        drop(pool);

        // try to assign last text to last clipboard entry
        let mut clipboard = Clipboard::new().expect("Clipboard must be accessible");
        let text_value = clipboard.get_text();
        let image_value = clipboard.get_image();

        if let Ok(text) = text_value {
            last_text = text;
        }

        if let Ok(image) = image_value {
            last_image = buffer_hash(&image.bytes);
        }

        match settings_manager.lock().await.read().await {
            Ok(settings) => {
                history_limit = settings.clipboard_history_size;
            }
            Err(err) => {
                log::error!(
                    "Unable to read initial settings for clipboard history limit, using default {}: {}",
                    history_limit,
                    err
                );
            }
        }

        let filters = Self::load_filters(&app_handle).await;
        log::info!("Clipboard watcher loaded {} filters", filters.len());
        log::info!("Clipboard watcher history limit set to {}", history_limit);

        let state = Arc::new(Mutex::new(Self {
            running: true,
            app_handle: app_handle.clone(),
            last_text,
            last_image,
            history_limit,
            pool: Arc::clone(&db.pool),
            filters,
        }));

        let refresh_state = Arc::clone(&state);
        let refresh_app_handle = app_handle.clone();
        let refresh_settings_manager = Arc::clone(&settings_manager);
        let refresh_bus = bus.clone();
        async_runtime::spawn(async move {
            let mut receiver = refresh_bus.subscribe();

            loop {
                match receiver.recv().await {
                    Ok(AppMessage::FiltersUpdated) => {
                        let filters = Self::load_filters(&refresh_app_handle).await;
                        let mut watcher = refresh_state.lock().await;
                        watcher.filters = filters;
                        log::info!(
                            "Clipboard watcher refreshed filters: {}",
                            watcher.filters.len()
                        );
                    }
                    Ok(AppMessage::SettingsUpdated) => {
                        match refresh_settings_manager.lock().await.read().await {
                            Ok(settings) => {
                                let mut watcher = refresh_state.lock().await;
                                watcher.history_limit = settings.clipboard_history_size;
                                log::info!(
                                    "Clipboard watcher updated history limit: {}",
                                    watcher.history_limit
                                );
                            }
                            Err(err) => {
                                log::error!(
                                    "Unable to refresh clipboard history limit from settings: {}",
                                    err
                                );
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
            let mut clipboard = Clipboard::new().expect("Clipboard must be accessible");
            log::info!("Clipboard watcher started");

            loop {
                let value = clipboard.get_text();
                // if value received
                if let Ok(text) = value {
                    let mut app_state = cloned_state.lock().await;
                    // if running and text is not same (we also discard empty text)
                    if !text.trim().is_empty() && app_state.running && text != app_state.last_text {
                        app_state.last_text.clone_from(&text);

                        if app_state.is_filtered(&text) {
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
                            entry: image_to_png(image),
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

    pub fn set_last_text(&mut self, text: String) {
        self.last_text.clone_from(&text);
    }

    pub async fn read(&self, count: u32) -> Result<Vec<ClipboardEvent>, sqlx::Error> {
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
                kind: ClipboardEventKind::from_str(row.get("kind"))
                    .expect("Unexpected ClipboardEventKind"),
                timestamp: row.get("timestamp"),
            });
        }
        log::info!("Read clipboard entries: {:#?}", events.len());
        Ok(events)
    }

    pub async fn read_one(&self, id: String) -> Result<ClipboardEvent, sqlx::Error> {
        log::info!("Read clipboard entry: {:#?}", id);
        let pool = self.pool.lock().await;
        let row = sqlx::query(
            r#"
            SELECT *
            FROM clipboard
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&*pool)
        .await?;

        Ok(ClipboardEvent {
            id: row.get("id"),
            entry: row.get("entry"),
            kind: ClipboardEventKind::from_str(row.get("kind"))
                .expect("Unexpected ClipboardEventKind"),
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

        Ok(())
    }
}

#[tauri::command]
pub async fn clipboard_pause_watcher(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    let mut clipboard_watcher = state.lock().await;
    clipboard_watcher.pause();
    log::info!("CMD:clipboard_pause_watcher");
    app_handle
        .emit("clipboard_status_changed", false)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clipboard_resume_watcher(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    // reset last text to prevent reading previous clipboard entry
    let mut clipboard_watcher = state.lock().await;
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    if let Ok(value) = clipboard.get_text() {
        clipboard_watcher.last_text.clone_from(&value);
    } else {
        clipboard_watcher.last_text.clear();
    }
    clipboard_watcher.resume();
    log::info!("CMD:clipboard_resume_watcher");
    app_handle
        .emit("clipboard_status_changed", true)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clipboard_add_entry(
    id: String,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    log::info!("CMD:clipboard_add_entry: {:#?}", id);
    let mut clipboard_watcher = state.lock().await;
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let entry = clipboard_watcher
        .read_one(id)
        .await
        .map_err(|e| e.to_string())?;
    match entry.kind {
        ClipboardEventKind::Text => {
            let text = String::from_utf8(entry.entry).map_err(|e| e.to_string())?;
            clipboard_watcher.last_text.clone_from(&text);
            clipboard.set_text(text).map_err(|e| e.to_string())?;
        }
        ClipboardEventKind::Image => {
            let decoder = PngDecoder::new(BufReader::new(Cursor::new(entry.entry))).unwrap();
            let (width, height) = decoder.dimensions();
            let mut buffer = vec![0; (width * height * 4) as usize]; // Assuming RGBA8 format
            decoder.read_image(&mut buffer).unwrap();
            let hash = buffer_hash(&buffer);

            clipboard
                .set_image(arboard::ImageData {
                    width: width as usize,
                    height: height as usize,
                    bytes: std::borrow::Cow::from(buffer),
                })
                .map_err(|e| e.to_string())?;
            clipboard_watcher.last_image = hash;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn clipboard_read_entries(
    count: u32,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<Vec<ClipboardEvent>, String> {
    log::info!("CMD:clipboard_read_entries: {}", count);
    state
        .lock()
        .await
        .read(count)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clipboard_delete_one_entry(
    id: String,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    log::info!("CMD:clipboard_delete_one_entry: {:#?}", id);
    state
        .lock()
        .await
        .delete_one(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clipboard_delete_all_entries(
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    log::info!("CMD:clipboard_delete_all_entries");
    state
        .lock()
        .await
        .delete_all()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clipboard_open_entry(
    id: String,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    log::info!("CMD:clipboard_open_entry: {:#?}", id);
    let clipboard_watcher = state.lock().await;
    let entry = clipboard_watcher
        .read_one(id.clone())
        .await
        .map_err(|e| e.to_string())?;
    if let ClipboardEventKind::Image = entry.kind {
        let image = entry.entry;

        let mut temp_file_path = temp_dir();
        temp_file_path.push(format!("{}.png", id));

        let mut temp_file = File::create(&temp_file_path).map_err(|e| e.to_string())?;
        temp_file.write_all(&image).map_err(|e| e.to_string())?;
        temp_file.flush().map_err(|e| e.to_string())?;

        let image_path_str = temp_file_path.to_str().ok_or("Invalid path".to_string())?;

        if let Err(e) = app_handle
            .opener()
            .open_path(image_path_str, None::<&str>)
            .map_err(|e| e.to_string())
        {
            log::error!("Failed to open image: {}", e);
            return Err(e.to_string());
        }

        log::info!("Image opened successfully");
    }
    Ok(())
}

#[tauri::command]
pub async fn clipboard_read_status(
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<bool, String> {
    log::info!("CMD:clipboard_read_status");
    let clipboard_watcher = state.lock().await;
    Ok(clipboard_watcher.running)
}
