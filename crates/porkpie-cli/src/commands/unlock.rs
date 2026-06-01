use crate::commands::{load_locked_vault, parse_vault_id, CommandContext};
use crate::errors::{map_core_error, Result};
use crate::interactive::{prompt_master_password, prompt_secret, prompt_vault_id};
use crate::secret_store::default_secret_store;
use crate::session::SessionState;
use porkpie_types::LocalSecretKey;

pub async fn run(context: &CommandContext) -> Result<()> {
    let vault_id = parse_vault_id(&prompt_vault_id()?)?;
    let pool = context.pool().await?;
    let mut vault = load_locked_vault(&pool, vault_id).await?;
    let password = prompt_master_password()?;
    let secret_key = prompt_secret("Local secret key (from recovery kit)", true)?;
    let secret_key =
        LocalSecretKey::from_hex(&secret_key).map_err(crate::errors::CliError::InvalidArgument)?;

    vault
        .unlock(&password, &secret_key)
        .map_err(map_core_error)?;

    // Store local secret key in OS keychain.
    if let Some(store) = default_secret_store() {
        if let Err(e) = store.store_local_secret_key(&vault_id, &secret_key) {
            eprintln!("Warning: could not store secret key in OS keychain: {e}");
            eprintln!("The vault is unlocked, but you will need to provide the secret key again next time.");
        }
    } else {
        eprintln!("Warning: OS keychain not available. Secret key will not be remembered.");
    }

    context.save_session(&SessionState::unlocked(vault_id))?;
    println!("Vault unlocked: {vault_id}");
    Ok(())
}
