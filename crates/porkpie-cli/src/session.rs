use crate::errors::{CliError, Result};
use crate::secret_store::default_secret_store;
use porkpie_types::{LocalSecretKey, Timestamp, VaultId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SESSION_TIMEOUT_MILLIS: i64 = 30 * 60 * 1000;
const DEFAULT_SESSION_PATH: &str = ".porkpie-session.json";

/// Session state persisted to disk.
///
/// The session file does NOT store the local secret key. The secret key is
/// stored in the OS keychain (or an encrypted fallback) and loaded by vault_id.
/// The session file stores only non-secret metadata: vault identity, unlock
/// status, and activity timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub current_vault_id: Option<VaultId>,
    pub unlocked: bool,
    pub last_activity: Timestamp,
    /// Deprecated field: plaintext secret key storage. Never written by new
    /// code. Read once during migration to keychain, then removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_key_hex: Option<String>,
    /// Deprecated field: encrypted secret key storage. Never written by new
    /// code. Read once during migration to keychain, then removed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_key_encrypted: Option<String>,
    /// Flag set after migrating legacy session secrets to keychain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migrated: Option<bool>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            current_vault_id: None,
            unlocked: false,
            last_activity: Timestamp::now(),
            secret_key_hex: None,
            secret_key_encrypted: None,
            migrated: None,
        }
    }
}

impl SessionState {
    pub fn unlocked(vault_id: VaultId) -> Self {
        Self {
            current_vault_id: Some(vault_id),
            unlocked: true,
            last_activity: Timestamp::now(),
            secret_key_hex: None,
            secret_key_encrypted: None,
            migrated: None,
        }
    }

    pub fn lock(&mut self) {
        self.unlocked = false;
        self.last_activity = Timestamp::now();
    }

    pub fn require_unlocked_vault(&self) -> Result<VaultId> {
        if !self.unlocked {
            return Err(CliError::NoUnlockedSession);
        }

        if Timestamp::now().to_millis() - self.last_activity.to_millis() > SESSION_TIMEOUT_MILLIS {
            return Err(CliError::SessionExpired);
        }

        self.current_vault_id.ok_or(CliError::NoUnlockedSession)
    }

    /// Load the local secret key from the OS keychain or from legacy session
    /// fields during migration.
    pub fn require_secret_key(&self) -> Result<LocalSecretKey> {
        let vault_id = self.current_vault_id.ok_or(CliError::NoUnlockedSession)?;

        // Try OS keychain first.
        if let Some(store) = default_secret_store() {
            match store.load_local_secret_key(&vault_id) {
                Ok(Some(key)) => return Ok(key),
                Ok(None) => {}
                Err(e) => {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "[porkpie] keychain load failed: {e}; falling back to legacy session"
                    );
                }
            }
        }

        // Legacy fallback: encrypted storage (preferred over plaintext).
        if let Some(encrypted) = &self.secret_key_encrypted {
            return decrypt_secret_key(encrypted, &vault_id)
                .map_err(|_| CliError::NoUnlockedSession);
        }

        // Legacy fallback: plaintext hex.
        if let Some(hex) = &self.secret_key_hex {
            return LocalSecretKey::from_hex(hex).map_err(|_| CliError::NoUnlockedSession);
        }

        Err(CliError::NoUnlockedSession)
    }

    /// Migrate a legacy session secret to the OS keychain and clear the
    /// legacy fields from the session file.
    pub fn migrate_legacy_secret(&mut self) -> Result<()> {
        let store = match default_secret_store() {
            Some(s) => s,
            None => {
                #[cfg(debug_assertions)]
                eprintln!("[porkpie] no keychain available; skipping legacy migration");
                return Ok(());
            }
        };
        self.migrate_legacy_secret_with_store(&*store)
    }

    /// Migrate a legacy session secret using a provided store.
    /// Testable with FakeKeychain.
    pub fn migrate_legacy_secret_with_store(
        &mut self,
        store: &dyn crate::secret_store::SecretStore,
    ) -> Result<()> {
        let vault_id = match self.current_vault_id {
            Some(id) => id,
            None => return Ok(()),
        };

        if self.migrated == Some(true) {
            return Ok(());
        }

        // Try encrypted first, then plaintext.
        let key = if let Some(encrypted) = &self.secret_key_encrypted {
            decrypt_secret_key(encrypted, &vault_id).ok()
        } else if let Some(hex) = &self.secret_key_hex {
            LocalSecretKey::from_hex(hex).ok()
        } else {
            None
        };

        if let Some(key) = key {
            if let Err(e) = store.store_local_secret_key(&vault_id, &key) {
                #[cfg(debug_assertions)]
                eprintln!("[porkpie] keychain migration failed: {e}; keeping legacy field");
                return Ok(());
            }
            // Clear legacy fields.
            self.secret_key_hex = None;
            self.secret_key_encrypted = None;
            self.migrated = Some(true);
            #[cfg(debug_assertions)]
            eprintln!("[porkpie] migrated legacy session secret to OS keychain");
        }

        Ok(())
    }
}

/// Resolve the session path from CLI options or the environment.
pub fn session_path(override_path: Option<PathBuf>) -> PathBuf {
    override_path
        .or_else(|| std::env::var_os("PORKPIE_SESSION_PATH").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SESSION_PATH))
}

