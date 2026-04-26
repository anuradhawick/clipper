use super::db::DbConnection;
use super::settings::SettingsEntry;
use crate::content_managers::message_bus::{AppMessage, SettingsUpdatedPayload};
use crate::error::{with_error_event, AppError, AppResult};
use crate::{content_managers::message_bus::MessageBus, AppHandle};
use anyhow::Context;
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

struct BookmarksState {
    bookmark_history_size: u32,
}

pub struct BookmarksManager {
    app_handle: AppHandle,
    pool: SqlitePool,
    state: Mutex<BookmarksState>,
}

impl BookmarksManager {
    fn notify_bookmarks_updated(&self) {
        // Notify bookmark lists to refetch after bookmark mutations.
        if self.app_handle.emit("bookmarks_updated", ()).is_err() {
            log::error!("Unable to emit: bookmarks_updated");
        }
    }

    fn extract_urls(text: &str) -> Vec<String> {
        let url_regex = match Regex::new(
            r#"(?i)\b((?:https?://|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/)(?:[^\s()<>]+|\(([^\s()<>]+|(\([^\s()<>]+\)))*\))+(?:\(([^\s()<>]+|(\([^\s()<>]+\)))*\)|[^\s`!()\[\]{};:'".,<>?«»“”‘’]))"#,
        ) {
            Ok(regex) => regex,
            Err(error) => {
                log::error!("Failed to compile URL regex: {}", error);
                return Vec::new();
            }
        };
        url_regex
            .find_iter(text)
            .map(|mat| mat.as_str().to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    async fn fetch_meta(url: &str) -> anyhow::Result<(String, String, Vec<u8>)> {
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
            .build()
            .context("Failed to create HTTP client for metadata fetch")?;

        // Fetch page (awaits happen before creating non-Send DOM)
        let resp = client
            .get(&normalized)
            .send()
            .await
            .with_context(|| format!("Failed to fetch URL: {}", normalized))?;
        if !resp.status().is_success() {
            anyhow::bail!("Non-success status: {}", resp.status());
        }
        let base_url = resp.url().clone();
        let html = resp
            .text()
            .await
            .with_context(|| format!("Failed to read response body for URL: {}", normalized))?;

        // Scope parsing so `document` is dropped before further awaits
        let (title, description, image_url_opt) = {
            use scraper::{Html, Selector};

            let document = Html::parse_document(&html);
            let sel_title_tag = Selector::parse("title").map_err(|error| {
                anyhow::anyhow!("Failed to build selector for title tag: {error}")
            })?;

            let selector_for = |attr: &str, value: &str| {
                Selector::parse(&format!(r#"meta[{}="{}"]"#, attr, value))
                    .map_err(|error| anyhow::anyhow!("Failed to build metadata selector: {error}"))
            };

            let extract_meta = |key: &str| -> Option<String> {
                for (attr, val) in [("property", key), ("name", key)] {
                    let sel = match selector_for(attr, val) {
                        Ok(selector) => selector,
                        Err(error) => {
                            log::error!("Metadata selector parse error for {}: {}", key, error);
                            continue;
                        }
                    };
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
    ) -> Arc<Self> {
        let history_limit = settings.bookmark_history_size;

        let state = Arc::new(Self {
            app_handle,
            pool: db.pool.clone(),
            state: Mutex::new(BookmarksState {
                bookmark_history_size: history_limit,
            }),
        });

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

                            match cloned_state.create(bookmark.clone()).await {
                                Ok(_) => {
                                    // Push newly discovered bookmark metadata to open windows.
                                    cloned_state
                                        .app_handle
                                        .emit("bookmark_entry_added", bookmark)
                                        .ok();
                                    log::info!("Bookmark saved for URL: {}", url)
                                }
                                Err(e) => {
                                    log::error!("Failed to save bookmark for URL {}: {}", url, e)
                                }
                            }
                        }
                    }
                    Ok(AppMessage::SettingsUpdated(settings)) => {
                        let bookmark_history_size = settings.bookmark_history_size;
                        cloned_state.update_settings(settings).await;
                        log::info!(
                            "Bookmarks manager updated history limit: {}",
                            bookmark_history_size
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

    async fn update_settings(&self, settings: SettingsUpdatedPayload) {
        let mut state = self.state.lock().await;
        state.bookmark_history_size = settings.bookmark_history_size;
    }

    async fn bookmark_history_size(&self) -> u32 {
        self.state.lock().await.bookmark_history_size
    }

    async fn create(&self, bookmark: BookmarkItem) -> Result<(), sqlx::Error> {
        log::info!("Creating bookmark: {:#?}", bookmark.url);
        let history_limit = self.bookmark_history_size().await;

        // Insert new bookmark or update existing one with same ID (URL hash).
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
        .execute(&self.pool)
        .await?;

        // Enforce history limit by deleting oldest entries exceeding the limit.
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
        .bind(history_limit)
        .execute(&self.pool)
        .await?;

        // Clean up tag items for deleted bookmarks.
        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE item_kind = 'bookmark'
              AND item_id NOT IN (SELECT id FROM bookmarks)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update(&self, bookmark: BookmarkItem) -> Result<(), sqlx::Error> {
        log::info!("Updating bookmark: {:#?}", bookmark.url);
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
        .execute(&self.pool)
        .await?;
        self.notify_bookmarks_updated();
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        log::info!("Deleting bookmark: {:#?}", id);
        sqlx::query(
            r#"
            DELETE FROM bookmarks
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE item_kind = 'bookmark' AND item_id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.notify_bookmarks_updated();
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<BookmarkItem>, sqlx::Error> {
        log::info!("Reading bookmarks");
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM bookmarks
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(&self.pool)
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
        let item = sqlx::query(
            r#"
            SELECT *
            FROM bookmarks
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
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
        sqlx::query("DELETE FROM bookmarks")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM tag_items WHERE item_kind = 'bookmark'")
            .execute(&self.pool)
            .await?;
        self.notify_bookmarks_updated();
        Ok(())
    }
}

#[tauri::command]
pub async fn bookmarks_update_entry(
    state: State<'_, Arc<BookmarksManager>>,
    id: String,
    app_handle: tauri::AppHandle,
) -> AppResult<BookmarkItem> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Updating bookmark: {:#?}", id);
        let existing = state.get(&id).await?;

        let (title, description, image) = BookmarksManager::fetch_meta(&existing.url)
            .await
            .map_err(|error| AppError::NetworkError(error.to_string()))?;

        let updated_bookmark = BookmarkItem {
            id: existing.id,
            url: existing.url,
            text: format!("{}\n{}", title, description),
            image,
            timestamp: Utc::now().to_rfc3339(),
        };
        state.update(updated_bookmark.clone()).await?;
        let app_handle = state.app_handle.clone();

        // Push refreshed bookmark metadata to open windows.
        app_handle.emit("bookmark_entry_added", updated_bookmark.clone())?;

        Ok(updated_bookmark)
    })
    .await
}

#[tauri::command]
pub async fn bookmarks_delete_one(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<BookmarksManager>>,
    id: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Deleting bookmark: {:#?}", id);
        state.delete(&id).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn bookmarks_delete_all(
    app_handle: tauri::AppHandle,
    state_bookmarks_mgr: State<'_, Arc<BookmarksManager>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Deleting all bookmarks");
        state_bookmarks_mgr.delete_all_bookmarks().await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn bookmarks_read_entries(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<BookmarksManager>>,
) -> AppResult<Vec<BookmarkItem>> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Reading bookmarks");
        let entries = state.read().await?;
        Ok(entries)
    })
    .await
}
