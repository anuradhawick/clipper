use super::db::DbConnection;
use crate::error::{with_error_event, AppError, AppResult};
use chrono::Utc;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct TagItem {
    id: String,
    tag: String,
    kind: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaggedItem {
    id: String,
    tag_id: String,
    item_kind: String,
    item_id: String,
    timestamp: String,
}

pub struct TagsManager {
    app_handle: AppHandle,
    pool: SqlitePool,
}

impl TagsManager {
    pub async fn new(db: Arc<DbConnection>, app_handle: AppHandle) -> Self {
        log::info!("Tags manager initialized");

        Self {
            app_handle,
            pool: db.pool.clone(),
        }
    }

    pub async fn create(&self, item: TagItem) -> Result<(), sqlx::Error> {
        log::info!("Creating tag: {:#?}", item);
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
        .execute(&self.pool)
        .await?;
        self.notify_tags_updated();
        Ok(())
    }

    pub async fn update(&self, item: TagItem) -> Result<(), sqlx::Error> {
        log::info!("Updating tag: {:#?}", item);
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
        .execute(&self.pool)
        .await?;
        self.notify_tags_updated();
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        log::info!("Deleting tag: {:#?}", id);
        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE tag_id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
            DELETE FROM tags
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.notify_tags_updated();
        self.notify_tag_items_updated();
        Ok(())
    }

    pub async fn set_item_tags(
        &self,
        item_kind: &str,
        item_id: &str,
        tag_ids: Vec<String>,
    ) -> Result<Vec<TagItem>, sqlx::Error> {
        log::info!(
            "Setting tags for tagged item: {:#?} {:#?} {:#?}",
            item_kind,
            item_id,
            tag_ids
        );
        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE item_kind = ? AND item_id = ?
            "#,
        )
        .bind(item_kind)
        .bind(item_id)
        .execute(&self.pool)
        .await?;

        for tag_id in tag_ids {
            sqlx::query(
                r#"
                INSERT INTO tag_items (id, tag_id, item_kind, item_id, timestamp)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(tag_id, item_kind, item_id) DO UPDATE SET
                    timestamp=excluded.timestamp
                "#,
            )
            .bind(Uuid::new_v4().to_string())
            .bind(tag_id)
            .bind(item_kind)
            .bind(item_id)
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;
        }

        self.notify_tag_items_updated();
        self.read_item_tags(item_kind, item_id).await
    }

    pub async fn assign_item_tag(
        &self,
        tag_id: &str,
        item_kind: &str,
        item_id: &str,
    ) -> Result<TaggedItem, sqlx::Error> {
        log::info!(
            "Assigning tag to item: {:#?} {:#?} {:#?}",
            tag_id,
            item_kind,
            item_id
        );
        let tagged_item_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO tag_items (id, tag_id, item_kind, item_id, timestamp)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(tag_id, item_kind, item_id) DO UPDATE SET
                timestamp=excluded.timestamp
            "#,
        )
        .bind(&tagged_item_id)
        .bind(tag_id)
        .bind(item_kind)
        .bind(item_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        let row = sqlx::query(
            r#"
            SELECT *
            FROM tag_items
            WHERE tag_id = ? AND item_kind = ? AND item_id = ?
            "#,
        )
        .bind(tag_id)
        .bind(item_kind)
        .bind(item_id)
        .fetch_one(&self.pool)
        .await?;

        self.notify_tag_items_updated();
        Ok(TaggedItem {
            id: row.get("id"),
            tag_id: row.get("tag_id"),
            item_kind: row.get("item_kind"),
            item_id: row.get("item_id"),
            timestamp: row.get("timestamp"),
        })
    }

    pub async fn remove_item_tag(
        &self,
        tag_id: &str,
        item_kind: &str,
        item_id: &str,
    ) -> Result<(), sqlx::Error> {
        log::info!(
            "Removing tag from item: {:#?} {:#?} {:#?}",
            tag_id,
            item_kind,
            item_id
        );
        sqlx::query(
            r#"
            DELETE FROM tag_items
            WHERE tag_id = ? AND item_kind = ? AND item_id = ?
            "#,
        )
        .bind(tag_id)
        .bind(item_kind)
        .bind(item_id)
        .execute(&self.pool)
        .await?;
        self.notify_tag_items_updated();
        Ok(())
    }

    pub async fn read_item_tags(
        &self,
        item_kind: &str,
        item_id: &str,
    ) -> Result<Vec<TagItem>, sqlx::Error> {
        log::info!("Reading item tags: {:#?} {:#?}", item_kind, item_id);
        let rows = sqlx::query(
            r#"
            SELECT tags.*
            FROM tags
            INNER JOIN tag_items ON tag_items.tag_id = tags.id
            WHERE tag_items.item_kind = ? AND tag_items.item_id = ?
            ORDER BY tags.tag ASC
            "#,
        )
        .bind(item_kind)
        .bind(item_id)
        .fetch_all(&self.pool)
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

    pub async fn read_tagged_items(&self) -> Result<Vec<TaggedItem>, sqlx::Error> {
        log::info!("Reading tagged items");
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM tag_items
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::new();
        for row in rows {
            items.push(TaggedItem {
                id: row.get("id"),
                tag_id: row.get("tag_id"),
                item_kind: row.get("item_kind"),
                item_id: row.get("item_id"),
                timestamp: row.get("timestamp"),
            });
        }

        Ok(items)
    }

    pub async fn read(&self) -> Result<Vec<TagItem>, sqlx::Error> {
        log::info!("Reading tags");
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM tags
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(&self.pool)
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
        let item = sqlx::query(
            r#"
            SELECT *
            FROM tags
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TagItem {
            id: item.get("id"),
            tag: item.get("tag"),
            kind: item.get("kind"),
            timestamp: item.get("timestamp"),
        })
    }

    fn notify_tags_updated(&self) {
        // Invalidate cached tag metadata after tags are created, edited, or deleted.
        if self.app_handle.emit("tags_updated", ()).is_err() {
            log::error!("Unable to emit: tags_updated");
        }
    }

    fn notify_tag_items_updated(&self) {
        // Invalidate per-item tag queries after tag assignments change.
        if self.app_handle.emit("tag_items_updated", ()).is_err() {
            log::error!("Unable to emit: tag_items_updated");
        }
    }
}

fn validate_tagged_item_kind(kind: &str) -> AppResult<()> {
    match kind {
        "clipboard" | "bookmark" | "note" => Ok(()),
        _ => Err(AppError::validation(format!(
            "Unsupported tagged item kind: {}",
            kind
        ))),
    }
}

#[tauri::command]
pub async fn tags_create_entry(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
    id: String,
    tag: String,
    kind: String,
) -> AppResult<TagItem> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Creating tag: {:#?} {:#?} {:#?}", id, tag, kind);
        let item = TagItem {
            id: id.clone(),
            tag,
            kind,
            timestamp: String::new(),
        };
        state.create(item).await?;
        state.get(&id).await.map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn tags_update_entry(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
    id: String,
    tag: String,
    kind: String,
) -> AppResult<TagItem> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Updating tag: {:#?} {:#?} {:#?}", id, tag, kind);
        let item = TagItem {
            id: id.clone(),
            tag,
            kind,
            timestamp: String::new(),
        };
        state.update(item).await?;
        state.get(&id).await.map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn tags_delete_one(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
    id: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Deleting tag: {:#?}", id);
        state.delete(&id).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn tags_read_entries(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
) -> AppResult<Vec<TagItem>> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Reading tags");
        let tags = state.read().await?;
        Ok(tags)
    })
    .await
}

#[tauri::command]
pub async fn tags_read_items(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
) -> AppResult<Vec<TaggedItem>> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Reading tagged items");
        let items = state.read_tagged_items().await?;
        Ok(items)
    })
    .await
}

