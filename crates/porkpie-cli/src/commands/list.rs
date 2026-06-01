use crate::commands::{unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use crate::interactive::{item_title, item_type_name};
use porkpie_store::load_items;

/// List decrypted item metadata for the current vault.
pub async fn run(context: &CommandContext) -> Result<()> {
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let rows = load_items(&pool, &vault.id)
        .await
        .map_err(map_store_error)?;

    if rows.is_empty() {
        println!("No items");
        return Ok(());
    }

    println!("{:<36}  {:<16}  {:<24}  Updated", "ID", "Type", "Title");
    for (id, ciphertext) in rows {
        let item = vault.decrypt_item(&ciphertext)?;
        println!(
            "{:<36}  {:<16}  {:<24}  {}",
            id,
            item_type_name(&item.data),
            item_title(&item.data),
            item.updated_at.to_millis()
        );
    }

    Ok(())
}
