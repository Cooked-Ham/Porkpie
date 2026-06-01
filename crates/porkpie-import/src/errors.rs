use thiserror::Error;

/// Result alias for import/export helpers.
pub type Result<T> = std::result::Result<T, ImportError>;

/// Errors returned by import/export routines.
#[derive(Debug, Error)]
pub enum ImportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Core error: {0}")]
    Core(#[from] porkpie_core::CoreError),
    #[error("Store error: {0}")]
    Store(#[from] porkpie_store::StoreError),
    #[error("invalid backup version: {0}")]
    InvalidBackupVersion(u32),
    #[error("invalid row {row}: {message}")]
    InvalidRow { row: usize, message: String },
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("unsupported CSV item type: {0}")]
    UnsupportedItemType(String),
}
