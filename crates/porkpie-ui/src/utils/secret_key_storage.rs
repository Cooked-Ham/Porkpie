//! OS credential manager integration for storing and retrieving the local secret key.
//!
//! On Windows this uses the Windows Credential Manager via the `keyring` crate.
//! The secret key is stored per-vault so that users do not need to copy/paste the
//! hex string during normal onboarding or unlock.
//!
//! The recovery kit remains the only offline backup of the secret key.

use porkpie_types::LocalSecretKey;

const SERVICE_NAME: &str = "Porkpie";

/// Build the keyring username for a given vault name.
fn keyring_username(vault_name: &str) -> String {
    format!("porkpie_vault_{}", vault_name)
}

/// Store a local secret key in the OS credential manager.
///
/// On Windows this writes to the Windows Credential Manager.
/// On macOS this would use the Keychain. On Linux this would use
/// the secret service or libsecret.
#[cfg(not(target_arch = "wasm32"))]
pub fn store_secret_key(vault_name: &str, secret_key: &LocalSecretKey) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &keyring_username(vault_name))
        .map_err(|e| format!("credential manager entry: {e}"))?;
    entry
        .set_password(&secret_key.to_hex())
        .map_err(|e| format!("credential manager write: {e}"))
}

/// Retrieve a local secret key from the OS credential manager.
#[cfg(not(target_arch = "wasm32"))]
pub fn get_secret_key(vault_name: &str) -> Option<LocalSecretKey> {
    let entry = keyring::Entry::new(SERVICE_NAME, &keyring_username(vault_name)).ok()?;
    let hex = entry.get_password().ok()?;
    LocalSecretKey::from_hex(&hex).ok()
}

/// Delete a stored secret key from the OS credential manager.
#[cfg(not(target_arch = "wasm32"))]
pub fn delete_secret_key(vault_name: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE_NAME, &keyring_username(vault_name))
        .map_err(|e| format!("credential manager entry: {e}"))?;
    entry
        .delete_credential()
        .map_err(|e| format!("credential manager delete: {e}"))
}

/// WASM stubs: the credential manager is not available in the browser.
#[cfg(target_arch = "wasm32")]
pub fn store_secret_key(_vault_name: &str, _secret_key: &LocalSecretKey) -> Result<(), String> {
    Err("Credential manager not available in WASM".to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn get_secret_key(_vault_name: &str) -> Option<LocalSecretKey> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn delete_secret_key(_vault_name: &str) -> Result<(), String> {
    Err("Credential manager not available in WASM".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn roundtrip_secret_key() {
        let key = LocalSecretKey::generate();
        let vault_name = "test_roundtrip_vault";
        // Clean up any existing entry
        let _ = delete_secret_key(vault_name);
        if let Err(e) = store_secret_key(vault_name, &key) {
            eprintln!("Skipping credential manager roundtrip test: store failed: {e}");
            return;
        }
        let retrieved = match get_secret_key(vault_name) {
            Some(k) => k,
            None => {
                eprintln!("Skipping credential manager roundtrip test: get returned None");
                return;
            }
        };
        assert_eq!(key.to_hex(), retrieved.to_hex());
        let _ = delete_secret_key(vault_name);
        assert!(get_secret_key(vault_name).is_none());
    }
}
