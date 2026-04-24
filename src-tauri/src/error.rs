use serde::Serialize;
use std::future::Future;
use tauri::{AppHandle, Emitter};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error, Serialize, Clone)]
#[allow(clippy::upper_case_acronyms)]
#[serde(tag = "type", content = "description")]
pub enum AppError {
    #[error("{0}")]
    DBERROR(String),
    #[error("{0}")]
    IOERROR(String),
    #[error("{0}")]
    VALIDATIONERROR(String),
    #[error("{0}")]
    NETWORKERROR(String),
    #[error("{0}")]
    RUNTIMEERROR(String),
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackendErrorPayload {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::DBERROR(_) => "DBERROR",
            AppError::IOERROR(_) => "IOERROR",
            AppError::VALIDATIONERROR(_) => "VALIDATIONERROR",
            AppError::NETWORKERROR(_) => "NETWORKERROR",
            AppError::RUNTIMEERROR(_) => "RUNTIMEERROR",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            AppError::DBERROR(message)
            | AppError::IOERROR(message)
            | AppError::VALIDATIONERROR(message)
            | AppError::NETWORKERROR(message)
            | AppError::RUNTIMEERROR(message) => message,
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::RUNTIMEERROR(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::VALIDATIONERROR(message.into())
    }
}

impl From<&AppError> for BackendErrorPayload {
    fn from(value: &AppError) -> Self {
        Self {
            code: value.code().to_string(),
            message: value.message().to_string(),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        AppError::RUNTIMEERROR(value.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::DBERROR(value.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        AppError::DBERROR(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::IOERROR(value.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(value: tauri::Error) -> Self {
        AppError::RUNTIMEERROR(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::VALIDATIONERROR(value.to_string())
    }
}

impl From<regex::Error> for AppError {
    fn from(value: regex::Error) -> Self {
        AppError::VALIDATIONERROR(value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        AppError::NETWORKERROR(value.to_string())
    }
}

impl From<image::ImageError> for AppError {
    fn from(value: image::ImageError) -> Self {
        AppError::IOERROR(value.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(value: url::ParseError) -> Self {
        AppError::VALIDATIONERROR(value.to_string())
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        AppError::VALIDATIONERROR(value.to_string())
    }
}

impl From<tauri_plugin_global_shortcut::Error> for AppError {
    fn from(value: tauri_plugin_global_shortcut::Error) -> Self {
        AppError::RUNTIMEERROR(value.to_string())
    }
}

impl From<arboard::Error> for AppError {
    fn from(value: arboard::Error) -> Self {
        AppError::IOERROR(value.to_string())
    }
}

pub fn emit_backend_error(app_handle: &AppHandle, error: &AppError) {
    if let Err(emit_err) = app_handle.emit("backend_error", BackendErrorPayload::from(error)) {
        log::error!(
            "Failed to emit backend_error event. original={}, emit={}",
            error,
            emit_err
        );
    }
}

pub fn with_error_event_sync<T>(
    app_handle: &AppHandle,
    operation: impl FnOnce() -> AppResult<T>,
) -> AppResult<T> {
    match operation() {
        Ok(value) => Ok(value),
        Err(error) => {
            emit_backend_error(app_handle, &error);
            Err(error)
        }
    }
}

pub async fn with_error_event<T, Fut>(app_handle: &AppHandle, operation: Fut) -> AppResult<T>
where
    Fut: Future<Output = AppResult<T>>,
{
    match operation.await {
        Ok(value) => Ok(value),
        Err(error) => {
            emit_backend_error(app_handle, &error);
            Err(error)
        }
    }
}
