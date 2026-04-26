use super::db::DbConnection;
use super::global_shortcut::{register_global_shortcut, unregister_global_shortcut};
use crate::content_managers::message_bus::AppMessage;
use crate::content_managers::message_bus::MessageBus;
use crate::content_managers::message_bus::SettingsUpdatedPayload;
use crate::error::{with_error_event, AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{sqlite::SqlitePool, Row};
use std::str::FromStr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingsEntry {
    color: String,
    lighting: String,
    pub clipboard_history_size: u32,
    pub bookmark_history_size: u32,
    pub autolaunch: bool,
    pub global_shortcut: Option<String>,
}

pub struct SettingsManager {
    app_handle: AppHandle,
    pool: SqlitePool,
    bus: MessageBus,
}

impl SettingsManager {
    pub async fn new(
        db: Arc<DbConnection>,
        bus: MessageBus,
        app_handle: AppHandle,
    ) -> AppResult<Arc<Self>> {
        let pool = db.pool.clone();

        #[cfg(target_os = "linux")]
        let global_shortcut_keys =
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyC);
        #[cfg(target_os = "macos")]
        let global_shortcut_keys =
            Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyC);

        let initial_settings = sqlx::query(
            r#"
            INSERT INTO settings (id, color, lighting, clipboardHistorySize, bookmarkHistorySize, globalShortcut)
            SELECT 1, 'default', 'system', 100, 100, ?
            WHERE NOT EXISTS (SELECT 1 FROM settings WHERE id = 1);
            "#,
        )
        .bind(global_shortcut_keys.to_string())
        .execute(&pool)
        .await?;
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
            .fetch_one(&pool)
            .await?;
            let global_shortcut_keys = settings.get::<Option<String>, _>("globalShortcut");

            if let Some(global_shortcut_keys) = global_shortcut_keys {
                let global_shortcut_keys = Shortcut::from_str(global_shortcut_keys.as_str())
                    .map_err(|error| {
                        AppError::validation(format!(
                            "Invalid saved shortcut '{}': {}",
                            global_shortcut_keys, error
                        ))
                    })?;

                log::info!("Creating global shortcut: {:#?}", global_shortcut_keys);

                if let Err(e) = register_global_shortcut(&app_handle, global_shortcut_keys) {
                    log::error!("Error registering global shortcut: {}", e);
                }
            }
        }

        Ok(Arc::new(Self {
            pool: db.pool.clone(),
            app_handle,
            bus,
        }))
    }

    pub async fn update(&self, settings: SettingsEntry) -> AppResult<()> {
        log::info!("Updating settings: {:#?}", settings);
        let settings_for_event = settings.clone();
        sqlx::query(
            r#"
            UPDATE settings
            SET color = ?, lighting = ?, clipboardHistorySize = ?, bookmarkHistorySize = ?, globalShortcut = ?
            WHERE id = 1
            "#,
        )
        .bind(settings.color)
        .bind(settings.lighting)
        .bind(settings.clipboard_history_size)
        .bind(settings.bookmark_history_size)
        .bind(settings.global_shortcut.clone())
        .execute(&self.pool)
        .await?;

        if settings.autolaunch {
            self.app_handle.autolaunch().enable().map_err(|error| {
                AppError::RuntimeError(format!("Failed to enable autolaunch: {error}"))
            })?;
        } else {
            self.app_handle.autolaunch().disable().map_err(|error| {
                AppError::RuntimeError(format!("Failed to disable autolaunch: {error}"))
            })?;
        }

        match settings.global_shortcut {
            Some(global_shortcut) => {
                let global_shortcut_keys =
                    Shortcut::from_str(global_shortcut.as_str()).map_err(|error| {
                        AppError::validation(format!(
                            "Invalid shortcut '{}': {}",
                            global_shortcut, error
                        ))
                    })?;
                log::info!("Creating global shortcut: {:#?}", global_shortcut_keys);

                register_global_shortcut(&self.app_handle, global_shortcut_keys).map_err(|e| {
                    log::error!("Error registering global shortcut: {}", e);
                    e
                })?;
            }
            None => {
                log::info!("Unregistering global shortcut");
                unregister_global_shortcut(&self.app_handle).map_err(|e| {
                    log::error!("Error unregistering global shortcut: {}", e);
                    e
                })?;
            }
        }

        let payload = SettingsUpdatedPayload {
            clipboard_history_size: settings_for_event.clipboard_history_size,
            bookmark_history_size: settings_for_event.bookmark_history_size,
        };

        if self.bus.send(AppMessage::SettingsUpdated(payload)).is_err() {
            log::error!("Unable to send message: SettingsUpdated");
        }

        Ok(())
    }

    pub async fn read(&self) -> AppResult<SettingsEntry> {
        log::info!("Reading settings");
        let result = sqlx::query(
            r#"
            SELECT *
            FROM settings
            LIMIT 1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SettingsEntry {
            color: result.get("color"),
            lighting: result.get("lighting"),
            clipboard_history_size: result.get("clipboardHistorySize"),
            bookmark_history_size: result.get("bookmarkHistorySize"),
            autolaunch: self.app_handle.autolaunch().is_enabled().map_err(|error| {
                AppError::RuntimeError(format!("Unable to read autolaunch state: {error}"))
            })?,
            global_shortcut: result.get("globalShortcut"),
        })
    }
}

#[tauri::command]
pub async fn settings_update(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<SettingsManager>>,
    settings: Value,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        let settings: SettingsEntry = serde_json::from_value(settings)?;
        log::info!("CMD:Updating settings: {:#?}", settings);
        state.update(settings.clone()).await?;

        // Broadcast saved settings so every window applies theme/preference changes immediately.
        if let Err(e) = state.app_handle.emit("settings_changed", settings) {
            log::error!("Error emitting settings_changed event: {}", e);
        }

        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn settings_read(
    app_handle: tauri::AppHandle,
    state: State<'_, Arc<SettingsManager>>,
) -> AppResult<SettingsEntry> {
    with_error_event(&app_handle, async {
        let settings = state.read().await?;
        log::info!("CMD:Reading settings: {:#?}", settings);
        Ok(settings)
    })
    .await
}
