use crate::commands::{load_locked_vault, parse_vault_id, CommandContext};
use crate::errors::{map_core_error, Result};
use crate::interactive::{prompt_master_password, prompt_vault_id};
use crate::session::SessionState;
use porkpie_types::LocalSecretKey;

pub async fn run(context: &CommandContext) -> Result<()> {
    let vault_id = parse_vault_id(&prompt_vault_id()?)?;
    let pool = context.pool().await?;
    let mut vault = load_locked_vault(&pool, vault_id).await?;
    let password = prompt_master_password()?;

    println!("Enter local secret key (from recovery kit):");
    let secret_key_hex = dialoguer::Input::<String>::new()
        .with_prompt("Secret key (hex)")
        .interact_text()
        .map_err(|e| crate::errors::CliError::Io(std::io::Error::other(e)))?;
    let secret_key = LocalSecretKey::from_hex(&secret_key_hex)
        .map_err(crate::errors::CliError::InvalidArgument)?;

    vault
        .unlock(&password, &secret_key)
        .map_err(map_core_error)?;
    context.save_session(&SessionState::unlocked_with_key(vault_id, &secret_key))?;

    println!("Vault unlocked: {vault_id}");
    Ok(())
}
