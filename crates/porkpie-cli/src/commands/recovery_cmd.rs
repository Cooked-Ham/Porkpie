use crate::commands::CommandContext;
use crate::errors::{CliError, Result};
use porkpie_types::LocalSecretKey;
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
/// NOT YET IMPLEMENTED. This command is a scaffold and will be completed
/// in a future release.
pub async fn restore(
    _context: &CommandContext,
    _kit_path: &Path,
    _backup_path: &Path,
) -> Result<()> {
    println!("porkpie recovery restore is not implemented yet.");
    println!("To restore a vault manually:");
    println!("  1. porkpie init <new-vault-name>");
    println!("  2. Use the recovery kit's local secret key when unlocking.");
    println!("  3. porkpie import <backup-file>");
    Ok(())
}
