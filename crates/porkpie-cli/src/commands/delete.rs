use crate::commands::{parse_item_id, CommandContext};
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
    let pool = context.pool().await?;
    delete_item(&pool, &item_id)
        .await
        .map_err(map_store_error)?;
    println!("Item deleted: {item_id}");
    Ok(())
}
