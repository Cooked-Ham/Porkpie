use thiserror::Error;

/// Result type used by the core vault crate.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Errors returned by vault lifecycle, item management, and password generation.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Vault is locked")]
    VaultLocked,
    #[error("Vault is already unlocked")]
    AlreadyUnlocked,
    #[error("Vault is already locked")]
    AlreadyLocked,
    #[error("Item not found")]
    ItemNotFound,
    #[error("Invalid password length: expected 8..=128, got {0}")]
    InvalidPasswordLength(usize),
    #[error("No character sets selected for password generation")]
    NoCharacterSetsSelected,
    #[error("Password character set is empty after applying options")]
    EmptyCharacterSet,
    #[error("Wrong password")]
    WrongPassword,
    #[error("Invalid encrypted item")]
    InvalidEncryptedItem,
    #[error("Crypto error: {0}")]
    CryptoError(#[from] porkpie_crypto::CryptoError),
}
