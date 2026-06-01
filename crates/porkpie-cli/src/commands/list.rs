use crate::commands::{unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use porkpie_store::load_items_with_type;

pub async fn run(context: &CommandContext) -> Result<()> {
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let rows = load_items_with_type(&pool, &vault.id)
        .await
        .map_err(map_store_error)?;

    if rows.is_empty() {
        println!("No items");
        return Ok(());
    }

    println!("{:<36}  {:<16}  {:<24}  Updated", "ID", "Type", "Title");
    for (id, item_type, _ciphertext) in rows {
        println!(
            "{:<36}  {:<16}  {:<24}  [redacted]",
            id, item_type, "[redacted]"
        );
    }

    Ok(())
}
