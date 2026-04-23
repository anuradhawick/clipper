use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::fs;
use tokio::sync::Mutex;

pub struct DbConnection {
    pub pool: Arc<Mutex<SqlitePool>>,
}

impl DbConnection {
    pub async fn new(app_handle: AppHandle) -> Self {
        let app_dir = app_handle
            .path()
            .home_dir()
            .expect("failed to get app data dir");
        let db_path = app_dir.join("clipper.db");
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).await.unwrap();
        }

        let db_url = format!("sqlite://{}", db_path.to_string_lossy());
        log::info!("Clipper db_url: {:?}", &db_url);
        let connect_options = SqliteConnectOptions::from_str(&db_url)
            .unwrap()
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .connect_with(connect_options)
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        log::info!("Clipper db connected");
        Self {
            pool: Arc::new(Mutex::new(pool)),
        }
    }
}

#[tauri::command]
pub async fn db_delete_dbfile(app_handle: tauri::AppHandle) {
    log::info!("CMD:db_delete_dbfile");
    let app_dir = app_handle
        .path()
        .home_dir()
        .expect("failed to get app data dir");
    let db_path = app_dir.join("clipper.db");
    if db_path.exists() {
        fs::remove_file(db_path).await.unwrap();
    }
    app_handle.exit(0);
}

#[tauri::command]
pub async fn db_get_dbfile_path(app_handle: tauri::AppHandle) -> String {
    log::info!("CMD:db_get_dbfile_path");
    let app_dir = app_handle
        .path()
        .home_dir()
        .expect("failed to get app data dir");
    let db_path = app_dir.join("clipper.db");
    db_path.to_string_lossy().to_string()
}
