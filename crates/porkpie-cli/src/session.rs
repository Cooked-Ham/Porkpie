use crate::errors::{CliError, Result};
use porkpie_types::{Timestamp, VaultId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SESSION_TIMEOUT_MILLIS: i64 = 30 * 60 * 1000;
const DEFAULT_SESSION_PATH: &str = ".porkpie-session.json";

/// Persisted session metadata. This file never stores passwords or vault keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub current_vault_id: Option<VaultId>,
    pub unlocked: bool,
    pub last_activity: Timestamp,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            current_vault_id: None,
            unlocked: false,
            last_activity: Timestamp::now(),
        }
    }
}

impl SessionState {
    /// Create a new unlocked session for a vault.
    pub fn unlocked(vault_id: VaultId) -> Self {
        Self {
            current_vault_id: Some(vault_id),
            unlocked: true,
            last_activity: Timestamp::now(),
        }
    }

    /// Mark the session locked without clearing the selected vault.
    pub fn lock(&mut self) {
        self.unlocked = false;
        self.last_activity = Timestamp::now();
    }

    /// Return the active vault id if the session is unlocked and not expired.
    pub fn require_unlocked_vault(&self) -> Result<VaultId> {
        if !self.unlocked {
            return Err(CliError::NoUnlockedSession);
        }

        if Timestamp::now().to_millis() - self.last_activity.to_millis() > SESSION_TIMEOUT_MILLIS {
            return Err(CliError::SessionExpired);
        }

        self.current_vault_id.ok_or(CliError::NoUnlockedSession)
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