/// Load session state, returning a locked default session when no file exists.
pub fn load(path: &Path) -> Result<SessionState> {
    if !path.exists() {
        return Ok(SessionState::default());
    }

    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

/// Save session state to disk.
pub fn save(path: &Path, session: &SessionState) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let contents = serde_json::to_string_pretty(session)?;
    std::fs::write(path, contents)?;
    Ok(())
}

/// LEGACY: Derive a 32-byte session encryption key from the vault ID.
///
/// This is obfuscation-level protection: the vault ID is stored in the
/// session file, so an attacker with the file can derive the same key.
/// Never called by new code. Kept only for reading legacy session files.
fn derive_session_key(vault_id: &VaultId) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(vault_id.to_string().as_bytes());
    hasher.update(b"porkpie-session-v1");
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    key
}

/// LEGACY: Encrypt a secret key using a key derived from the vault ID.
/// Never called by new code. Kept only for reading legacy session files.
#[allow(dead_code)]
fn encrypt_secret_key(secret_key: &LocalSecretKey, vault_id: &VaultId) -> Option<String> {
    let key = derive_session_key(vault_id);
    let wrapped = porkpie_crypto::wrap_vault_key(&key, secret_key.as_bytes()).ok()?;
    Some(hex::encode(&wrapped))
}

/// LEGACY: Decrypt a secret key using a key derived from the vault ID.
/// Never called by new code. Kept only for reading legacy session files.
fn decrypt_secret_key(wrapped_hex: &str, vault_id: &VaultId) -> Result<LocalSecretKey> {
    let key = derive_session_key(vault_id);
    let wrapped = hex::decode(wrapped_hex).map_err(|_| CliError::NoUnlockedSession)?;
    let bytes = porkpie_crypto::unwrap_vault_key(&key, &wrapped)
        .map_err(|_| CliError::NoUnlockedSession)?;
    Ok(LocalSecretKey::from_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_store::SecretStore;

    #[test]
    fn legacy_migration_clears_secret_fields() {
        let vault_id = VaultId::new();
        let key = LocalSecretKey::generate();
        let mut session = SessionState::unlocked(vault_id);
        session.secret_key_hex = Some(key.to_hex());

        let fake = crate::secret_store::FakeKeychain::new();
        session.migrate_legacy_secret_with_store(&fake).unwrap();

        assert!(session.secret_key_hex.is_none());
        assert!(session.secret_key_encrypted.is_none());
        assert_eq!(session.migrated, Some(true));

        let loaded = fake.load_local_secret_key(&vault_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().to_hex(), key.to_hex());
    }

    #[test]
    fn legacy_migration_from_encrypted_clears_fields() {
        let vault_id = VaultId::new();
        let key = LocalSecretKey::generate();
        let mut session = SessionState::unlocked(vault_id);
        session.secret_key_encrypted = Some(encrypt_secret_key(&key, &vault_id).unwrap());

        let fake = crate::secret_store::FakeKeychain::new();
        session.migrate_legacy_secret_with_store(&fake).unwrap();

        assert!(session.secret_key_hex.is_none());
        assert!(session.secret_key_encrypted.is_none());
        assert_eq!(session.migrated, Some(true));

        let loaded = fake.load_local_secret_key(&vault_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().to_hex(), key.to_hex());
    }

    #[test]
    fn failed_migration_keeps_legacy_fields() {
        let vault_id = VaultId::new();
        let key = LocalSecretKey::generate();
        let mut session = SessionState::unlocked(vault_id);
        session.secret_key_hex = Some(key.to_hex());

        // Use a fake keychain that fails by not implementing store (but FakeKeychain never fails)
        // Instead, we test with a custom failing store.
        struct FailingStore;
        impl crate::secret_store::SecretStore for FailingStore {
            fn store_local_secret_key(
                &self,
                _vault_id: &VaultId,
                _key: &LocalSecretKey,
            ) -> crate::secret_store::Result<()> {
                Err(crate::secret_store::SecretStoreError::Unavailable(
                    "test failure".to_string(),
                ))
            }
            fn load_local_secret_key(
                &self,
                _vault_id: &VaultId,
            ) -> crate::secret_store::Result<Option<LocalSecretKey>> {
                Ok(None)
            }
            fn delete_local_secret_key(
                &self,
                _vault_id: &VaultId,
            ) -> crate::secret_store::Result<()> {
                Ok(())
            }
        }

        session
            .migrate_legacy_secret_with_store(&FailingStore)
            .unwrap();

        // Fields should be preserved because migration failed.
        assert!(session.secret_key_hex.is_some());
        assert_eq!(session.migrated, None);
    }

    #[test]
    fn new_session_never_contains_secret_material() {
        let vault_id = VaultId::new();
        let session = SessionState::unlocked(vault_id);
        let json = serde_json::to_string(&session).unwrap();
        assert!(
            !json.contains("secret_key_hex"),
            "session JSON should not store secret_key_hex"
        );
        assert!(
            !json.contains("secret_key_encrypted"),
            "session JSON should not store secret_key_encrypted"
        );
    }

    #[test]
    fn session_timeout_expires_after_inactivity() {
        let vault_id = VaultId::new();
        let mut session = SessionState::unlocked(vault_id);
        session.last_activity = Timestamp(0);
        let result = session.require_unlocked_vault();
        assert!(result.is_err(), "session should expire after inactivity");
    }
}