#[tauri::command]
pub async fn tags_set_item_tags(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
    item_kind: String,
    item_id: String,
    tag_ids: Vec<String>,
) -> AppResult<Vec<TagItem>> {
    with_error_event(&app_handle, async {
        validate_tagged_item_kind(&item_kind)?;
        log::info!(
            "CMD:Setting item tags: {:#?} {:#?} {:#?}",
            item_kind,
            item_id,
            tag_ids
        );
        let tags = state.set_item_tags(&item_kind, &item_id, tag_ids).await?;
        Ok(tags)
    })
    .await
}

#[tauri::command]
pub async fn tags_assign_item(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
    tag_id: String,
    item_kind: String,
    item_id: String,
) -> AppResult<TaggedItem> {
    with_error_event(&app_handle, async {
        validate_tagged_item_kind(&item_kind)?;
        log::info!(
            "CMD:Assigning tag item: {:#?} {:#?} {:#?}",
            tag_id,
            item_kind,
            item_id
        );
        let item = state.assign_item_tag(&tag_id, &item_kind, &item_id).await?;
        Ok(item)
    })
    .await
}

#[tauri::command]
pub async fn tags_remove_item(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
    tag_id: String,
    item_kind: String,
    item_id: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        validate_tagged_item_kind(&item_kind)?;
        log::info!(
            "CMD:Removing tag item: {:#?} {:#?} {:#?}",
            tag_id,
            item_kind,
            item_id
        );
        state.remove_item_tag(&tag_id, &item_kind, &item_id).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn tags_read_item_tags(
    app_handle: tauri::AppHandle,
    state: State<'_, TagsManager>,
    item_kind: String,
    item_id: String,
) -> AppResult<Vec<TagItem>> {
    with_error_event(&app_handle, async {
        validate_tagged_item_kind(&item_kind)?;
        log::info!("CMD:Reading item tags: {:#?} {:#?}", item_kind, item_id);
        let tags = state.read_item_tags(&item_kind, &item_id).await?;
        Ok(tags)
    })
    .await
}
