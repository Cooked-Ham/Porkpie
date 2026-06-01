use crate::commands::{unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use crate::interactive::{prompt_item, select_item_type};
use porkpie_store::{store_item, EncryptedItemData};

/// Add a new encrypted item to the current vault.
pub async fn run(context: &CommandContext, item_type: &str) -> Result<()> {
    let mut vault = unlock_current_vault(context).await?;
    let selected_type = if item_type.eq_ignore_ascii_case("select") {
        select_item_type()?
    } else {
        item_type
    };
    let item = prompt_item(selected_type)?;
    let item_id = vault.create_item(item)?;
    let stored_item = vault.get_item(item_id)?.clone();
    let ciphertext = vault.encrypt_item(&stored_item)?;
    let encrypted = EncryptedItemData::new(
        item_id,
        vault.id,
        crate::interactive::item_type_name(&stored_item.data),
        ciphertext,
        stored_item.created_at,
        stored_item.updated_at,
        vault.sync_revision(),
    );

    let pool = context.pool().await?;
    store_item(&pool, &encrypted)
        .await
        .map_err(map_store_error)?;

    println!("Item created: {item_id}");
    Ok(())
}
