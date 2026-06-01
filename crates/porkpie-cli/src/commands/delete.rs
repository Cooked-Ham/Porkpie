use crate::commands::{parse_item_id, unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use crate::interactive::confirm_delete;
use porkpie_store::delete_item;

/// Delete an encrypted item from the current vault.
pub async fn run(context: &CommandContext, id: &str) -> Result<()> {
    let session = context.load_session()?;
    session.require_unlocked_vault()?;

    if !confirm_delete(id)? {
        println!("Delete cancelled");
        return Ok(());
    }

    let item_id = parse_item_id(id)?;
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let vault_id = vault.id;
    delete_item(&pool, &vault_id, &item_id)
        .await
        .map_err(map_store_error)?;
    println!("Item deleted: {item_id}");
    Ok(())
}
