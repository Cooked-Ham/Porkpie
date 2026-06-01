use crate::commands::CommandContext;
use crate::errors::{map_store_error, Result};
use crate::interactive::{prompt_new_master_password, prompt_vault_name};
use crate::session::SessionState;
use porkpie_core::{LocalSecretKey, Vault};
use porkpie_store::store_vault;

pub async fn run(context: &CommandContext) -> Result<()> {
    println!("Creating new vault");
    let name = prompt_vault_name()?;
    let password = prompt_new_master_password()?;
    let secret_key = LocalSecretKey::generate();
    let (vault, recovery_kit) = Vault::create(&name, &password, &secret_key)?;
    let pool = context.pool().await?;

    store_vault(&pool, &vault).await.map_err(map_store_error)?;
    context.save_session(&SessionState::unlocked_with_key(vault.id, &secret_key))?;

    let recovery_path = format!("porkpie-recovery-kit-{}.json", vault.id);
    let recovery_json = serde_json::to_string_pretty(&recovery_kit)?;
    std::fs::write(&recovery_path, &recovery_json)?;

    println!("Vault created: {} ({})", vault.name, vault.id);
    println!("Recovery kit saved to: {recovery_path}");
    println!("Store the recovery kit in a secure, offline location.");
    println!("Next: porkpie add login");
    Ok(())
}
