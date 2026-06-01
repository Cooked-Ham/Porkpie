use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault not found: {0}")]
    NotFound(String),

    #[error("Vault already locked")]
    AlreadyLocked,

    #[error("Vault not unlocked")]
    NotUnlocked,

    #[error("Invalid item ID")]
    InvalidItemId,
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed: ciphertext tampered")]
    DecryptionFailed,

    #[error("Wrong password")]
    WrongPassword,

    #[error("Invalid nonce")]
    InvalidNonce,
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection failed")]
    ConnectionFailed,

    #[error("Query execution failed")]
    QueryFailed,

    #[error("Item already exists")]
    ItemAlreadyExists,
}

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Network connection failed")]
    NetworkFailed,

    #[error("Sync server returned error code: {0}")]
    ServerError(u16),

    #[error("Conflict during sync resolution")]
    Conflict,
}
