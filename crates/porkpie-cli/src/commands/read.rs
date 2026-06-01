use crate::commands::{find_item_by_name, parse_pie_uri, unlock_vault_by_name, CommandContext};
use crate::errors::{map_store_error, CliError, Result};
use porkpie_store::{load_item, load_item_record};

pub async fn run(context: &CommandContext, uri_str: &str) -> Result<()> {
    let uri = parse_pie_uri(uri_str)?;
    let vault = unlock_vault_by_name(context, &uri.vault_name).await?;
    let pool = context.pool().await?;
    let vault_id = vault.id;

    let (item_id, _item) = find_item_by_name(&vault, &uri.item_name)?;
    let record = load_item_record(&pool, &vault_id, &item_id)
        .await
        .map_err(map_store_error)?;

    let ciphertext = load_item(&pool, &vault_id, &item_id)
        .await
        .map_err(map_store_error)?;
    let decrypted = vault.decrypt_item(&ciphertext, &item_id, &record.item_type)?;

    let value = decrypted
        .data
        .get_field(&uri.field_name)
        .map_err(|e| CliError::FieldError(e.to_string()))?;

    println!("{}", value);
    Ok(())
}
