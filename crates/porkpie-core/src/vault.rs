use crate::errors::{CoreError, Result};
use crate::{Item, VaultState};
use porkpie_crypto::{
    decrypt_item, derive_key, encrypt_item, unwrap_vault_key, wrap_vault_key, Argon2Params,
    CryptoError,
};
use porkpie_types::{ItemId, Timestamp, VaultId};
use rand::{rngs::OsRng, RngCore};
use secrecy::Secret;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use zeroize::Zeroizing;

/// In-memory vault domain object.
///
/// Decrypted items and the raw vault key are only present while the vault is unlocked.
/// Persistent fields such as `master_key_wrapped` and `salt` are encrypted metadata.
pub struct Vault {
    pub id: VaultId,
    pub created_at: Timestamp,
    pub salt: [u8; 32],
    pub master_key_wrapped: Vec<u8>,
    pub items: HashMap<ItemId, Item>,
    pub is_locked: bool,
    pub sync_revision: u64,
    vault_key: Option<Zeroizing<[u8; 32]>>,
}

impl Vault {
    /// Create a new unlocked vault from a master password.
    pub fn create(password: &str) -> Result<Self> {
        let salt = random_bytes();
        let password = Secret::new(password.to_owned());
        let master_key = Zeroizing::new(derive_key(&password, &salt, &Argon2Params::default())?);
        let vault_key = Zeroizing::new(random_bytes());
        let master_key_wrapped = wrap_vault_key(&master_key, &vault_key)?;

        Ok(Self {
            id: VaultId::new(),
            created_at: Timestamp::now(),
            salt,
            master_key_wrapped,
            items: HashMap::new(),
            is_locked: false,
            sync_revision: 0,
            vault_key: Some(vault_key),
        })
    }

    /// Reconstruct a locked vault from encrypted metadata.
    pub fn from_encrypted_metadata(
        id: VaultId,
        created_at: Timestamp,
        salt: [u8; 32],
        master_key_wrapped: Vec<u8>,
        sync_revision: u64,
    ) -> Self {
        Self {
            id,
            created_at,
            salt,
            master_key_wrapped,
            items: HashMap::new(),
            is_locked: true,
            sync_revision,
            vault_key: None,
        }
    }

    /// Unlock the vault with the master password.
    pub fn unlock(&mut self, password: &str) -> Result<()> {
        if !self.is_locked {
            return Err(CoreError::AlreadyUnlocked);
        }

        let password = Secret::new(password.to_owned());
        let master_key =
            Zeroizing::new(derive_key(&password, &self.salt, &Argon2Params::default())?);
        let vault_key =
            unwrap_vault_key(&master_key, &self.master_key_wrapped).map_err(|error| {
                if matches!(
                    error,
                    CryptoError::WrongPassword | CryptoError::DecryptionFailed
                ) {
                    CoreError::WrongPassword
                } else {
                    CoreError::CryptoError(error)
                }
            })?;

        self.vault_key = Some(Zeroizing::new(vault_key));
        self.is_locked = false;
        Ok(())
    }

    /// Lock the vault, clearing decrypted items and zeroizing the in-memory vault key.
    pub fn lock(&mut self) -> Result<()> {
        if self.is_locked {
            return Err(CoreError::AlreadyLocked);
        }

        for item in self.items.values_mut() {
            item.zeroize_secret_material();
        }
        self.items.clear();
        self.vault_key = None;
        self.is_locked = true;
        Ok(())
    }

    /// Return the current vault state.
    pub fn state(&self) -> VaultState {
        if self.is_locked {
            VaultState::Locked
        } else {
            VaultState::Unlocked
        }
    }

    /// Create an item in the unlocked vault and return its assigned identifier.
    pub fn create_item(&mut self, item: Item) -> Result<ItemId> {
        self.ensure_unlocked()?;

        let id = ItemId::new();
        let item = item.into_created(id, Timestamp::now());
        self.items.insert(id, item);
        self.bump_revision();
        Ok(id)
    }

    /// Get an item by identifier from the unlocked vault.
    pub fn get_item(&self, id: ItemId) -> Result<&Item> {
        self.ensure_unlocked()?;
        self.items.get(&id).ok_or(CoreError::ItemNotFound)
    }

    /// List all in-memory items from the unlocked vault.
    pub fn list_items(&self) -> Result<Vec<&Item>> {
        self.ensure_unlocked()?;
        Ok(self.items.values().collect())
    }

    /// Update an item by identifier in the unlocked vault.
    pub fn update_item(&mut self, id: ItemId, item: Item) -> Result<()> {
        self.ensure_unlocked()?;

        let created_at = self
            .items
            .get(&id)
            .map(|existing| existing.created_at)
            .ok_or(CoreError::ItemNotFound)?;
        let item = item.into_updated(id, created_at, Timestamp::now());
        self.items.insert(id, item);
        self.bump_revision();
        Ok(())
    }

    /// Delete an item by identifier from the unlocked vault.
    pub fn delete_item(&mut self, id: ItemId) -> Result<()> {
        self.ensure_unlocked()?;

        self.items.remove(&id).ok_or(CoreError::ItemNotFound)?;
        self.bump_revision();
        Ok(())
    }

    /// Encrypt a decrypted item with the in-memory vault key.
    pub fn encrypt_item(&self, item: &Item) -> Result<Vec<u8>> {
        self.ensure_unlocked()?;
        let vault_key = self.vault_key.as_ref().ok_or(CoreError::VaultLocked)?;
        Ok(encrypt_item(item, vault_key)?)
    }

    /// Decrypt an encrypted item blob with the in-memory vault key.
    pub fn decrypt_item(&self, ciphertext: &[u8]) -> Result<Item> {
        self.ensure_unlocked()?;
        let vault_key = self.vault_key.as_ref().ok_or(CoreError::VaultLocked)?;
        decrypt_item(ciphertext, vault_key).map_err(CoreError::CryptoError)
    }

    /// Encrypt a serializable vault-scoped payload with the in-memory vault key.
    pub fn encrypt_payload<T: Serialize>(&self, payload: &T) -> Result<Vec<u8>> {
        self.ensure_unlocked()?;
        let vault_key = self.vault_key.as_ref().ok_or(CoreError::VaultLocked)?;
        Ok(encrypt_item(payload, vault_key)?)
    }

    /// Decrypt a vault-scoped payload with the in-memory vault key.
    pub fn decrypt_payload<T: DeserializeOwned>(&self, ciphertext: &[u8]) -> Result<T> {
        self.ensure_unlocked()?;
        let vault_key = self.vault_key.as_ref().ok_or(CoreError::VaultLocked)?;
        decrypt_item(ciphertext, vault_key).map_err(CoreError::CryptoError)
    }

    fn ensure_unlocked(&self) -> Result<()> {
        if self.is_locked {
            Err(CoreError::VaultLocked)
        } else {
            Ok(())
        }
    }

    fn bump_revision(&mut self) {
        self.sync_revision = self.sync_revision.saturating_add(1);
    }
}

fn random_bytes() -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::Vault;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn vault_is_send_and_sync() {
        assert_send_sync::<Vault>();
    }
}
