use futures::{future::BoxFuture, FutureExt};
use serde::Serialize;
use std::{
    io,
    path::{Path, PathBuf},
    str::FromStr,
};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::fs::{self};

fn copy_dir_recursive(
    src: impl AsRef<Path> + Send + 'static,
    dst: impl AsRef<Path> + Send + 'static,
) -> BoxFuture<'static, io::Result<()>> {
    async move {
        let src = src.as_ref().to_path_buf();
        let dst = dst.as_ref().to_path_buf();

        // Create the target directory
        fs::create_dir_all(&dst).await?;

        let mut entries = fs::read_dir(src).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if path.is_dir() {
                copy_dir_recursive(path, dest_path).await?;
            } else {
                fs::copy(path, dest_path).await?;
            }
        }
        Ok(())
    }
    .boxed()
}

#[derive(Debug, Serialize, Clone)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    file: String,
    clipper_path: String,
    file_type: FileType,
}

pub struct FilesManager {
    app_handle: AppHandle,
}

impl FilesManager {
    pub async fn new(app_handle: AppHandle) -> Self {
        let app_dir = app_handle.path().home_dir().expect("Home path failed");
        let clipper_path = app_dir.join("clipper/");
        fs::create_dir_all(&clipper_path)
            .await
            .expect("Clipper path creation failed");
        log::info!("files manager initialized");
        Self { app_handle }
    }

    pub async fn get_files(&self) -> Result<Vec<FileEntry>, String> {
        let app_dir = self
            .app_handle
            .path()
            .home_dir()
            .map_err(|e| e.to_string())?;
        let clipper_path = app_dir.join("clipper/");
        let mut files = vec![];
        let mut entries = fs::read_dir(clipper_path)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let path = entry.path();
            let file = path.file_name().unwrap().to_str().unwrap().to_string();

            // Skip hidden files and directories
            if file.starts_with(".") {
                continue;
            }

            if path.is_dir() {
                files.push(FileEntry {
                    file,
                    clipper_path: path.to_string_lossy().to_string(),
                    file_type: FileType::Directory,
                });
            } else {
                files.push(FileEntry {
                    file,
                    clipper_path: path.to_string_lossy().to_string(),
                    file_type: FileType::File,
                });
            }
        }
        Ok(files)
    }

    pub fn get_files_path(&self) -> Result<String, String> {
        let app_dir = self
            .app_handle
            .path()
            .home_dir()
            .map_err(|e| e.to_string())?;
        let clipper_path = app_dir.join("clipper/");
        Ok(clipper_path.to_string_lossy().to_string())
    }

    pub async fn delete_all_files(&self) -> Result<(), String> {
        let app_dir = self
            .app_handle
            .path()
            .home_dir()
            .map_err(|e| e.to_string())?;
        let clipper_path = app_dir.join("clipper/");
        fs::remove_dir_all(&clipper_path)
            .await
            .map_err(|e| e.to_string())?;
        fs::create_dir_all(&clipper_path)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn delete_file(&self, file: String) -> Result<(), String> {
        let app_dir = self
            .app_handle
            .path()
            .home_dir()
            .map_err(|e| e.to_string())?;
        let clipper_path = app_dir.join("clipper/").join(file);

        if clipper_path.is_dir() {
            fs::remove_dir_all(&clipper_path)
                .await
                .map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(&clipper_path)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub async fn handle_drop(&self, paths: Vec<PathBuf>) -> Result<(), String> {
        log::info!("Dropped some files: {:#?}", paths);
        let files_path = PathBuf::from_str(&self.get_files_path()?).map_err(|e| e.to_string())?;

        if !files_path.exists() {
            fs::create_dir_all(&files_path)
                .await
                .map_err(|e| e.to_string())?;
        }

        let mut added_paths: Vec<FileEntry> = vec![];

        for file in paths.into_iter() {
            let file_name = file.file_name().unwrap().to_str().unwrap();
            let file_path = files_path.join(file_name);

            println!("Copying file: {:#?} to {:#?}", file, file_path);

            if file.is_dir() {
                copy_dir_recursive(file.clone(), file_path.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                added_paths.push(FileEntry {
                    file: file_name.to_string(),
                    clipper_path: file_path.to_string_lossy().to_string(),
                    file_type: FileType::Directory,
                });
            } else {
                fs::copy(file.clone(), file_path.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                added_paths.push(FileEntry {
                    file: file_name.to_string(),
                    clipper_path: file_path.to_string_lossy().to_string(),
                    file_type: FileType::File,
                });
            }
        }

        self.app_handle
            .emit("files_added", added_paths)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[tauri::command]
pub async fn get_files(files_manager: State<'_, FilesManager>) -> Result<Vec<FileEntry>, String> {
    files_manager.get_files().await
}

#[tauri::command]
pub async fn get_files_path(files_manager: State<'_, FilesManager>) -> Result<String, String> {
    files_manager.get_files_path()
}

#[tauri::command]
pub async fn delete_files_path(files_manager: State<'_, FilesManager>) -> Result<(), String> {
    files_manager.delete_all_files().await
}

#[tauri::command]
pub async fn delete_file(
    files_manager: State<'_, FilesManager>,
    file: String,
) -> Result<(), String> {
    files_manager.delete_file(file).await
}
