use super::db::DbConnection;
use arboard::Clipboard;
use chrono::Utc;
use sqlx::Row;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tauri::{self, async_runtime, AppHandle, State};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize)]
pub enum ClipboardEventKind {
    Text,
    _Image,
}

impl ClipboardEventKind {
    fn as_str(&self) -> &str {
        match self {
            ClipboardEventKind::Text => "text",
            ClipboardEventKind::_Image => "image",
        }
    }

    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "text" => Ok(ClipboardEventKind::Text),
            "image" => Ok(ClipboardEventKind::_Image),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ClipboardEvent {
    id: String,
    entry: String,
    kind: ClipboardEventKind,
    timestamp: String,
}

pub struct ClipboardWatcher {
    running: bool,
    app_handle: AppHandle,
    last_text: String,
    pool: Arc<Mutex<SqlitePool>>,
}

impl ClipboardWatcher {
    pub async fn new(db: Arc<DbConnection>, app_handle: AppHandle) -> Arc<Mutex<Self>> {
        let mut last_text = String::from("");
        let pool = db.pool.lock().await;

        // create table if not exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS clipboard (
                id TEXT PRIMARY KEY,
                entry TEXT NOT NULL,
                kind TEXT NOT NULL,
                timestamp TEXT
            );
            "#,
        )
        .execute(&*pool)
        .await
        .expect("Unable to execute SQL!");

        // try to assign last text to last clipboard saved text
        let result = sqlx::query(
            r#"
            SELECT *
            FROM clipboard
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .fetch_one(&*pool)
        .await;

        if let Ok(row) = result {
            last_text = row.get("entry");
        }

        let state = Arc::new(Mutex::new(Self {
            running: true,
            app_handle,
            last_text,
            pool: Arc::clone(&db.pool),
        }));

        // watcher
        let cloned_state = Arc::clone(&state);
        async_runtime::spawn(async move {
            let mut clipboard = Clipboard::new().expect("Clipboard must be accessible");
            if clipboard.clear().is_err() {
                eprintln!("Unable to clear clipboard");
            };

            loop {
                {
                    let mut app_state = cloned_state.lock().await;
                    if !app_state.running {
                        continue;
                    }

                    let value = clipboard.get_text();
                    let Ok(text) = value else {
                        continue;
                    };

                    if text == app_state.last_text {
                        continue;
                    }

                    app_state.last_text.clone_from(&text);
                    let entry = ClipboardEvent {
                        id: Uuid::new_v4().to_string(),
                        entry: text,
                        kind: ClipboardEventKind::Text,
                        timestamp: Utc::now().to_rfc3339(),
                    };
                    println!("Clipboard text changed:\n{:#?}", entry);
                    if app_state
                        .app_handle
                        .emit("clipboard_entry_added", entry.clone())
                        .is_err()
                    {
                        eprintln!("Unable to emit: clipboard_entry_added");
                    }
                    if app_state.save(entry).await.is_err() {
                        eprintln!("Unable to save: clipboard_entry_added");
                    }
                }
                // when sleeping lock is released
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });

        state
    }

    pub fn pause(&mut self) {
        self.running = false;
        println!("Clipboard watcher paused");
    }

    pub fn resume(&mut self) {
        self.running = true;
        println!("Clipboard watcher resumed");
    }

    pub async fn read(&self) -> Result<Vec<ClipboardEvent>, sqlx::Error> {
        let pool = self.pool.lock().await;
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM clipboard
            ORDER BY timestamp DESC
            "#,
        )
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

        Ok(events)
    }

    pub async fn delete_one(&self, id: String) -> Result<(), sqlx::Error> {
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
        Ok(())
    }

    pub async fn delete_all(&self) -> Result<(), sqlx::Error> {
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            DELETE FROM clipboard
            "#,
        )
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn save(&self, event: ClipboardEvent) -> Result<(), sqlx::Error> {
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
        Ok(())
    }
}

#[tauri::command]
pub async fn pause_clipboard_watcher(
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    let mut clipboard_watcher = state.lock().await;
    clipboard_watcher.pause();

    Ok(())
}

#[tauri::command]
pub async fn resume_clipboard_watcher(
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    let mut clipboard_watcher = state.lock().await;
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    if let Ok(value) = clipboard.get_text() {
        clipboard_watcher.last_text = value;
    }
    clipboard_watcher.resume();

    Ok(())
}

#[tauri::command]
pub async fn clipboard_add_entry(
    entry: &str,
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    let mut clipboard_watcher = state.lock().await;
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard_watcher.last_text = String::from(entry);

    clipboard.set_text(entry).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn read_clipboard_entries(
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<Vec<ClipboardEvent>, String> {
    state.lock().await.read().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_one_clipboard_entry(
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
    id: String,
) -> Result<(), String> {
    state
        .lock()
        .await
        .delete_one(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_all_clipboard_entries(
    state: State<'_, Arc<Mutex<ClipboardWatcher>>>,
) -> Result<(), String> {
    state
        .lock()
        .await
        .delete_all()
        .await
        .map_err(|e| e.to_string())
}
