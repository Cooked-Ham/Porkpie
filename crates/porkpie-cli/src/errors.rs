use porkpie_types::{ItemId, VaultId};
use thiserror::Error;

/// Result type used by the CLI.
pub type Result<T> = std::result::Result<T, CliError>;

/// Errors returned by CLI commands.
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Password must be at least {min} characters")]
    PasswordTooShort { min: usize },
    #[error("Invalid vault id: {0}")]
    InvalidVaultId(String),
    #[error("Invalid item id: {0}")]
    InvalidItemId(String),
    #[error("Vault not found: {0}")]
    VaultNotFound(VaultId),
    #[error("Item not found: {0}")]
    ItemNotFound(ItemId),
    #[error("Item not found: {0}")]
    ItemNotFoundByName(String),
    #[error("Invalid password")]
    InvalidPassword,
    #[error("No unlocked session. Run `porkpie unlock` first")]
    NoUnlockedSession,
    #[error("Session expired. Run `porkpie unlock` again")]
    SessionExpired,
    #[error("Unsupported item type: {0}")]
    UnsupportedItemType(String),
    #[error("Invalid pie:// URI: {0}")]
    InvalidPieUri(String),
    #[error("Field error: {0}")]
    FieldError(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Prompt error: {0}")]
    Prompt(#[from] dialoguer::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Core error: {0}")]
    Core(#[from] porkpie_core::CoreError),
    #[error("Store error: {0}")]
    Store(#[from] porkpie_store::StoreError),
    #[error("Import/export error: {0}")]
    Import(#[from] porkpie_import::ImportError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Sync server returned {status}: {message}")]
    SyncHttp {
        status: reqwest::StatusCode,
        message: String,
    },
}

impl From<uuid::Error> for CliError {
    fn from(error: uuid::Error) -> Self {
        Self::InvalidItemId(error.to_string())
    }
}

/// Convert storage errors into friendlier CLI errors.
pub fn map_store_error(error: porkpie_store::StoreError) -> CliError {
    match error {
        porkpie_store::StoreError::VaultNotFound(id) => CliError::VaultNotFound(id),
        porkpie_store::StoreError::ItemNotFound(id) => CliError::ItemNotFound(id),
        other => CliError::Store(other),
    }
}

/// Convert core errors into friendlier CLI errors.
pub fn map_core_error(error: porkpie_core::CoreError) -> CliError {
    match error {
        porkpie_core::CoreError::WrongPassword => CliError::InvalidPassword,
        other => CliError::Core(other),
    }
}
