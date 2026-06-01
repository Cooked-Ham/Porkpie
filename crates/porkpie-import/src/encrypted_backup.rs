use crate::errors::{ImportError, Result};
use porkpie_core::Vault;
use porkpie_store::{EncryptedItemData, EncryptedVaultData};
use porkpie_types::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

const BACKUP_VERSION: u32 = 1;

/// Encrypted Porkpie backup file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupFile {
    pub version: u32,
    pub vault: EncryptedVaultData,
    pub timestamp: i64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BackupPayload {
    items: Vec<EncryptedItemData>,
}

/// Duplicate handling behavior when importing a backup into an existing vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupImportMode {
    SkipDuplicates,
    OverwriteDuplicates,
}

/// Result of validating and merging a backup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupImportResult {
    pub vault: EncryptedVaultData,
    pub items: Vec<EncryptedItemData>,
    pub imported: usize,
    pub skipped: usize,
}

/// Build an encrypted backup payload from encrypted store rows.
pub fn export_backup_file(
    unlocked_vault: &Vault,
    vault: EncryptedVaultData,
    items: Vec<EncryptedItemData>,
) -> Result<BackupFile> {
    let payload = BackupPayload { items };
    Ok(BackupFile {
        version: BACKUP_VERSION,
        vault,
        timestamp: Timestamp::now().to_millis(),
        payload: unlocked_vault.encrypt_payload(&payload)?,
    })
}

/// Write an encrypted backup payload to disk as JSON.
pub fn write_backup_file(path: &Path, backup: &BackupFile) -> Result<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, backup)?;
    Ok(())
}

/// Read an encrypted backup payload from disk.
pub fn read_backup_file(path: &Path) -> Result<BackupFile> {
    let file = File::open(path)?;
    let backup: BackupFile = serde_json::from_reader(file)?;
    validate_backup(&backup)?;
    Ok(backup)
}

/// Load, validate, and decrypt-check a backup before returning encrypted rows to persist.
pub fn import_backup_file(
    path: &Path,
    password: &str,
    existing_item_ids: &HashSet<String>,
    mode: BackupImportMode,
) -> Result<BackupImportResult> {
    let backup = read_backup_file(path)?;
    import_backup(backup, password, existing_item_ids, mode)
}

/// Validate a backup using the master password and merge duplicate item ids.
pub fn import_backup(
    backup: BackupFile,
    password: &str,
    existing_item_ids: &HashSet<String>,
    mode: BackupImportMode,
) -> Result<BackupImportResult> {
    validate_backup(&backup)?;

    let mut vault = backup.vault.clone().into_locked_vault();
    vault.unlock(password)?;
    let payload: BackupPayload = vault.decrypt_payload(&backup.payload)?;

    let mut seen = HashSet::new();
    let mut items = Vec::new();
    let mut skipped = 0usize;

    for item in payload.items {
        let item_id = item.id.to_string();
        let duplicate = existing_item_ids.contains(&item_id) || !seen.insert(item_id);
        if duplicate && mode == BackupImportMode::SkipDuplicates {
            skipped = skipped.saturating_add(1);
            continue;
        }

        let decrypted = vault.decrypt_item(&item.ciphertext)?;
        if decrypted.id != item.id {
            return Err(ImportError::InvalidRow {
                row: items.len().saturating_add(1),
                message: "decrypted item id does not match backup metadata".to_string(),
            });
        }
        items.push(item);
    }

    Ok(BackupImportResult {
        vault: backup.vault,
        imported: items.len(),
        skipped,
        items,
    })
}

/// Return the default encrypted backup filename for a timestamp.
pub fn backup_file_name(timestamp_millis: i64) -> String {
    format!("porkpie-backup-{timestamp_millis}.json.enc")
}

fn validate_backup(backup: &BackupFile) -> Result<()> {
    if backup.version != BACKUP_VERSION {
        return Err(ImportError::InvalidBackupVersion(backup.version));
    }
    Ok(())
}
