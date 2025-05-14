use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::fs;
use tokio::sync::Mutex;

pub struct DbConnection {
    pub pool: Arc<Mutex<SqlitePool>>,
}

impl DbConnection {
    pub async fn new(app_handle: AppHandle) -> Self {
        // Ensure the database file is created if it doesn't exist
        let app_dir = app_handle
            .path()
            .home_dir()
            .expect("failed to get app data dir");
        let db_path = app_dir.join("clipper.db");
        let db_url = format!("sqlite://{}", db_path.to_string_lossy());
        log::info!("Clipper db_url: {:?}", &db_url);

        if !db_path.exists() {
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent).await.unwrap();
            }
            fs::File::create(db_path).await.unwrap();
        }

        let pool = SqlitePool::connect(&db_url).await.unwrap();
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
