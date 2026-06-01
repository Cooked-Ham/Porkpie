//! SQLite persistence for encrypted Porkpie vault data.
//!
//! The store crate persists encrypted blobs and vault metadata only. It does not
//! decrypt item ciphertext or handle master passwords.

pub mod db;
pub mod errors;
pub mod item_store;
pub mod migrations;
pub mod models;
pub mod vault_store;

pub use db::{connect, connect_database};
pub use errors::{Result, StoreError};
pub use item_store::{
    delete_item, load_item, load_item_record, load_items, store_item, update_item,
};
pub use models::{EncryptedItemData, EncryptedVaultData, SyncState};
pub use vault_store::{delete_vault, load_vault, store_vault};
