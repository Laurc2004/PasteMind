use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("DATABASE_ERROR: {0}")]
  Database(String),
  #[error("IO_ERROR: {0}")]
  Io(String),
  #[error("SERDE_ERROR: {0}")]
  Serde(String),
  #[error("CLIPBOARD_ERROR: {0}")]
  Clipboard(String),
  #[error("AUTOPASTE_ERROR: {0}")]
  AutoPaste(String),
  #[error("PERMISSION_DENIED: Accessibility permission is required for auto paste")]
  PermissionDenied,
  #[error("ENTRY_NOT_FOUND")]
  EntryNotFound,
  #[error("INVALID_IMAGE_DATA")]
  InvalidImageData,
  #[error("INVALID_SETTINGS: {0}")]
  InvalidSettings(String),
  #[error("WINDOW_NOT_FOUND: {0}")]
  WindowNotFound(String),
  #[error("SHORTCUT_ERROR: {0}")]
  Shortcut(String),
  #[error("TAURI_ERROR: {0}")]
  Tauri(String),
  #[error("OTHER_ERROR: {0}")]
  Other(String)
}

pub type AppResult<T> = Result<T, AppError>;

impl From<rusqlite::Error> for AppError {
  fn from(value: rusqlite::Error) -> Self {
    Self::Database(value.to_string())
  }
}

impl From<std::io::Error> for AppError {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value.to_string())
  }
}

impl From<serde_json::Error> for AppError {
  fn from(value: serde_json::Error) -> Self {
    Self::Serde(value.to_string())
  }
}

impl From<tauri::Error> for AppError {
  fn from(value: tauri::Error) -> Self {
    Self::Tauri(value.to_string())
  }
}
