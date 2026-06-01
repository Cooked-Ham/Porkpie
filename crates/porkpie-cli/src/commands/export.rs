use crate::commands::{unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use porkpie_import::{backup_file_name, encrypted_backup::write_backup_file, export_backup_file};
use porkpie_store::{
    load_item_record, load_items, load_vault, EncryptedItemData, EncryptedVaultData,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Encrypted backup payload. Item ciphertext is exported as-is.
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedBackup {
    pub vault: EncryptedVaultData,
    pub items: Vec<EncryptedItemData>,
}

/// Export the current vault metadata and encrypted items.
pub async fn run(context: &CommandContext, output: Option<PathBuf>) -> Result<()> {
    let unlocked_vault = unlock_current_vault(context).await?;
    let vault_id = unlocked_vault.id;
    let pool = context.pool().await?;
    let vault = load_vault(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;
    let item_refs = load_items(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;
    let mut items = Vec::with_capacity(item_refs.len());

    for (item_id, _) in item_refs {
        items.push(
            load_item_record(&pool, &item_id)
                .await
                .map_err(map_store_error)?,
        );
    }

    let backup = export_backup_file(&unlocked_vault, vault, items)?;
    let output_path = output.unwrap_or_else(default_backup_path);
    write_backup_file(&output_path, &backup)?;
    println!("Backup saved to {}", output_path.display());
    Ok(())
}

fn default_backup_path() -> PathBuf {
    PathBuf::from(backup_file_name(
        porkpie_types::Timestamp::now().to_millis(),
    ))
}
