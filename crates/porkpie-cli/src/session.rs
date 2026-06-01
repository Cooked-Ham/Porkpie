use crate::errors::{CliError, Result};
use porkpie_types::{LocalSecretKey, Timestamp, VaultId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SESSION_TIMEOUT_MILLIS: i64 = 30 * 60 * 1000;
const DEFAULT_SESSION_PATH: &str = ".porkpie-session.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub current_vault_id: Option<VaultId>,
    pub unlocked: bool,
    pub last_activity: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_key_hex: Option<String>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            current_vault_id: None,
            unlocked: false,
            last_activity: Timestamp::now(),
            secret_key_hex: None,
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
        }
    }

    pub fn unlocked_with_key(vault_id: VaultId, secret_key: &LocalSecretKey) -> Self {
        Self {
            current_vault_id: Some(vault_id),
            unlocked: true,
            last_activity: Timestamp::now(),
            secret_key_hex: Some(secret_key.to_hex()),
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
