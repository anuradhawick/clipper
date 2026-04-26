use crate::error::{with_error_event, AppError, AppResult};
use anyhow::Context;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use tauri::{AppHandle, Manager};
use tokio::fs;

pub struct DbConnection {
    pub pool: SqlitePool,
}

impl DbConnection {
    pub async fn new(app_handle: AppHandle) -> AppResult<Self> {
        let app_dir = app_handle
            .path()
            .home_dir()
            .map_err(|error| AppError::IoError(format!("failed to get home dir: {error}")))?;
        let db_path = app_dir.join("clipper.db");
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("failed to create parent directory for database")?;
        }

        let db_url = format!("sqlite://{}", db_path.to_string_lossy());
        log::info!("Clipper db_url: {:?}", &db_url);
        let connect_options = SqliteConnectOptions::from_str(&db_url)
            .context("failed to parse sqlite url")?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .connect_with(connect_options)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        log::info!("Clipper db connected");
        Ok(Self { pool })
    }
}

#[tauri::command]
pub async fn db_delete_dbfile(app_handle: tauri::AppHandle) -> AppResult<()> {
    with_error_event(&app_handle, async {
        log::info!("CMD:db_delete_dbfile");
        let app_dir = app_handle
            .path()
            .home_dir()
            .map_err(|error| AppError::IoError(format!("failed to get home dir: {error}")))?;
        let db_path = app_dir.join("clipper.db");
        if db_path.exists() {
            fs::remove_file(db_path).await?;
        }
        app_handle.exit(0);
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn db_get_dbfile_path(app_handle: tauri::AppHandle) -> AppResult<String> {
    with_error_event(&app_handle, async {
        log::info!("CMD:db_get_dbfile_path");
        let app_dir = app_handle
            .path()
            .home_dir()
            .map_err(|error| AppError::IoError(format!("failed to get home dir: {error}")))?;
        let db_path = app_dir.join("clipper.db");
        Ok(db_path.to_string_lossy().to_string())
    })
    .await
}
