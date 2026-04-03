use super::db::DbConnection;
use chrono::Utc;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct TagItem {
    id: String,
    tag: String,
    kind: String,
    timestamp: String,
}

pub struct TagsManager {
    app_handle: AppHandle,
    pool: Arc<Mutex<SqlitePool>>,
}

impl TagsManager {
    pub async fn new(db: Arc<DbConnection>, app_handle: AppHandle) -> Arc<Mutex<Self>> {
        let pool = db.pool.lock().await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                tag TEXT NOT NULL,
                kind TEXT NOT NULL,
                timestamp TEXT
            );
            "#,
        )
        .execute(&*pool)
        .await
        .unwrap();
        log::info!("Tags manager initialized");

        Arc::new(Mutex::new(Self {
            app_handle,
            pool: Arc::clone(&db.pool),
        }))
    }

    pub async fn create(&self, item: TagItem) -> Result<(), sqlx::Error> {
        log::info!("Creating tag: {:#?}", item);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            INSERT INTO tags (id, tag, kind, timestamp)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(item.id)
        .bind(item.tag)
        .bind(item.kind)
        .bind(Utc::now().to_rfc3339())
        .execute(&*pool)
        .await?;
        self.notify_tags_updated();
        Ok(())
    }

    pub async fn update(&self, item: TagItem) -> Result<(), sqlx::Error> {
        log::info!("Updating tag: {:#?}", item);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            UPDATE tags
            SET tag = ?, kind = ?, timestamp = ?
            WHERE id = ?
            "#,
        )
        .bind(item.tag)
        .bind(item.kind)
        .bind(Utc::now().to_rfc3339())
        .bind(item.id)
        .execute(&*pool)
        .await?;
        self.notify_tags_updated();
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        log::info!("Deleting tag: {:#?}", id);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            DELETE FROM tags
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&*pool)
        .await?;
        self.notify_tags_updated();
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<TagItem>, sqlx::Error> {
        log::info!("Reading tags");
        let pool = self.pool.lock().await;
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM tags
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(&*pool)
        .await?;

        let mut items = Vec::new();
        for row in rows {
            items.push(TagItem {
                id: row.get("id"),
                tag: row.get("tag"),
                kind: row.get("kind"),
                timestamp: row.get("timestamp"),
            });
        }

        Ok(items)
    }

    pub async fn get(&self, id: &str) -> Result<TagItem, sqlx::Error> {
        log::info!("Getting tag: {:#?}", id);
        let pool = self.pool.lock().await;
        let item = sqlx::query(
            r#"
            SELECT *
            FROM tags
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&*pool)
        .await?;

        Ok(TagItem {
            id: item.get("id"),
            tag: item.get("tag"),
            kind: item.get("kind"),
            timestamp: item.get("timestamp"),
        })
    }

    fn notify_tags_updated(&self) {
        if self.app_handle.emit("tags_updated", ()).is_err() {
            log::error!("Unable to emit: tags_updated");
        }
    }
}

#[tauri::command]
pub async fn tags_create_entry(
    state: State<'_, Arc<Mutex<TagsManager>>>,
    id: String,
    tag: String,
    kind: String,
) -> Result<TagItem, String> {
    log::info!("CMD:Creating tag: {:#?} {:#?} {:#?}", id, tag, kind);
    let item = TagItem {
        id: id.clone(),
        tag,
        kind,
        timestamp: String::new(),
    };
    let mgr = state.lock().await;
    mgr.create(item).await.map_err(|e| e.to_string())?;
    mgr.get(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tags_update_entry(
    state: State<'_, Arc<Mutex<TagsManager>>>,
    id: String,
    tag: String,
    kind: String,
) -> Result<TagItem, String> {
    log::info!("CMD:Updating tag: {:#?} {:#?} {:#?}", id, tag, kind);
    let item = TagItem {
        id: id.clone(),
        tag,
        kind,
        timestamp: String::new(),
    };
    let mgr = state.lock().await;
    mgr.update(item).await.map_err(|e| e.to_string())?;
    mgr.get(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tags_delete_one(
    state: State<'_, Arc<Mutex<TagsManager>>>,
    id: String,
) -> Result<(), String> {
    log::info!("CMD:Deleting tag: {:#?}", id);
    state
        .lock()
        .await
        .delete(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tags_read_entries(
    state: State<'_, Arc<Mutex<TagsManager>>>,
) -> Result<Vec<TagItem>, String> {
    log::info!("CMD:Reading tags");
    state.lock().await.read().await.map_err(|e| e.to_string())
}
