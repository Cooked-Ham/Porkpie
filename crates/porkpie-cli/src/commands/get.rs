use crate::commands::{parse_item_id, unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use porkpie_store::load_item;

/// Print one decrypted item as JSON.
pub async fn run(context: &CommandContext, id: &str) -> Result<()> {
    let item_id = parse_item_id(id)?;
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let ciphertext = load_item(&pool, &item_id).await.map_err(map_store_error)?;
    let item = vault.decrypt_item(&ciphertext)?;

    println!("{}", serde_json::to_string_pretty(&item)?);
    Ok(())
}
