use crate::commands::{parse_item_id, unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use porkpie_store::load_item_record;

pub async fn run(context: &CommandContext, id: &str) -> Result<()> {
    let item_id = parse_item_id(id)?;
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let vault_id = vault.id;
    let record = load_item_record(&pool, &vault_id, &item_id)
        .await
        .map_err(map_store_error)?;

    println!("ID: {}", record.id);
    println!("Type: {}", record.item_type);
    println!("Created: {}", record.created_at.to_millis());
    println!("Updated: {}", record.updated_at.to_millis());
    println!("Fields: [redacted - use `porkpie read pie://...` to reveal]");

    Ok(())
}
