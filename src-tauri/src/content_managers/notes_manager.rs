use super::{clipboard_watcher::ClipboardWatcher, db::DbConnection};
use arboard::Clipboard;
use chrono::Utc;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct NoteItem {
    id: String,
    entry: String,
    created_time: String,
    updated_time: String,
}

pub struct NotesManager {
    pool: Arc<Mutex<SqlitePool>>,
}

impl NotesManager {
    pub async fn new(db: Arc<DbConnection>) -> Arc<Mutex<Self>> {
        let pool = db.pool.lock().await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                entry TEXT NOT NULL,
                created_time TEXT,
                updated_time TEXT
            );
            "#,
        )
        .execute(&*pool)
        .await
        .unwrap();
        log::info!("Notes manager initialized");
        Arc::new(Mutex::new(Self {
            pool: Arc::clone(&db.pool),
        }))
    }

    pub async fn create(&self, note: NoteItem) -> Result<(), sqlx::Error> {
        log::info!("Creating note: {:#?}", note);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            INSERT INTO notes (id, entry, created_time, updated_time)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(note.id)
        .bind(note.entry)
        .bind(Utc::now().to_rfc3339())
        .bind(String::new())
        .execute(&*pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, note: NoteItem) -> Result<(), sqlx::Error> {
        log::info!("Updating note: {:#?}", note);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            UPDATE notes
            SET entry = ?, updated_time = ?
            WHERE id = ?
            "#,
        )
        .bind(note.entry)
        .bind(Utc::now().to_rfc3339())
        .bind(note.id)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        log::info!("Deleting note: {:#?}", id);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            DELETE FROM notes
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<NoteItem>, sqlx::Error> {
        log::info!("Reading notes");
        let pool = self.pool.lock().await;
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM notes
            ORDER BY created_time DESC
            "#,
        )
        .fetch_all(&*pool)
        .await?;

        let mut notes = Vec::new();
        for row in rows {
            notes.push(NoteItem {
                id: row.get("id"),
                entry: row.get("entry"),
                created_time: row.get("created_time"),
                updated_time: row.get("updated_time"),
            });
        }

        Ok(notes)
    }

    pub async fn get(&self, id: &str) -> Result<NoteItem, sqlx::Error> {
        log::info!("Getting note: {:#?}", id);
        let pool = self.pool.lock().await;
        let item = sqlx::query(
            r#"
            SELECT *
            FROM notes
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&*pool)
        .await?;

        Ok(NoteItem {
            id: item.get("id"),
            entry: item.get("entry"),
            created_time: item.get("created_time"),
            updated_time: item.get("updated_time"),
        })
    }
}

#[tauri::command]
pub async fn create_note(
    state: State<'_, Arc<Mutex<NotesManager>>>,
    id: String,
    entry: String,
) -> Result<NoteItem, String> {
    log::info!("CMD:Creating note: {:#?} {:#?}", id, entry);
    let note = NoteItem {
        id: id.clone(),
        entry,
        created_time: String::new(),
        updated_time: String::new(),
    };
    let mgr = state.lock().await;
    mgr.create(note).await.map_err(|e| e.to_string())?;
    mgr.get(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, Arc<Mutex<NotesManager>>>,
    id: String,
    entry: String,
) -> Result<NoteItem, String> {
    log::info!("CMD:Updating note: {:#?} {:#?}", id, entry);
    let note = NoteItem {
        id: id.clone(),
        entry,
        created_time: String::new(),
        updated_time: String::new(),
    };
    let mgr = state.lock().await;
    mgr.update(note).await.map_err(|e| e.to_string())?;
    mgr.get(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_note(
    state: State<'_, Arc<Mutex<NotesManager>>>,
    id: String,
) -> Result<(), String> {
    log::info!("CMD:Deleting note: {:#?}", id);
    state
        .lock()
        .await
        .delete(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_notes(
    state: State<'_, Arc<Mutex<NotesManager>>>,
) -> Result<Vec<NoteItem>, String> {
    log::info!("CMD:Reading notes");
    state.lock().await.read().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clipboard_add_note(
    id: String,
    state_clipboard_mgr: State<'_, Arc<Mutex<ClipboardWatcher>>>,
    state_notes_mgr: State<'_, Arc<Mutex<NotesManager>>>,
) -> Result<(), String> {
    log::info!("CMD:Note added to clipboard: {:#?}", id);
    let mut clipboard_watcher = state_clipboard_mgr.lock().await;
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let notes_mgr = state_notes_mgr.lock().await;
    let entry = notes_mgr.get(&id).await.map_err(|e| e.to_string())?;
    let text = entry.entry;
    clipboard_watcher.set_last_text(text.clone());
    clipboard.set_text(text).map_err(|e| e.to_string())?;

    Ok(())
}
