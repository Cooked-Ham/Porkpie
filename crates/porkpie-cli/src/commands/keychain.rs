//! Keychain management commands for the Porkpie CLI.

use crate::commands::CommandContext;
use crate::errors::{CliError, Result};
use porkpie_types::VaultId;

/// Show keychain status.
pub async fn run_status(_context: &CommandContext) -> Result<()> {
    let store = crate::secret_store::default_secret_store();
    match store {
        Some(backend) => {
            println!("Keychain backend: available");
            #[cfg(target_os = "macos")]
            println!("Platform: macOS Keychain");
            #[cfg(target_os = "windows")]
            println!("Platform: Windows Credential Manager");
            #[cfg(target_os = "linux")]
            println!("Platform: Linux Secret Service");
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
            println!("Platform: unknown");
            // Count vaults with stored keys (best effort)
            println!("Keychain is active.");
            // We don't have a list API, so we can't count without knowing vault IDs.
            let _ = backend;
        }
        None => {
            println!("Keychain backend: NOT AVAILABLE");
            println!("No OS keychain integration found. Secret keys will not be remembered.");
        }
    }
    Ok(())
}

/// Test keychain storage by writing and reading a test secret.
pub async fn run_test(_context: &CommandContext) -> Result<()> {
    let store = crate::secret_store::default_secret_store()
        .ok_or_else(|| CliError::InvalidArgument("No keychain backend available.".to_string()))?;
    let test_vault_id = VaultId::new();
    let test_key = porkpie_types::LocalSecretKey::generate();
    store
        .store_local_secret_key(&test_vault_id, &test_key)
        .map_err(|e| CliError::InvalidArgument(format!("Failed to store test key: {e}")))?;
    let loaded = store
        .load_local_secret_key(&test_vault_id)
        .map_err(|e| CliError::InvalidArgument(format!("Failed to load test key: {e}")))?;
    match loaded {
        Some(key) => {
            if key.to_hex() == test_key.to_hex() {
                println!("Keychain test: PASS");
            } else {
                println!("Keychain test: FAIL — loaded key does not match stored key");
            }
        }
        None => {
            println!("Keychain test: FAIL — key not found after store");
        }
    }
    store
        .delete_local_secret_key(&test_vault_id)
        .map_err(|e| CliError::InvalidArgument(format!("Failed to delete test key: {e}")))?;
    Ok(())
}

/// Forget (delete) the local secret key for a vault.
pub async fn run_forget(_context: &CommandContext, vault: &str) -> Result<()> {
    let store = crate::secret_store::default_secret_store()
        .ok_or_else(|| CliError::InvalidArgument("No keychain backend available.".to_string()))?;
    let vault_id = VaultId::from_string(vault)
        .map_err(|e| CliError::InvalidArgument(format!("Invalid vault ID: {e}")))?;
    store
        .delete_local_secret_key(&vault_id)
        .map_err(|e| CliError::InvalidArgument(format!("Failed to delete key: {e}")))?;
    println!("Forgotten local secret key for vault {vault_id}.");
    Ok(())
}
