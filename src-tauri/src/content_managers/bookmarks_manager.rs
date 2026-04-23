use super::db::DbConnection;
use super::settings::SettingsEntry;
use crate::content_managers::message_bus::AppMessage;
use crate::{content_managers::message_bus::MessageBus, AppHandle};
use chrono::Utc;
use regex::Regex;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Row};
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use tauri::async_runtime;
use tauri::Emitter;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct BookmarkItem {
    id: String,
    url: String,
    text: String,
    image: Vec<u8>,
    timestamp: String,
}

pub struct BookmarksManager {
    app_handle: AppHandle,
    settings: SettingsEntry,
    pool: Arc<Mutex<SqlitePool>>,
}

impl BookmarksManager {
    fn notify_bookmarks_updated(&self) {
        if self.app_handle.emit("bookmarks_updated", ()).is_err() {
            log::error!("Unable to emit: bookmarks_updated");
        }
    }

    fn extract_urls(text: &str) -> Vec<String> {
        let url_regex = Regex::new(
            r#"(?i)\b((?:https?://|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/)(?:[^\s()<>]+|\(([^\s()<>]+|(\([^\s()<>]+\)))*\))+(?:\(([^\s()<>]+|(\([^\s()<>]+\)))*\)|[^\s`!()\[\]{};:'".,<>?«»“”‘’]))"#,
        )
        .unwrap();
        url_regex
            .find_iter(text)
            .map(|mat| mat.as_str().to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    async fn fetch_meta(
        url: &str,
    ) -> Result<(String, String, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        // Normalize URL
        let mut normalized = url.trim().to_string();
        if normalized.starts_with("www.") {
            normalized = format!("https://{}", normalized);
        }
        if !normalized.starts_with("http://") && !normalized.starts_with("https://") {
            normalized = format!("https://{}", normalized);
        }

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Clipper Bookmark Metadata Fetcher)")
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        // Fetch page (awaits happen before creating non-Send DOM)
        let resp = client.get(&normalized).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Non-success status: {}", resp.status()).into());
        }
        let base_url = resp.url().clone();
        let html = resp.text().await?;

        // Scope parsing so `document` is dropped before further awaits
        let (title, description, image_url_opt) = {
            use scraper::{Html, Selector};

            let document = Html::parse_document(&html);
            let sel_title_tag = Selector::parse("title").unwrap();

            let selector_for = |attr: &str, value: &str| {
                Selector::parse(&format!(r#"meta[{}="{}"]"#, attr, value)).unwrap()
            };

            let extract_meta = |key: &str| -> Option<String> {
                for (attr, val) in [("property", key), ("name", key)] {
                    let sel = selector_for(attr, val);
                    if let Some(node) = document.select(&sel).next() {
                        if let Some(content) = node.value().attr("content") {
                            let trimmed = content.trim();
                            if !trimmed.is_empty() {
                                return Some(trimmed.to_string());
                            }
                        }
                    }
                }
                None
            };

            let title = extract_meta("og:title")
                .or_else(|| extract_meta("twitter:title"))
                .or_else(|| {
                    document
                        .select(&sel_title_tag)
                        .next()
                        .map(|n| n.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .filter(|s| !s.is_empty())
                })
                .unwrap_or_default();

            let description = extract_meta("og:description")
                .or_else(|| extract_meta("twitter:description"))
                .or_else(|| extract_meta("description"))
                .unwrap_or_default();

            let image_url = extract_meta("og:image")
                .or_else(|| extract_meta("twitter:image"))
                .or_else(|| extract_meta("twitter:image:src"));

            (title, description, image_url)
        }; // document dropped here

        // Fetch image after DOM is gone (no Send issues)
        let mut image_bytes = Vec::new();
        if let Some(raw) = image_url_opt {
            if let Ok(parsed) = reqwest::Url::parse(&raw).or_else(|_| base_url.join(&raw)) {
                if let Ok(img_resp) = client.get(parsed).send().await {
                    if img_resp.status().is_success() {
                        if let Ok(bytes) = img_resp.bytes().await {
                            image_bytes = bytes.to_vec();
                        }
                    }
                }
            }
        }

        Ok((title, description, image_bytes))
    }

    pub async fn new(
        db: Arc<DbConnection>,
        bus: MessageBus,
        app_handle: AppHandle,
        settings: SettingsEntry,
    ) -> Arc<Mutex<Self>> {
        let history_limit = settings.bookmark_history_size;

        let state = Arc::new(Mutex::new(Self {
            app_handle,
            settings,
            pool: Arc::clone(&db.pool),
        }));

        log::info!("Bookmarks manager history limit set to {}", history_limit);

        let cloned_state = Arc::clone(&state);
        async_runtime::spawn(async move {
            let mut receiver = bus.subscribe();

            loop {
                match receiver.recv().await {
                    Ok(AppMessage::AddedToClipboard(text)) => {
                        let urls = Self::extract_urls(&text);
                        log::info!("Extracted {} URLs from clipboard text", urls.len());
                        for url in urls {
                            let mut hasher = DefaultHasher::new();
                            url.hash(&mut hasher);
                            let id = hasher.finish();
                            log::info!("Extracted URL: {}", url);

                            let bookmark_extraction = Self::fetch_meta(&url).await;
                            let bookmark = match bookmark_extraction {
                                Ok((title, description, image)) => {
                                    log::info!(
                                        "Fetched metadata - Title: {}, Description: {}, Image size: {} bytes",
                                        title,
                                        description,
                                        image.len()
                                    );

                                    BookmarkItem {
                                        id: id.to_string(),
                                        url: url.clone(),
                                        text: format!("{}\\n{}", title, description),
                                        image,
                                        timestamp: Utc::now().to_rfc3339(),
                                    }
                                }
                                Err(_) => {
                                    log::warn!("Failed to fetch metadata for URL: {}", url);
                                    BookmarkItem {
                                        id: id.to_string(),
                                        url: url.clone(),
                                        text: "".to_string(),
                                        image: Vec::new(),
                                        timestamp: Utc::now().to_rfc3339(),
                                    }
                                }
                            };

                            {
                                let app_state = cloned_state.lock().await;

                                match app_state.create(bookmark.clone()).await {
                                    Ok(_) => {
                                        app_state
                                            .app_handle
                                            .emit("bookmark_entry_added", bookmark)
                                            .ok();
                                        log::info!("Bookmark saved for URL: {}", url)
                                    }
                                    Err(e) => log::error!(
                                        "Failed to save bookmark for URL {}: {}",
                                        url,
                                        e
                                    ),
                                }
                            }
                        }
                    }
                    Ok(AppMessage::SettingsUpdated(settings)) => {
                        let mut app_state = cloned_state.lock().await;
                        app_state.settings.clipboard_history_size = settings.clipboard_history_size;
                        app_state.settings.bookmark_history_size = settings.bookmark_history_size;
                        log::info!(
                            "Bookmarks manager updated history limit: {}",
                            app_state.settings.bookmark_history_size
                        );
                    }
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        log::warn!("Bookmarks listener lagged and skipped {} messages", skipped);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        log::error!("Message bus closed for bookmarks manager");
                        break;
                    }
                }
            }
        });

        log::info!("Bookmarks manager initialized");
        state
    }

    async fn create(&self, bookmark: BookmarkItem) -> Result<(), sqlx::Error> {
        log::info!("Creating bookmark: {:#?}", bookmark.url);

        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            INSERT INTO bookmarks (id, url, text, image, timestamp)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                text=excluded.text,
                image=excluded.image,
                timestamp=excluded.timestamp
            "#,
        )
        .bind(bookmark.id)
        .bind(bookmark.url)
        .bind(bookmark.text)
        .bind(bookmark.image)
        .bind(Utc::now().to_rfc3339())
        .execute(&*pool)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM bookmarks
            WHERE id NOT IN (
                SELECT id
                FROM bookmarks
                ORDER BY timestamp DESC
                LIMIT ?
            )
            "#,
        )
        .bind(self.settings.bookmark_history_size)
        .execute(&*pool)
        .await?;

        Ok(())
    }

