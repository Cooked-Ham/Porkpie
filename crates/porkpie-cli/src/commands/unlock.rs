use crate::commands::{load_locked_vault, parse_vault_id, CommandContext};
use crate::errors::{map_core_error, Result};
use crate::interactive::{prompt_master_password, prompt_vault_id};
use crate::session::SessionState;

/// Unlock a vault and remember it as the current session vault.
pub async fn run(context: &CommandContext) -> Result<()> {
    let vault_id = parse_vault_id(&prompt_vault_id()?)?;
    let pool = context.pool().await?;
    let mut vault = load_locked_vault(&pool, vault_id).await?;
    let password = prompt_master_password()?;

    vault.unlock(&password).map_err(map_core_error)?;
    context.save_session(&SessionState::unlocked(vault_id))?;

    println!("Vault unlocked: {vault_id}");
    Ok(())
}
