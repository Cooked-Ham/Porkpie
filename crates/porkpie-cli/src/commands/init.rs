use crate::commands::CommandContext;
use crate::errors::{map_store_error, Result};
use crate::interactive::prompt_new_master_password;
use crate::session::SessionState;
use porkpie_core::Vault;
use porkpie_store::store_vault;

/// Create and persist a new vault.
pub async fn run(context: &CommandContext) -> Result<()> {
    println!("Creating new vault");
    let password = prompt_new_master_password()?;
    let vault = Vault::create(&password)?;
    let pool = context.pool().await?;

    store_vault(&pool, &vault).await.map_err(map_store_error)?;
    context.save_session(&SessionState::unlocked(vault.id))?;

    println!("Vault created: {}", vault.id);
    println!("Next: porkpie add login");
    Ok(())
}
