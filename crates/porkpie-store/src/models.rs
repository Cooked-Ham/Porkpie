use crate::errors::{Result, StoreError};
use porkpie_core::Vault;
use porkpie_types::{ItemId, Timestamp, VaultId};
use serde::{Deserialize, Serialize};

/// Encrypted vault metadata loaded from SQLite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedVaultData {
    pub id: VaultId,
    pub name: String,
    pub created_at: Timestamp,
    pub salt: [u8; 32],
    pub master_key_wrapped: Vec<u8>,
    pub sync_revision: u64,
}

impl EncryptedVaultData {
    /// Convert encrypted metadata into a locked core vault.
    pub fn into_locked_vault(self) -> Vault {
        Vault::from_encrypted_metadata(
            self.id,
            self.name,
            self.created_at,
            self.salt,
            self.master_key_wrapped,
            self.sync_revision,
        )
    }
}

/// Encrypted item row for persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedItemData {
    pub id: ItemId,
    pub vault_id: VaultId,
    pub item_type: String,
    pub ciphertext: Vec<u8>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub sync_revision: u64,
}

impl EncryptedItemData {
    /// Create encrypted item data ready for storage.
    pub fn new(
        id: ItemId,
        vault_id: VaultId,
        item_type: impl Into<String>,
        ciphertext: Vec<u8>,
        created_at: Timestamp,
        updated_at: Timestamp,
        sync_revision: u64,
    ) -> Self {
        Self {
            id,
            vault_id,
            item_type: item_type.into(),
            ciphertext,
            created_at,
            updated_at,
            sync_revision,
        }
    }
}

/// Sync progress for a vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncState {
    pub vault_id: VaultId,
    pub last_synced_revision: Option<u64>,
    pub last_synced_at: Option<Timestamp>,
}

pub(crate) fn parse_vault_id(value: String) -> Result<VaultId> {
    VaultId::from_string(&value).map_err(|_| StoreError::InvalidVaultId(value))
}

pub(crate) fn parse_item_id(value: String) -> Result<ItemId> {
    ItemId::from_string(&value).map_err(|_| StoreError::InvalidItemId(value))
}

pub(crate) fn parse_salt(value: Vec<u8>) -> Result<[u8; 32]> {
    let length = value.len();
    value
        .try_into()
        .map_err(|_| StoreError::InvalidSaltLength(length))
}

pub(crate) fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

pub(crate) fn u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
