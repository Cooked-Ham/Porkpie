//! Core vault lifecycle and in-memory item management for Porkpie.

pub mod errors;
pub mod item;
pub mod models;
pub mod password_gen;
pub mod state;
pub mod vault;

pub use errors::{CoreError, Result};
pub use item::Item;
pub use models::{EncryptedItemData, EncryptedVaultData, SyncState};
pub use password_gen::{generate_password, PasswordOptions};
pub use state::VaultState;
pub use vault::Vault;

pub use porkpie_types::{LocalSecretKey, RecoveryKit};

pub use porkpie_crypto::Argon2Params;
