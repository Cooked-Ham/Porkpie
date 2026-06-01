//! SQLite persistence for encrypted Porkpie vault data.
//!
//! The store crate persists encrypted blobs and vault metadata only. It does not
//! decrypt item ciphertext or handle master passwords.

pub mod db;
pub mod errors;
pub mod item_store;
pub mod migrations;
pub mod models;
pub mod sync_state;
pub mod vault_store;

pub use db::{connect, connect_database};
pub use errors::{Result, StoreError};
pub use item_store::{
    delete_item, load_item, load_item_record, load_item_records, load_items, load_items_with_type,
    load_items_with_type_since, store_item, update_item, upsert_item_revision,
};
pub use models::{EncryptedItemData, EncryptedVaultData, SyncState};
pub use sync_state::{load_sync_state, save_sync_state};
pub use vault_store::{delete_vault, load_vault, load_vault_by_name, store_vault};
