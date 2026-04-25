use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::future::Future;
use tauri::{AppHandle, Emitter};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    DbError(String),
    #[error("{0}")]
    IoError(String),
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    NetworkError(String),
    #[error("{0}")]
    RuntimeError(String),

    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Tauri(#[from] tauri::Error),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("{0}")]
    Regex(#[from] regex::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Image(#[from] image::ImageError),
    #[error("{0}")]
    UrlParse(#[from] url::ParseError),
    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    GlobalShortcut(#[from] tauri_plugin_global_shortcut::Error),
    #[error("{0}")]
    Clipboard(#[from] arboard::Error),
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
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
            AppError::DbError(_) | AppError::Sqlx(_) | AppError::SqlxMigrate(_) => "DBERROR",
            AppError::IoError(_)
            | AppError::Io(_)
            | AppError::Image(_)
            | AppError::Clipboard(_) => "IOERROR",
            AppError::ValidationError(_)
            | AppError::SerdeJson(_)
            | AppError::Regex(_)
            | AppError::UrlParse(_)
            | AppError::Utf8(_) => "VALIDATIONERROR",
            AppError::NetworkError(_) | AppError::Reqwest(_) => "NETWORKERROR",
            AppError::RuntimeError(_)
            | AppError::Tauri(_)
            | AppError::GlobalShortcut(_)
            | AppError::Anyhow(_) => "RUNTIMEERROR",
        }
    }

    pub fn message(&self) -> String {
        self.to_string()
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::RuntimeError(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }
}

impl From<&AppError> for BackendErrorPayload {
    fn from(value: &AppError) -> Self {
        Self {
            code: value.code().to_string(),
            message: value.message(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("type", self.code())?;
        state.serialize_field("description", &self.message())?;
        state.end()
    }
}

pub fn emit_backend_error(app_handle: &AppHandle, error: &AppError) {
    // Surface backend failures to the UI error/toast channel.
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
