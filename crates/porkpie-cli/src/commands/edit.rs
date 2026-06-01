use crate::commands::{parse_item_id, unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use crate::interactive::{item_type_name, prompt_updated_item};
use porkpie_store::{load_item, load_item_record, update_item};

pub async fn run(context: &CommandContext, id: &str) -> Result<()> {
    let item_id = parse_item_id(id)?;
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let record = load_item_record(&pool, &item_id)
        .await
        .map_err(map_store_error)?;
    let ciphertext = load_item(&pool, &item_id).await.map_err(map_store_error)?;
    let current_item = vault.decrypt_item(&ciphertext, &item_id, &record.item_type)?;
    let mut replacement = prompt_updated_item(&current_item)?;

    replacement.id = item_id;
    replacement.created_at = current_item.created_at;
    replacement.updated_at = porkpie_types::Timestamp::now();

    let updated_ciphertext = vault.encrypt_item(&replacement)?;
    update_item(&pool, &item_id, &updated_ciphertext)
        .await
        .map_err(map_store_error)?;

    let _record = load_item_record(&pool, &item_id)
        .await
        .map_err(map_store_error)?;
    println!(
        "Item updated: {item_id} ({})",
        item_type_name(&replacement.data)
    );
    Ok(())
}