    pub async fn update(&self, bookmark: BookmarkItem) -> Result<(), sqlx::Error> {
        log::info!("Updating bookmark: {:#?}", bookmark.url);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            UPDATE bookmarks
            SET url = ?, text = ?, image = ?, timestamp = ?
            WHERE id = ?
            "#,
        )
        .bind(bookmark.url)
        .bind(bookmark.text)
        .bind(bookmark.image)
        .bind(bookmark.timestamp)
        .bind(bookmark.id)
        .execute(&*pool)
        .await?;
        self.notify_bookmarks_updated();
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        log::info!("Deleting bookmark: {:#?}", id);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            DELETE FROM bookmarks
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&*pool)
        .await?;
        self.notify_bookmarks_updated();
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<BookmarkItem>, sqlx::Error> {
        log::info!("Reading bookmarks");
        let pool = self.pool.lock().await;
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM bookmarks
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(&*pool)
        .await?;

        let mut bookmarks = Vec::new();
        for row in rows {
            bookmarks.push(BookmarkItem {
                id: row.get("id"),
                url: row.get("url"),
                text: row.get("text"),
                image: row.get("image"),
                timestamp: row.get("timestamp"),
            });
        }

        Ok(bookmarks)
    }

    pub async fn get(&self, id: &str) -> Result<BookmarkItem, sqlx::Error> {
        log::info!("Getting bookmark: {:#?}", id);
        let pool = self.pool.lock().await;
        let item = sqlx::query(
            r#"
            SELECT *
            FROM bookmarks
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&*pool)
        .await?;

        Ok(BookmarkItem {
            id: item.get("id"),
            url: item.get("url"),
            text: item.get("text"),
            image: item.get("image"),
            timestamp: item.get("timestamp"),
        })
    }

    pub async fn delete_all_bookmarks(&self) -> Result<(), sqlx::Error> {
        log::info!("Deleting all bookmarks");
        let pool = self.pool.lock().await;
        sqlx::query("DELETE FROM bookmarks").execute(&*pool).await?;
        self.notify_bookmarks_updated();
        Ok(())
    }
}

