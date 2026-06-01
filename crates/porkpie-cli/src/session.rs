use crate::errors::{CliError, Result};
use porkpie_types::{LocalSecretKey, Timestamp, VaultId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const SESSION_TIMEOUT_MILLIS: i64 = 30 * 60 * 1000;
const DEFAULT_SESSION_PATH: &str = ".porkpie-session.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub current_vault_id: Option<VaultId>,
    pub unlocked: bool,
    pub last_activity: Timestamp,
    /// Deprecated: plaintext secret key storage. Use `secret_key_encrypted` instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_key_hex: Option<String>,
    /// Encrypted secret key (hex-encoded nonce+ciphertext). Decrypted with a key derived from the vault_id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_key_encrypted: Option<String>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            current_vault_id: None,
            unlocked: false,
            last_activity: Timestamp::now(),
            secret_key_hex: None,
            secret_key_encrypted: None,
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
        }
    }

    pub fn unlocked_with_key(vault_id: VaultId, secret_key: &LocalSecretKey) -> Self {
        let encrypted = encrypt_secret_key(secret_key, &vault_id);
        Self {
            current_vault_id: Some(vault_id),
            unlocked: true,
            last_activity: Timestamp::now(),
            secret_key_hex: None,
            secret_key_encrypted: encrypted,
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

    pub fn require_secret_key(&self) -> Result<LocalSecretKey> {
        // Prefer encrypted storage; fall back to deprecated plaintext for backward compatibility.
        if let Some(encrypted) = &self.secret_key_encrypted {
            let vault_id = self.current_vault_id.ok_or(CliError::NoUnlockedSession)?;
            return decrypt_secret_key(encrypted, &vault_id)
                .map_err(|_| CliError::NoUnlockedSession);
        }
        let hex = self
            .secret_key_hex
            .as_ref()
            .ok_or(CliError::NoUnlockedSession)?;
        LocalSecretKey::from_hex(hex).map_err(|_| CliError::NoUnlockedSession)
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

/// Derive a 32-byte session encryption key from the vault ID.
///
/// This is obfuscation-level protection: the vault ID is stored in the
/// session file, so an attacker with the file can derive the same key.
/// It raises the bar from "read plaintext" to "reverse the key derivation".
fn derive_session_key(vault_id: &VaultId) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(vault_id.to_string().as_bytes());
    hasher.update(b"porkpie-session-v1");
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    key
}

/// Encrypt a secret key using a key derived from the vault ID.
fn encrypt_secret_key(secret_key: &LocalSecretKey, vault_id: &VaultId) -> Option<String> {
    let key = derive_session_key(vault_id);
    let wrapped = porkpie_crypto::wrap_vault_key(&key, secret_key.as_bytes()).ok()?;
    Some(hex::encode(&wrapped))
}

/// Decrypt a secret key using a key derived from the vault ID.
fn decrypt_secret_key(wrapped_hex: &str, vault_id: &VaultId) -> Result<LocalSecretKey> {
    let key = derive_session_key(vault_id);
    let wrapped = hex::decode(wrapped_hex).map_err(|_| CliError::NoUnlockedSession)?;
    let bytes = porkpie_crypto::unwrap_vault_key(&key, &wrapped)
        .map_err(|_| CliError::NoUnlockedSession)?;
    Ok(LocalSecretKey::from_bytes(&bytes))
}
