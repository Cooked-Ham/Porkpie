use crate::errors::{Result, StoreError};
use porkpie_types::{ItemId, VaultId};

pub use porkpie_core::{EncryptedItemData, EncryptedVaultData, SyncState};

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
