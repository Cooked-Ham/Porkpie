//! OS keychain storage for the local secret key.
//!
//! Uses the `keyring` crate as a cross-platform abstraction over:
//! - Windows: Windows Credential Manager (DPAPI)
//! - macOS: Keychain
//! - Linux: Secret Service / libsecret
//!
//! A trait-based design allows tests to use an in-memory fake backend.

use porkpie_types::{LocalSecretKey, VaultId};

/// Error type for secret storage operations.
#[derive(Debug, thiserror::Error)]
pub enum SecretStoreError {
    #[error("OS keychain not available: {0}")]
    Unavailable(String),
    #[error("Secret not found for vault {0}")]
    NotFound(VaultId),
    #[error("IO error: {0}")]
    Io(String),
}

/// Result type for secret storage operations.
pub type Result<T> = std::result::Result<T, SecretStoreError>;

/// Abstraction over platform-specific secret storage.
pub trait SecretStore: Send + Sync {
    /// Store the local secret key for a vault.
    fn store_local_secret_key(&self, vault_id: &VaultId, key: &LocalSecretKey) -> Result<()>;

    /// Load the local secret key for a vault.
    fn load_local_secret_key(&self, vault_id: &VaultId) -> Result<Option<LocalSecretKey>>;

    /// Delete the local secret key for a vault.
    fn delete_local_secret_key(&self, vault_id: &VaultId) -> Result<()>;
}

/// OS keychain backend using the `keyring` crate.
#[cfg(feature = "keychain")]
pub struct OsKeychain;

#[cfg(feature = "keychain")]
impl SecretStore for OsKeychain {
    fn store_local_secret_key(&self, vault_id: &VaultId, key: &LocalSecretKey) -> Result<()> {
        let entry = keyring::Entry::new("porkpie", &vault_id.to_string())
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))?;
        entry
            .set_password(&key.to_hex())
            .map_err(|e| SecretStoreError::Io(e.to_string()))?;
        Ok(())
    }

    fn load_local_secret_key(&self, vault_id: &VaultId) -> Result<Option<LocalSecretKey>> {
        let entry = keyring::Entry::new("porkpie", &vault_id.to_string())
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))?;
        match entry.get_password() {
            Ok(hex) => LocalSecretKey::from_hex(&hex)
                .map(Some)
                .map_err(SecretStoreError::Io),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretStoreError::Io(e.to_string())),
        }
    }

    fn delete_local_secret_key(&self, vault_id: &VaultId) -> Result<()> {
        let entry = keyring::Entry::new("porkpie", &vault_id.to_string())
            .map_err(|e| SecretStoreError::Unavailable(e.to_string()))?;
        entry
            .delete_credential()
            .map_err(|e| SecretStoreError::Io(e.to_string()))?;
        Ok(())
    }
}

/// In-memory fake backend for tests.
pub struct FakeKeychain {
    data: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl FakeKeychain {
    pub fn new() -> Self {
        Self {
            data: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for FakeKeychain {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for FakeKeychain {
    fn store_local_secret_key(&self, vault_id: &VaultId, key: &LocalSecretKey) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.insert(vault_id.to_string(), key.to_hex());
        Ok(())
    }

    fn load_local_secret_key(&self, vault_id: &VaultId) -> Result<Option<LocalSecretKey>> {
        let data = self.data.lock().unwrap();
        match data.get(&vault_id.to_string()) {
            Some(hex) => LocalSecretKey::from_hex(hex)
                .map(Some)
                .map_err(SecretStoreError::Io),
            None => Ok(None),
        }
    }

    fn delete_local_secret_key(&self, vault_id: &VaultId) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.remove(&vault_id.to_string());
        Ok(())
    }
}

/// Attempt to create the best available secret store.
pub fn default_secret_store() -> Option<Box<dyn SecretStore>> {
    #[cfg(feature = "keychain")]
    {
        Some(Box::new(OsKeychain))
    }
    #[cfg(not(feature = "keychain"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_keychain_roundtrip() {
        let store = FakeKeychain::new();
        let vault_id = VaultId::new();
        let key = LocalSecretKey::generate();

        assert!(store.load_local_secret_key(&vault_id).unwrap().is_none());
        store.store_local_secret_key(&vault_id, &key).unwrap();
        let loaded = store.load_local_secret_key(&vault_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().to_hex(), key.to_hex());
        store.delete_local_secret_key(&vault_id).unwrap();
        assert!(store.load_local_secret_key(&vault_id).unwrap().is_none());
    }
}
