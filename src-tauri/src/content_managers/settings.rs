use super::db::DbConnection;
use super::global_shortcut::{register_global_shortcut, unregister_global_shortcut};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{sqlite::SqlitePool, Row};
use std::str::FromStr;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingsEntry {
    color: String,
    lighting: String,
    history_size: u32,
    autolaunch: bool,
    global_shortcut: Option<String>,
}

pub struct SettingsManager {
    app_handle: AppHandle,
    pool: Arc<Mutex<SqlitePool>>,
}

impl SettingsManager {
    pub async fn new(db: Arc<DbConnection>, app_handle: AppHandle) -> Arc<Mutex<Self>> {
        let pool = db.pool.lock().await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                color TEXT NOT NULL,
                lighting TEXT NOT NULL,
                historySize INTEGER NOT NULL,
                globalShortcut TEXT
            );
            "#,
        )
        .execute(&*pool)
        .await
        .unwrap();

        #[cfg(target_os = "linux")]
        let global_shortcut_keys =
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyC);
        #[cfg(target_os = "macos")]
        let global_shortcut_keys =
            Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyC);

        let initial_settings = sqlx::query(
            r#"
            INSERT INTO settings (id, color, lighting, historySize, globalShortcut)
            SELECT 1, 'default', 'system', 100, ?
            WHERE NOT EXISTS (SELECT 1 FROM settings WHERE id = 1);
            "#,
        )
        .bind(global_shortcut_keys.to_string())
        .execute(&*pool)
        .await
        .unwrap();
        log::info!("Settings manager initialized");

        if initial_settings.rows_affected() > 0 {
            log::info!("Settings initialized");
            log::info!("Creating global shortcut: {:#?}", global_shortcut_keys);

            if let Err(e) = register_global_shortcut(&app_handle, global_shortcut_keys) {
                log::error!("Error registering global shortcut: {}", e);
            }
        } else {
            log::info!("Settings already exist");
            let settings = sqlx::query(
                r#"
                SELECT globalShortcut
                FROM settings
                WHERE id = 1
                "#,
            )
            .fetch_one(&*pool)
            .await
            .unwrap();
            let global_shortcut_keys = settings.get::<Option<String>, _>("globalShortcut");

            if let Some(global_shortcut_keys) = global_shortcut_keys {
                let global_shortcut_keys =
                    Shortcut::from_str(global_shortcut_keys.as_str()).unwrap();

                log::info!("Creating global shortcut: {:#?}", global_shortcut_keys);

                if let Err(e) = register_global_shortcut(&app_handle, global_shortcut_keys) {
                    log::error!("Error registering global shortcut: {}", e);
                }
            }
        }

        Arc::new(Mutex::new(Self {
            pool: Arc::clone(&db.pool),
            app_handle,
        }))
    }

    pub async fn update(&self, settings: SettingsEntry) -> Result<(), String> {
        log::info!("Updating settings: {:#?}", settings);
        let pool = self.pool.lock().await;
        sqlx::query(
            r#"
            UPDATE settings
            SET color = ?, lighting = ?, historySize = ?, globalShortcut = ?
            WHERE id = 1
            "#,
        )
        .bind(settings.color)
        .bind(settings.lighting)
        .bind(settings.history_size)
        .bind(settings.global_shortcut.clone())
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

        if settings.autolaunch {
            self.app_handle
                .autolaunch()
                .enable()
                .map_err(|e| e.to_string())?;
        } else {
            self.app_handle
                .autolaunch()
                .disable()
                .map_err(|e| e.to_string())?;
        }

        match settings.global_shortcut {
            Some(global_shortcut) => {
                let global_shortcut_keys = Shortcut::from_str(global_shortcut.as_str()).unwrap();
                log::info!("Creating global shortcut: {:#?}", global_shortcut_keys);

                register_global_shortcut(&self.app_handle, global_shortcut_keys).map_err(|e| {
                    log::error!("Error registering global shortcut: {}", e);
                    e.to_string()
                })?;
            }
            None => {
                log::info!("Unregistering global shortcut");
                unregister_global_shortcut(&self.app_handle).map_err(|e| {
                    log::error!("Error unregistering global shortcut: {}", e);
                    e.to_string()
                })?;
            }
        }

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
            history_size: result.get("historySize"),
            autolaunch: self.app_handle.autolaunch().is_enabled().unwrap(),
            global_shortcut: result.get("globalShortcut"),
        })
    }
}

#[tauri::command]
pub async fn settings_update(
    state: State<'_, Arc<Mutex<SettingsManager>>>,
    settings: Value,
) -> Result<(), String> {
    let settings: SettingsEntry = serde_json::from_value(settings).map_err(|e| e.to_string())?;
    log::info!("CMD:Updating settings: {:#?}", settings);
    let mgr = state.lock().await;
    mgr.update(settings).await
}

#[tauri::command]
pub async fn settings_read(
    state: State<'_, Arc<Mutex<SettingsManager>>>,
) -> Result<SettingsEntry, String> {
    let settings = state.lock().await.read().await.map_err(|e| e.to_string())?;
    log::info!("CMD:Reading settings: {:#?}", settings);
    Ok(settings)
}
