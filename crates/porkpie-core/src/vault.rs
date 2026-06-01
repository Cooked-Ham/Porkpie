use crate::errors::{CoreError, Result};
use crate::{Item, VaultState};
use porkpie_crypto::{
    decrypt_item, derive_key, encrypt_item, unwrap_vault_key, wrap_vault_key, Argon2Params,
    CryptoError,
};
use porkpie_types::{
    build_item_aad, build_payload_aad, ItemId, ItemType, LocalSecretKey, RecoveryKit, Timestamp,
    VaultId,
};
use rand::{rngs::OsRng, RngCore};
use secrecy::Secret;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use zeroize::Zeroizing;

pub struct Vault {
    pub id: VaultId,
    pub name: String,
    pub created_at: Timestamp,
    pub salt: [u8; 32],
    master_key_wrapped: Vec<u8>,
    items: HashMap<ItemId, Item>,
    is_locked: bool,
    sync_revision: u64,
    vault_key: Option<Zeroizing<[u8; 32]>>,
}

impl Vault {
    pub fn create(
        name: &str,
        password: &str,
        secret_key: &LocalSecretKey,
    ) -> Result<(Self, RecoveryKit)> {
        let salt = random_bytes();
        let password = Secret::new(password.to_owned());
        let master_key = Zeroizing::new(derive_key(
            &password,
            secret_key.as_bytes(),
            &salt,
            &Argon2Params::default(),
        )?);
        let vault_key = Zeroizing::new(random_bytes());
        let master_key_wrapped = wrap_vault_key(&master_key, &vault_key)?;

        let id = VaultId::new();
        let created_at = Timestamp::now();
        let recovery_kit = RecoveryKit::new(&id.to_string(), secret_key, created_at.to_millis());

        Ok((
            Self {
                id,
                name: name.to_string(),
                created_at,
                salt,
                master_key_wrapped,
                items: HashMap::new(),
                is_locked: false,
                sync_revision: 0,
                vault_key: Some(vault_key),
            },
            recovery_kit,
        ))
    }

    pub fn from_encrypted_metadata(
        id: VaultId,
        name: String,
        created_at: Timestamp,
        salt: [u8; 32],
        master_key_wrapped: Vec<u8>,
        sync_revision: u64,
    ) -> Self {
        Self {
            id,
            name,
            created_at,
            salt,
            master_key_wrapped,
            items: HashMap::new(),
            is_locked: true,
            sync_revision,
            vault_key: None,
        }
    }

