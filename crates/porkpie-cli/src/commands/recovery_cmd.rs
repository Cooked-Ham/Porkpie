use crate::commands::CommandContext;
use crate::errors::{CliError, Result};
use porkpie_import::{import_backup, read_backup_file, BackupImportMode, BackupImportResult};
use porkpie_store::{load_items, store_item, store_vault};
use porkpie_types::LocalSecretKey;
use std::collections::HashSet;
use std::path::Path;

/// Verify a recovery kit structure without printing secrets.
pub async fn verify(_context: &CommandContext, kit_path: &Path) -> Result<()> {
    let contents = std::fs::read_to_string(kit_path).map_err(CliError::Io)?;
    let kit: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| CliError::InvalidArgument(format!("invalid JSON: {e}")))?;

    // Check required fields.
    if kit.get("vault_id").is_none() {
        return Err(CliError::InvalidArgument("missing vault_id".to_string()));
    }
    if kit.get("local_secret_key").is_none() {
        return Err(CliError::InvalidArgument(
            "missing local_secret_key".to_string(),
        ));
    }
    if kit.get("created_at").is_none() {
        return Err(CliError::InvalidArgument("missing created_at".to_string()));
    }

    // Verify the local secret key is valid hex.
    let hex = kit["local_secret_key"]
        .as_str()
        .ok_or_else(|| CliError::InvalidArgument("local_secret_key is not a string".to_string()))?;
    let _ = LocalSecretKey::from_hex(hex)
        .map_err(|e| CliError::InvalidArgument(format!("invalid local secret key: {e}")))?;

    println!("Recovery kit structure is valid.");
    println!("Vault ID: {}", kit["vault_id"].as_str().unwrap_or(""));
    println!("Created at: {}", kit["created_at"].as_i64().unwrap_or(0));
    println!(
        "Instructions: {} lines",
        kit["instructions"].as_array().map(|a| a.len()).unwrap_or(0)
    );
    println!("WARNING: Do not print or share the local secret key.");
    Ok(())
}

/// Restore a vault from a recovery kit and encrypted backup.
///
/// Reads the recovery kit for the vault_id and local_secret_key, then reads
/// the encrypted backup file, prompts for the master password, decrypts the
/// backup, and stores the vault + items in the local database.
pub async fn restore(context: &CommandContext, kit_path: &Path, backup_path: &Path) -> Result<()> {
    // Read the recovery kit.
    let kit_contents = std::fs::read_to_string(kit_path).map_err(CliError::Io)?;
    let kit: serde_json::Value = serde_json::from_str(&kit_contents)
        .map_err(|e| CliError::InvalidArgument(format!("invalid recovery kit JSON: {e}")))?;

    let vault_id = kit["vault_id"]
        .as_str()
        .ok_or_else(|| CliError::InvalidArgument("missing vault_id in recovery kit".to_string()))?;
    let secret_key_hex = kit["local_secret_key"].as_str().ok_or_else(|| {
        CliError::InvalidArgument("missing local_secret_key in recovery kit".to_string())
    })?;
    let secret_key = LocalSecretKey::from_hex(secret_key_hex).map_err(|e| {
        CliError::InvalidArgument(format!("invalid local_secret_key in recovery kit: {e}"))
    })?;

    // Read the encrypted backup.
    let backup = read_backup_file(backup_path)
        .map_err(|e| CliError::InvalidArgument(format!("failed to read backup: {e}")))?;

    // Verify the backup vault_id matches the recovery kit.
    let backup_vault_id = backup.vault.id.to_string();
    if backup_vault_id != vault_id {
        return Err(CliError::InvalidArgument(format!(
            "backup vault_id ({backup_vault_id}) does not match recovery kit vault_id ({vault_id})"
        )));
    }

    // Prompt for the master password.
    println!(
        "Restoring vault {}. Enter the master password for this backup:",
        vault_id
    );
    let password = crate::interactive::prompt_master_password()?;

    // Get existing item IDs from the database (empty set if vault doesn't exist yet).
    let pool = context.pool().await?;
    let existing_item_ids = match load_items(&pool, &backup.vault.id).await {
        Ok(items) => items
            .into_iter()
            .map(|(item_id, _)| item_id.to_string())
            .collect(),
        Err(porkpie_store::StoreError::VaultNotFound(_)) => HashSet::new(),
        Err(error) => return Err(CliError::Store(error)),
    };

    // Decrypt and import the backup.
    let BackupImportResult {
        vault,
        items,
        imported,
        skipped,
    } = import_backup(
        backup,
        &password,
        &secret_key,
        &existing_item_ids,
        BackupImportMode::SkipDuplicates,
    )
    .map_err(|e| CliError::InvalidArgument(format!("failed to decrypt backup: {e}")))?;

    let vault_id = vault.id;
    let locked_vault = vault.into_locked_vault();

    // Store the vault and items.
    store_vault(&pool, &locked_vault)
        .await
        .map_err(CliError::Store)?;
    for item in &items {
        store_item(&pool, item).await.map_err(CliError::Store)?;
    }

    // Store the secret key in the keychain.
    if let Some(store) = crate::secret_store::default_secret_store() {
        if let Err(e) = store.store_local_secret_key(&vault_id, &secret_key) {
            eprintln!("Warning: could not store secret key in OS keychain: {e}");
            eprintln!("You will need to provide the secret key manually when unlocking.");
        }
    } else {
        eprintln!("Warning: OS keychain not available. Secret key will not be remembered.");
    }

    context.save_session(&crate::session::SessionState::unlocked(vault_id))?;

    println!("Vault {} restored successfully.", vault_id);
    println!("  Imported: {} items", imported);
    println!("  Skipped: {} duplicates", skipped);
    println!("  Secret key: stored in OS keychain (or manual entry required)");
    println!("Next: porkpie unlock");
    Ok(())
}
