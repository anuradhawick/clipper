use super::db::DbConnection;
use super::message_bus::{AppMessage, MessageBus};
use crate::error::{with_error_event, AppError, AppResult};
use chrono::Utc;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct NoteItem {
    id: String,
    entry: String,
    created_time: String,
    updated_time: String,
}

pub struct NotesManager {
    app_handle: AppHandle,
    bus: MessageBus,
    pool: SqlitePool,
}

impl NotesManager {
    pub async fn new(
        db: Arc<DbConnection>,
        app_handle: AppHandle,
        bus: MessageBus,
    ) -> Arc<Mutex<Self>> {
        log::info!("Notes manager initialized");
        Arc::new(Mutex::new(Self {
            app_handle,
            bus,
            pool: db.pool.clone(),
        }))
    }

    fn notify_notes_updated(&self) {
        // Notify note lists to refetch after note mutations.
        if self.app_handle.emit("notes_updated", ()).is_err() {
            log::error!("Unable to emit: notes_updated");
        }
    }

    pub async fn create(&self, note: NoteItem) -> Result<(), sqlx::Error> {
        log::info!("Creating note: {:#?}", note);
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
        .execute(&self.pool)
        .await?;
        self.notify_notes_updated();
        Ok(())
    }

    pub async fn update(&self, note: NoteItem) -> Result<(), sqlx::Error> {
        log::info!("Updating note: {:#?}", note);
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
        .execute(&self.pool)
        .await?;
        self.notify_notes_updated();
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        log::info!("Deleting note: {:#?}", id);
        sqlx::query(
            r#"
            DELETE FROM notes
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE item_kind = 'note' AND item_id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.notify_notes_updated();
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<NoteItem>, sqlx::Error> {
        log::info!("Reading notes");
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM notes
            ORDER BY created_time DESC
            "#,
        )
        .fetch_all(&self.pool)
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
        let item = sqlx::query(
            r#"
            SELECT *
            FROM notes
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(NoteItem {
            id: item.get("id"),
            entry: item.get("entry"),
            created_time: item.get("created_time"),
            updated_time: item.get("updated_time"),
        })
    }

    pub async fn delete_all_notes(&self) -> Result<(), sqlx::Error> {
        log::info!("Deleting all notes");
        sqlx::query("DELETE FROM notes").execute(&self.pool).await?;
        sqlx::query("DELETE FROM tag_items WHERE item_kind = 'note'")
            .execute(&self.pool)
            .await?;
        self.notify_notes_updated();
        Ok(())
    }
}

#[tauri::command]
pub async fn create_note(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<NotesManager>>>,
    id: String,
    entry: String,
) -> AppResult<NoteItem> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Creating note: {:#?} {:#?}", id, entry);
        let note = NoteItem {
            id: id.clone(),
            entry,
            created_time: String::new(),
            updated_time: String::new(),
        };
        let mgr = state.lock().await;
        mgr.create(note).await?;
        mgr.get(&id).await.map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn update_note(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<NotesManager>>>,
    id: String,
    entry: String,
) -> AppResult<NoteItem> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Updating note: {:#?} {:#?}", id, entry);
        let note = NoteItem {
            id: id.clone(),
            entry,
            created_time: String::new(),
            updated_time: String::new(),
        };
        let mgr = state.lock().await;
        mgr.update(note).await?;
        mgr.get(&id).await.map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn delete_note(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<NotesManager>>>,
    id: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Deleting note: {:#?}", id);
        state.lock().await.delete(&id).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn delete_all_notes(
    app_handle: tauri::AppHandle,
    state_notes_mgr: State<'_, Arc<Mutex<NotesManager>>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Deleting all notes");
        state_notes_mgr.lock().await.delete_all_notes().await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn read_notes(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<Mutex<NotesManager>>>,
) -> AppResult<Vec<NoteItem>> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Reading notes");
        let notes = state.lock().await.read().await?;
        Ok(notes)
    })
    .await
}

#[tauri::command]
pub async fn clipboard_add_note(
    app_handle: tauri::AppHandle,
    id: String,
    state_notes_mgr: State<'_, Arc<Mutex<NotesManager>>>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Note added to clipboard: {:#?}", id);
        let (text, bus) = {
            let notes_mgr = state_notes_mgr.lock().await;
            let entry = notes_mgr.get(&id).await?;
            (entry.entry, notes_mgr.bus.clone())
        };

        bus.send(AppMessage::SetClipboardText(text))
            .map_err(|error| AppError::RuntimeError(error.to_string()))?;

        Ok(())
    })
    .await
}