    pub fn unlock(&mut self, password: &str, secret_key: &LocalSecretKey) -> Result<()> {
        if !self.is_locked {
            return Err(CoreError::AlreadyUnlocked);
        }

        let password = Secret::new(password.to_owned());
        let master_key = Zeroizing::new(derive_key(
            &password,
            secret_key.as_bytes(),
            &self.salt,
            &Argon2Params::default(),
        )?);
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

    pub fn state(&self) -> VaultState {
        if self.is_locked {
            VaultState::Locked
        } else {
            VaultState::Unlocked
        }
    }

    pub fn create_item(&mut self, item: Item) -> Result<ItemId> {
        self.ensure_unlocked()?;

        let id = ItemId::new();
        let item = item.into_created(id, Timestamp::now());
        self.items.insert(id, item);
        self.bump_revision();
        Ok(id)
    }

    pub fn get_item(&self, id: ItemId) -> Result<&Item> {
        self.ensure_unlocked()?;
        self.items.get(&id).ok_or(CoreError::ItemNotFound)
    }

    pub fn list_items(&self) -> Result<Vec<&Item>> {
        self.ensure_unlocked()?;
        Ok(self.items.values().collect())
    }

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

    pub fn delete_item(&mut self, id: ItemId) -> Result<()> {
        self.ensure_unlocked()?;

        self.items.remove(&id).ok_or(CoreError::ItemNotFound)?;
        self.bump_revision();
        Ok(())
    }

    pub fn encrypt_item(&self, item: &Item) -> Result<Vec<u8>> {
        self.ensure_unlocked()?;
        let vault_key = self.vault_key.as_ref().ok_or(CoreError::VaultLocked)?;
        let aad = build_item_aad(
            &self.id.to_string(),
            &item.id.to_string(),
            item_type_name(&item.data),
        );
        Ok(encrypt_item(item, vault_key, &aad)?)
    }

    pub fn decrypt_item(
        &self,
        ciphertext: &[u8],
        item_id: &ItemId,
        item_type: &str,
    ) -> Result<Item> {
        self.ensure_unlocked()?;
        let vault_key = self.vault_key.as_ref().ok_or(CoreError::VaultLocked)?;
        let aad = build_item_aad(&self.id.to_string(), &item_id.to_string(), item_type);
        decrypt_item(ciphertext, vault_key, &aad).map_err(CoreError::CryptoError)
    }

    pub fn encrypt_payload<T: Serialize>(&self, payload: &T) -> Result<Vec<u8>> {
        self.ensure_unlocked()?;
        let vault_key = self.vault_key.as_ref().ok_or(CoreError::VaultLocked)?;
        let aad = build_payload_aad(&self.id.to_string());
        Ok(encrypt_item(payload, vault_key, &aad)?)
    }

    pub fn decrypt_payload<T: DeserializeOwned>(&self, ciphertext: &[u8]) -> Result<T> {
        self.ensure_unlocked()?;
        let vault_key = self.vault_key.as_ref().ok_or(CoreError::VaultLocked)?;
        let aad = build_payload_aad(&self.id.to_string());
        decrypt_item(ciphertext, vault_key, &aad).map_err(CoreError::CryptoError)
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

    pub fn master_key_wrapped(&self) -> &Vec<u8> {
        &self.master_key_wrapped
    }

    pub fn sync_revision(&self) -> u64 {
        self.sync_revision
    }

    pub fn items(&self) -> &HashMap<ItemId, Item> {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut HashMap<ItemId, Item> {
        &mut self.items
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Rotate the vault key: generate a new vault key, re-encrypt all items,
    /// and re-wrap the new vault key with the master key derived from the
    /// password and secret key.
    ///
    /// Returns the re-encrypted item ciphertexts so the caller can persist them.
    /// The vault must be unlocked before calling this method.
    pub fn rotate_vault_key(
        &mut self,
        password: &str,
        secret_key: &LocalSecretKey,
    ) -> Result<Vec<(ItemId, Vec<u8>)>> {
        self.ensure_unlocked()?;

        let password = Secret::new(password.to_owned());
        let master_key = Zeroizing::new(derive_key(
            &password,
            secret_key.as_bytes(),
            &self.salt,
            &Argon2Params::default(),
        )?);
        let new_vault_key = Zeroizing::new(random_bytes());
        let new_master_key_wrapped = wrap_vault_key(&master_key, &new_vault_key)?;

        // Replace the old vault key (it will be zeroized on drop)
        self.vault_key = Some(Zeroizing::new(*new_vault_key));
        self.master_key_wrapped = new_master_key_wrapped;
        self.bump_revision();

        // Re-encrypt all items with the new vault key
        let mut re_encrypted = Vec::new();
        for item in self.items.values() {
            let ciphertext = self.encrypt_item(item)?;
            re_encrypted.push((item.id, ciphertext));
        }

        Ok(re_encrypted)
    }
}

fn item_type_name(item_type: &ItemType) -> &str {
    match item_type {
        ItemType::Login(_) => "Login",
        ItemType::APIKey(_) => "APIKey",
        ItemType::SSHKey(_) => "SSHKey",
        ItemType::SecureNote(_) => "SecureNote",
        ItemType::Server(_) => "Server",
        ItemType::Database(_) => "Database",
        ItemType::Identity(_) => "Identity",
        ItemType::SoftwareLicense(_) => "SoftwareLicense",
        ItemType::RecoveryCodes(_) => "RecoveryCodes",
        ItemType::Custom(_) => "Custom",
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
