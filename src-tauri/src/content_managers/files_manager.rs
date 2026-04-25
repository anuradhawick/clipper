use crate::error::{with_error_event, AppError, AppResult};
use futures::{future::BoxFuture, FutureExt};
use serde::Serialize;
use std::{
    io,
    path::{Path, PathBuf},
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
    pub async fn new(app_handle: AppHandle) -> AppResult<Self> {
        let app_dir = app_handle
            .path()
            .home_dir()
            .map_err(|error| AppError::IoError(format!("Home path failed: {error}")))?;
        let clipper_path = app_dir.join("clipper/");
        fs::create_dir_all(&clipper_path).await?;
        log::info!("files manager initialized");
        Ok(Self { app_handle })
    }

    pub async fn get_files(&self) -> AppResult<Vec<FileEntry>> {
        let app_dir = self
            .app_handle
            .path()
            .home_dir()
            .map_err(|error| AppError::IoError(format!("Home path failed: {error}")))?;
        let clipper_path = app_dir.join("clipper/");
        let mut files = vec![];
        let mut entries = fs::read_dir(clipper_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().ok_or_else(|| {
                AppError::IoError(format!("Invalid file entry without filename: {path:?}"))
            })?;
            let file = file_name
                .to_str()
                .ok_or_else(|| {
                    AppError::ValidationError(format!("Invalid UTF-8 filename: {file_name:?}"))
                })?
                .to_string();

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

    pub fn get_files_path(&self) -> AppResult<String> {
        let app_dir = self
            .app_handle
            .path()
            .home_dir()
            .map_err(|error| AppError::IoError(format!("Home path failed: {error}")))?;
        let clipper_path = app_dir.join("clipper/");
        Ok(clipper_path.to_string_lossy().to_string())
    }

    pub async fn delete_all_files(&self) -> AppResult<()> {
        let app_dir = self
            .app_handle
            .path()
            .home_dir()
            .map_err(|error| AppError::IoError(format!("Home path failed: {error}")))?;
        let clipper_path = app_dir.join("clipper/");
        fs::remove_dir_all(&clipper_path).await?;
        fs::create_dir_all(&clipper_path).await?;
        Ok(())
    }

    pub async fn delete_file(&self, file: String) -> AppResult<()> {
        let app_dir = self
            .app_handle
            .path()
            .home_dir()
            .map_err(|error| AppError::IoError(format!("Home path failed: {error}")))?;
        let clipper_path = app_dir.join("clipper/").join(file);

        if clipper_path.is_dir() {
            fs::remove_dir_all(&clipper_path).await?;
        } else {
            fs::remove_file(&clipper_path).await?;
        }

        Ok(())
    }

    pub async fn handle_drop(&self, paths: Vec<PathBuf>) -> AppResult<()> {
        log::info!("Dropped some files: {:#?}", paths);
        let files_path = PathBuf::from(self.get_files_path()?);

        if !files_path.exists() {
            fs::create_dir_all(&files_path).await?;
        }

        let mut added_paths: Vec<FileEntry> = vec![];

        for file in paths.into_iter() {
            let file_name_os = file.file_name().ok_or_else(|| {
                AppError::IoError(format!(
                    "Unable to read dropped file name for path: {file:?}"
                ))
            })?;
            let file_name = file_name_os.to_str().ok_or_else(|| {
                AppError::ValidationError(format!("Dropped file path is not valid UTF-8: {file:?}"))
            })?;
            let file_path = files_path.join(file_name);

            println!("Copying file: {:#?} to {:#?}", file, file_path);

            if file.is_dir() {
                copy_dir_recursive(file.clone(), file_path.clone())
                    .await
                    .map_err(AppError::from)?;
                added_paths.push(FileEntry {
                    file: file_name.to_string(),
                    clipper_path: file_path.to_string_lossy().to_string(),
                    file_type: FileType::Directory,
                });
            } else {
                fs::copy(file.clone(), file_path.clone())
                    .await
                    .map_err(AppError::from)?;
                added_paths.push(FileEntry {
                    file: file_name.to_string(),
                    clipper_path: file_path.to_string_lossy().to_string(),
                    file_type: FileType::File,
                });
            }
        }

        // Tell the frontend which dropped paths were copied into managed storage.
        self.app_handle
            .emit("files_added_paths", added_paths)
            .map_err(AppError::from)?;

        Ok(())
    }
}

#[tauri::command]
pub async fn files_get_entries(
    app_handle: tauri::AppHandle,
    files_manager: State<'_, FilesManager>,
) -> AppResult<Vec<FileEntry>> {
    with_error_event(&app_handle, async { files_manager.get_files().await }).await
}

#[tauri::command]
pub async fn files_get_storage_path(
    app_handle: tauri::AppHandle,
    files_manager: State<'_, FilesManager>,
) -> AppResult<String> {
    with_error_event(&app_handle, async { files_manager.get_files_path() }).await
}

#[tauri::command]
pub async fn files_delete_storage_path(
    app_handle: tauri::AppHandle,
    files_manager: State<'_, FilesManager>,
) -> AppResult<()> {
    with_error_event(&app_handle, async {
        files_manager.delete_all_files().await
    })
    .await
}

#[tauri::command]
pub async fn files_delete_one_file(
    app_handle: tauri::AppHandle,
    files_manager: State<'_, FilesManager>,
    file: String,
) -> AppResult<()> {
    with_error_event(&app_handle, async { files_manager.delete_file(file).await }).await
}
