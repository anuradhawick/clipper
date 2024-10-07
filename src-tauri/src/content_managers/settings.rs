use super::db::DbConnection;
use serde::Serialize;
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct SettingsEntry {
    color: String,
    lighting: String,
}

pub struct SettingsManager {
    pool: Arc<Mutex<SqlitePool>>,
}

impl SettingsManager {
    pub async fn new(db: Arc<DbConnection>) -> Arc<Mutex<Self>> {
        let pool = db.pool.lock().await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                color TEXT NOT NULL,
                lighting TEXT NOT NULL
            );
            "#,
        )
        .execute(&*pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO settings (id, color, lighting)
            SELECT 1, 'default', 'system'
            WHERE NOT EXISTS (SELECT 1 FROM settings WHERE id = 1);
            "#,
        )
        .execute(&*pool)
        .await
        .unwrap();
        log::info!("Settings manager initialized");
        Arc::new(Mutex::new(Self {
            pool: Arc::clone(&db.pool),
        }))
    }

    pub async fn update(&self, settings: SettingsEntry) -> Result<(), sqlx::Error> {
        log::info!("Updating settings: {:#?}", settings);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            UPDATE settings
            SET color = ?, lighting = ?
            WHERE id = 1
            "#,
        )
        .bind(settings.color)
        .bind(settings.lighting)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    pub async fn read(&self) -> Result<SettingsEntry, sqlx::Error> {
        log::info!("Reading settings");
        let pool = self.pool.lock().await;
        let result = sqlx::query(
            r#"
            SELECT *
            FROM settings
            LIMIT 1
            "#,
        )
        .fetch_one(&*pool)
        .await?;

        Ok(SettingsEntry {
            color: result.get("color"),
            lighting: result.get("lighting"),
        })
    }
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, Arc<Mutex<SettingsManager>>>,
    color: String,
    lighting: String,
) -> Result<(), String> {
    log::info!(
        "CMD:Updating settings: color: {}, lighting: {}",
        color,
        lighting
    );
    let settings = SettingsEntry { color, lighting };
    let mgr = state.lock().await;
    mgr.update(settings).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_settings(
    state: State<'_, Arc<Mutex<SettingsManager>>>,
) -> Result<SettingsEntry, String> {
    log::info!("CMD:Reading settings");
    state.lock().await.read().await.map_err(|e| e.to_string())
}
