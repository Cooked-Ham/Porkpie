use porkpie_types::{ItemId, VaultId};
use thiserror::Error;

/// Result type used by the SQLite store crate.
pub type Result<T> = std::result::Result<T, StoreError>;

/// Errors returned by encrypted vault persistence operations.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Vault not found: {0}")]
    VaultNotFound(VaultId),
    #[error("Item not found: {0}")]
    ItemNotFound(ItemId),
    #[error("Database constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("Invalid vault id loaded from database: {0}")]
    InvalidVaultId(String),
    #[error("Invalid item id loaded from database: {0}")]
    InvalidItemId(String),
    #[error("Invalid salt length: expected 32 bytes, got {0}")]
    InvalidSaltLength(usize),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub(crate) fn map_sqlx_error(error: sqlx::Error) -> StoreError {
    if let sqlx::Error::Database(database_error) = &error {
        let message = database_error.message().to_string();
        let lower = message.to_ascii_lowercase();
        if lower.contains("constraint") || lower.contains("foreign key") || lower.contains("unique")
        {
            return StoreError::ConstraintViolation(message);
        }
    }

    StoreError::DatabaseError(error.to_string())
}
