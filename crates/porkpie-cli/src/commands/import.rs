use crate::commands::{unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, Result};
use crate::session::SessionState;
use porkpie_import::{
    import_backup_file, import_csv_reader, BackupImportMode, BackupImportResult, CsvImportResult,
};
use porkpie_store::{delete_item, load_items, store_item, store_vault, update_item};
use std::collections::HashSet;
use std::path::Path;

/// Import a CSV file into the current vault or an encrypted backup into local storage.
pub async fn run(context: &CommandContext, file: &Path) -> Result<()> {
    if is_csv(file) {
        import_csv(context, file).await
    } else {
        import_backup(context, file).await
    }
}

async fn import_csv(context: &CommandContext, file: &Path) -> Result<()> {
    let mut vault = unlock_current_vault(context).await?;
    let file_handle = std::fs::File::open(file)?;
    let CsvImportResult {
        imported,
        encrypted_items,
    } = import_csv_reader(file_handle, &mut vault)?;
    let pool = context.pool().await?;

    for item in &encrypted_items {
        store_item(&pool, item).await.map_err(map_store_error)?;
    }

    println!("Imported {imported} CSV items into vault {}", vault.id);
    Ok(())
}

async fn import_backup(context: &CommandContext, file: &Path) -> Result<()> {
    let password = crate::interactive::prompt_master_password()?;
    let session = context.load_session()?;
    let secret_key = session.require_secret_key()?;
    let pool = context.pool().await?;
    let existing_item_ids = existing_item_ids(&pool, file).await?;
    let BackupImportResult {
        vault,
        items,
        imported,
        skipped,
    } = import_backup_file(
        file,
        &password,
        &secret_key,
        &existing_item_ids,
        BackupImportMode::SkipDuplicates,
    )?;
    let vault_id = vault.id;
    let locked_vault = vault.into_locked_vault();

    store_vault(&pool, &locked_vault)
        .await
        .map_err(map_store_error)?;
    for item in &items {
        if update_item(&pool, &item.id, &item.ciphertext)
            .await
            .is_err()
        {
            let _ = delete_item(&pool, &item.id).await;
            store_item(&pool, item).await.map_err(map_store_error)?;
        }
    }

    context.save_session(&SessionState::unlocked(vault_id))?;
    println!("Imported {imported} items into vault {vault_id} ({skipped} skipped)");
    Ok(())
}

async fn existing_item_ids(pool: &sqlx::SqlitePool, file: &Path) -> Result<HashSet<String>> {
    let backup = porkpie_import::encrypted_backup::read_backup_file(file)?;
    match load_items(pool, &backup.vault.id).await {
        Ok(items) => Ok(items
            .into_iter()
            .map(|(item_id, _)| item_id.to_string())
            .collect()),
        Err(porkpie_store::StoreError::VaultNotFound(_)) => Ok(HashSet::new()),
        Err(error) => Err(map_store_error(error)),
    }
}

fn is_csv(file: &Path) -> bool {
    file.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("csv"))
}