// #[tauri::command]
// pub async fn create_bookmark(
//     state: State<'_, Arc<Mutex<BookmarksManager>>>,
//     id: String,
//     entry: String,
// ) -> Result<BookmarkItem, String> {
//     log::info!("CMD:Creating bookmark: {:#?} {:#?}", id, entry);
//     let bookmark = BookmarkItem {
//         id: id.clone(),
//         url: entry,
//         created_time: String::new(),
//         updated_time: String::new(),
//     };
//     let mgr = state.lock().await;
//     mgr.create(bookmark).await.map_err(|e| e.to_string())?;
//     mgr.get(&id).await.map_err(|e| e.to_string())
// }

#[tauri::command]
pub async fn bookmarks_update_entry(
    state: State<'_, Arc<Mutex<BookmarksManager>>>,
    id: String,
) -> Result<BookmarkItem, String> {
    log::info!("CMD:Updating bookmark: {:#?}", id);

    let existing = {
        let mgr = state.lock().await;
        mgr.get(&id).await.map_err(|e| e.to_string())?
    };

    let (title, description, image) = BookmarksManager::fetch_meta(&existing.url)
        .await
        .map_err(|e| e.to_string())?;

    let updated_bookmark = BookmarkItem {
        id: existing.id,
        url: existing.url,
        text: format!("{}\n{}", title, description),
        image,
        timestamp: Utc::now().to_rfc3339(),
    };

    let app_handle = {
        let mgr = state.lock().await;
        mgr.update(updated_bookmark.clone())
            .await
            .map_err(|e| e.to_string())?;
        mgr.app_handle.clone()
    };

    app_handle
        .emit("bookmark_entry_added", updated_bookmark.clone())
        .map_err(|e| e.to_string())?;

    Ok(updated_bookmark)
}

#[tauri::command]
pub async fn bookmarks_delete_one(
    state: State<'_, Arc<Mutex<BookmarksManager>>>,
    id: String,
) -> Result<(), String> {
    log::info!("CMD:Deleting bookmark: {:#?}", id);
    state
        .lock()
        .await
        .delete(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bookmarks_delete_all(
    state_bookmarks_mgr: State<'_, Arc<Mutex<BookmarksManager>>>,
) -> Result<(), String> {
    log::info!("CMD:Deleting all bookmarks");
    state_bookmarks_mgr
        .lock()
        .await
        .delete_all_bookmarks()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bookmarks_read_entries(
    state: State<'_, Arc<Mutex<BookmarksManager>>>,
) -> Result<Vec<BookmarkItem>, String> {
    log::info!("CMD:Reading bookmarks");
    state.lock().await.read().await.map_err(|e| e.to_string())
}
