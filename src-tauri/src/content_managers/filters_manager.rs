use super::db::DbConnection;
use super::message_bus::{AppMessage, FiltersUpdatedPayload, MessageBus};
use chrono::Utc;
use regex::Regex;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

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
    pool: Arc<Mutex<SqlitePool>>,
}

impl FiltersManager {
    pub async fn new(db: Arc<DbConnection>, bus: MessageBus) -> Arc<Mutex<Self>> {
        log::info!("Filters manager initialized");

        Arc::new(Mutex::new(Self {
            bus,
            pool: Arc::clone(&db.pool),
        }))
    }

    pub async fn create(&self, filter: FilterItem) -> Result<(), sqlx::Error> {
        log::info!("Creating filter: {:#?}", filter);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            INSERT INTO filters (id, filter_regex, created_date)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(filter.id)
        .bind(filter.filter_regex)
        .bind(Utc::now().to_rfc3339())
        .execute(&*pool)
        .await?;
        drop(pool);
        self.notify_filters_updated().await;
        Ok(())
    }

    pub async fn update(&self, filter: FilterItem) -> Result<(), sqlx::Error> {
        log::info!("Updating filter: {:#?}", filter);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            UPDATE filters
            SET filter_regex = ?
            WHERE id = ?
            "#,
        )
        .bind(filter.filter_regex)
        .bind(filter.id)
        .execute(&*pool)
        .await?;
        drop(pool);
        self.notify_filters_updated().await;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        log::info!("Deleting filter: {:#?}", id);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            DELETE FROM filters
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&*pool)
        .await?;
        drop(pool);
        self.notify_filters_updated().await;
        Ok(())
    }

    pub async fn read(&self) -> Result<Vec<FilterItem>, sqlx::Error> {
        log::info!("Reading filters");
        let pool = self.pool.lock().await;
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM filters
            ORDER BY created_date DESC
            "#,
        )
        .fetch_all(&*pool)
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
        let pool = self.pool.lock().await;
        let item = sqlx::query(
            r#"
            SELECT *
            FROM filters
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&*pool)
        .await?;

        Ok(FilterItem {
            id: item.get("id"),
            filter_regex: item.get("filter_regex"),
            created_date: item.get("created_date"),
        })
    }

    pub async fn delete_all_filters(&self) -> Result<(), sqlx::Error> {
        log::info!("Deleting all filters");
        let pool = self.pool.lock().await;
        sqlx::query("DELETE FROM filters").execute(&*pool).await?;
        drop(pool);
        self.notify_filters_updated().await;
        Ok(())
    }

    async fn notify_filters_updated(&self) {
        match self.read().await {
            Ok(filters) => {
                let filter_regexes = filters
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
                    .collect();
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
    state: State<'_, Arc<Mutex<FiltersManager>>>,
    id: String,
    filter_regex: String,
) -> Result<FilterItem, String> {
    log::info!("CMD:Creating filter: {:#?} {:#?}", id, filter_regex);
    let filter = FilterItem {
        id: id.clone(),
        filter_regex,
        created_date: String::new(),
    };
    let mgr = state.lock().await;
    mgr.create(filter).await.map_err(|e| e.to_string())?;
    mgr.get(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn filters_update_entry(
    state: State<'_, Arc<Mutex<FiltersManager>>>,
    id: String,
    filter_regex: String,
) -> Result<FilterItem, String> {
    log::info!("CMD:Updating filter: {:#?} {:#?}", id, filter_regex);
    let filter = FilterItem {
        id: id.clone(),
        filter_regex,
        created_date: String::new(),
    };
    let mgr = state.lock().await;
    mgr.update(filter).await.map_err(|e| e.to_string())?;
    mgr.get(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn filters_delete_one(
    state: State<'_, Arc<Mutex<FiltersManager>>>,
    id: String,
) -> Result<(), String> {
    log::info!("CMD:Deleting filter: {:#?}", id);
    state
        .lock()
        .await
        .delete(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn filters_delete_all(
    state_filters_mgr: State<'_, Arc<Mutex<FiltersManager>>>,
) -> Result<(), String> {
    log::info!("CMD:Deleting all filters");
    state_filters_mgr
        .lock()
        .await
        .delete_all_filters()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn filters_read_entries(
    state: State<'_, Arc<Mutex<FiltersManager>>>,
) -> Result<Vec<FilterItem>, String> {
    log::info!("CMD:Reading filters");
    state.lock().await.read().await.map_err(|e| e.to_string())
}
