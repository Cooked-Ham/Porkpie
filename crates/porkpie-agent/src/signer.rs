//! SSH signer trait and in-memory implementations for Porkpie.

use thiserror::Error;

/// Error returned by SSH signing operations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{0}")]
pub struct SignerError(pub String);

/// Trait for SSH key signing operations.
///
/// Implementors must hold the private key in memory while the vault is unlocked
/// and must not write the private key to disk in plaintext.
pub trait SshSigner {
    /// SSH algorithm name, e.g. `ssh-ed25519`.
    fn algorithm(&self) -> &'static str;

    /// Raw public key bytes (not the OpenSSH line format).
    fn public_key_bytes(&self) -> &[u8];

    /// Sign `data` and return the signature bytes.
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SignerError>;
}
