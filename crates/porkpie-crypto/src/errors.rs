use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed: ciphertext tampered")]
    DecryptionFailed,

    #[error("Wrong password or Invalid Key")]
    WrongPassword,

    #[error("Invalid nonce length")]
    InvalidNonce,

    #[error("Argon2 error: {0}")]
    Argon2Error(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid ciphertext format")]
    InvalidCiphertext,
}

impl From<argon2::password_hash::Error> for CryptoError {
    fn from(err: argon2::password_hash::Error) -> Self {
        CryptoError::Argon2Error(err.to_string())
    }
}
