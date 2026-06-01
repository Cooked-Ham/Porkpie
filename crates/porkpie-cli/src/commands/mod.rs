use crate::errors::{map_core_error, map_store_error, CliError, Result};
use crate::session::{self, SessionState};
use porkpie_core::Vault;
use porkpie_store::{connect_database, load_vault};
use porkpie_types::{ItemId, VaultId};
use sqlx::SqlitePool;
use std::path::PathBuf;

pub mod add;
pub mod delete;
pub mod edit;
pub mod export;
pub mod get;
pub mod import;
pub mod init;
pub mod list;
pub mod lock;
pub mod sync;
pub mod unlock;

const DEFAULT_DATABASE_URL: &str = "sqlite:porkpie.db";

/// Shared command context derived from global CLI options.
#[derive(Debug, Clone)]
pub struct CommandContext {
    database_url: String,
    session_path: PathBuf,
}

impl CommandContext {
    /// Build a command context from optional CLI overrides.
    pub fn new(database_url: Option<String>, session_path: Option<PathBuf>) -> Self {
        let database_url = database_url
            .or_else(|| std::env::var("PORKPIE_DATABASE_URL").ok())
            .unwrap_or_else(|| DEFAULT_DATABASE_URL.to_string());

        Self {
            database_url,
            session_path: session::session_path(session_path),
        }
    }

    /// Connect to the configured SQLite database.
    pub async fn pool(&self) -> Result<SqlitePool> {
        connect_database(&self.database_url)
            .await
            .map_err(map_store_error)
    }

    /// Load the current session.
    pub fn load_session(&self) -> Result<SessionState> {
        session::load(&self.session_path)
    }

    /// Save session state.
    pub fn save_session(&self, state: &SessionState) -> Result<()> {
        session::save(&self.session_path, state)
    }
}

async fn load_locked_vault(pool: &SqlitePool, vault_id: VaultId) -> Result<Vault> {
    Ok(load_vault(pool, &vault_id)
        .await
        .map_err(map_store_error)?
        .into_locked_vault())
}

pub(crate) async fn unlock_current_vault(context: &CommandContext) -> Result<Vault> {
    let session = context.load_session()?;
    let vault_id = session.require_unlocked_vault()?;
    let pool = context.pool().await?;
    let mut vault = load_locked_vault(&pool, vault_id).await?;
    let password = crate::interactive::prompt_master_password()?;
    vault.unlock(&password).map_err(map_core_error)?;

    let updated = SessionState::unlocked(vault_id);
    context.save_session(&updated)?;
    Ok(vault)
}

pub(crate) fn parse_vault_id(value: &str) -> Result<VaultId> {
    VaultId::from_string(value).map_err(|_| CliError::InvalidVaultId(value.to_string()))
}

pub(crate) fn parse_item_id(value: &str) -> Result<ItemId> {
    ItemId::from_string(value).map_err(|_| CliError::InvalidItemId(value.to_string()))
}
