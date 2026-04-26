use super::db::DbConnection;
use super::message_bus::{AppMessage, FiltersUpdatedPayload, MessageBus};
use crate::error::{with_error_event, AppError, AppResult};
use chrono::Utc;
use regex::Regex;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Serialize, Clone)]
pub struct FilterItem {
    id: String,
    filter_regex: String,
    created_date: String,
}

impl FilterItem {
    pub fn regex(&self) -> &str {
        &self.filter_regex
    }
}

pub struct FiltersManager {
    bus: MessageBus,
    pool: SqlitePool,
}

impl FiltersManager {
    pub async fn new(db: Arc<DbConnection>, bus: MessageBus) -> Self {
        log::info!("Filters manager initialized");

        Self {
            bus,
            pool: db.pool.clone(),
        }
    }

    pub async fn create(&self, filter: FilterItem) -> Result<(), sqlx::Error> {
        log::info!("Creating filter: {:#?}", filter);
        sqlx::query(
            r#"
            INSERT INTO filters (id, filter_regex, created_date)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(filter.id)
        .bind(filter.filter_regex)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        self.notify_filters_updated().await;
        Ok(())
    }

    pub async fn update(&self, filter: FilterItem) -> Result<(), sqlx::Error> {
        log::info!("Updating filter: {:#?}", filter);
        sqlx::query(
            r#"
            UPDATE filters
            SET filter_regex = ?
            WHERE id = ?
            "#,
        )
        .bind(filter.filter_regex)
        .bind(filter.id)
        .execute(&self.pool)
        .await?;
        self.notify_filters_updated().await;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        log::info!("Deleting filter: {:#?}", id);
        sqlx::query(
            r#"
            DELETE FROM filters
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        self.notify_filters_updated().await;
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<FilterItem>, sqlx::Error> {
        log::info!("Reading filters");
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM filters
            ORDER BY created_date DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut filters = Vec::new();
        for row in rows {
            filters.push(FilterItem {
                id: row.get("id"),
                filter_regex: row.get("filter_regex"),
                created_date: row.get("created_date"),
            });
        }

        Ok(filters)
    }

    pub async fn get(&self, id: &str) -> Result<FilterItem, sqlx::Error> {
        log::info!("Getting filter: {:#?}", id);
        let item = sqlx::query(
            r#"
            SELECT *
            FROM filters
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(FilterItem {
            id: item.get("id"),
            filter_regex: item.get("filter_regex"),
            created_date: item.get("created_date"),
        })
    }

    pub async fn delete_all_filters(&self) -> Result<(), sqlx::Error> {
        log::info!("Deleting all filters");
        sqlx::query("DELETE FROM filters")
            .execute(&self.pool)
            .await?;
        self.notify_filters_updated().await;
        Ok(())
    }

    pub fn compile_filter_regexes(filters: Vec<FilterItem>) -> Vec<Regex> {
        filters
            .into_iter()
            .filter_map(|filter| match Regex::new(filter.regex()) {
                Ok(regex) => Some(regex),
                Err(err) => {
                    log::warn!(
                        "Skipping invalid filter regex '{}': {}",
                        filter.regex(),
                        err
                    );
                    None
                }
            })
            .collect()
    }

    async fn notify_filters_updated(&self) {
        match self.read().await {
            Ok(filters) => {
                let filter_regexes = Self::compile_filter_regexes(filters);
                let payload = FiltersUpdatedPayload { filter_regexes };
                if self.bus.send(AppMessage::FiltersUpdated(payload)).is_err() {
                    log::warn!("Unable to send message: FiltersUpdated");
                }
            }
            Err(err) => {
                log::error!("Unable to read filter regexes for bus update: {}", err);
            }
        }
    }
}

#[tauri::command]
pub async fn filters_create_entry(
    app_handle: tauri::AppHandle,
    state: State<'_, FiltersManager>,
    id: String,
    filter_regex: String,
) -> AppResult<FilterItem> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Creating filter: {:#?} {:#?}", id, filter_regex);
        let filter = FilterItem {
            id: id.clone(),
            filter_regex,
            created_date: String::new(),
        };
        state.create(filter).await?;
        state.get(&id).await.map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn filters_update_entry(
    app_handle: tauri::AppHandle,
    state: State<'_, FiltersManager>,
    id: String,
    filter_regex: String,
) -> AppResult<FilterItem> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Updating filter: {:#?} {:#?}", id, filter_regex);
        let filter = FilterItem {
            id: id.clone(),
            filter_regex,
            created_date: String::new(),
        };
        state.update(filter).await?;
        state.get(&id).await.map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn filters_delete_one(
    app_handle: tauri::AppHandle,
    state: State<'_, FiltersManager>,
    id: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Deleting filter: {:#?}", id);
        state.delete(&id).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn filters_delete_all(
    app_handle: tauri::AppHandle,
    state_filters_mgr: State<'_, FiltersManager>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Deleting all filters");
        state_filters_mgr.delete_all_filters().await?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn filters_read_entries(
    app_handle: tauri::AppHandle,
    state: State<'_, FiltersManager>,
) -> AppResult<Vec<FilterItem>> {
    with_error_event(&app_handle, async {
        log::info!("CMD:Reading filters");
        let filters = state.read().await?;
        Ok(filters)
    })
    .await
}
