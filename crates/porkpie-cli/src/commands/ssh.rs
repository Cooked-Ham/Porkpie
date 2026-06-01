//! SSH command implementations for the Porkpie CLI.

use crate::commands::{find_item_by_name, unlock_current_vault, CommandContext};
use crate::errors::{CliError, Result};
use base64::Engine;
use porkpie_types::ItemType;

/// Display the public key for an SSH key item.
///
/// `target` may be a `pie://` URI pointing to the `public_key` field, or an
/// item name.  Only the public key is printed; the private key is never
/// revealed by this command.
pub async fn run_public_key(context: &CommandContext, target: &str) -> Result<()> {
    let pie_uri = if target.starts_with("pie://") {
        Some(crate::commands::parse_pie_uri(target)?)
    } else {
        None
    };

    let vault = unlock_current_vault(context).await?;

    let public_key = if let Some(uri) = pie_uri {
        // pie://vault/item/public_key
        let vault_name = vault.name.clone();
        if uri.vault_name != vault_name {
            return Err(CliError::InvalidArgument(format!(
                "URI vault '{}' does not match unlocked vault '{}'",
                uri.vault_name, vault_name
            )));
        }
        let item = find_item_by_name(&vault, &uri.item_name)?;
        item.1.data.get_field(&uri.field_name).map_err(|e| {
            CliError::FieldError(format!(
                "field '{}' on item '{}': {}",
                uri.field_name, uri.item_name, e
            ))
        })?
    } else {
        // Treat target as an item name and extract the public_key field.
        let item = find_item_by_name(&vault, target)?;
        item.1
            .data
            .get_field("public_key")
            .map_err(|e| CliError::FieldError(format!("item '{target}': {e}")))?
    };

    println!("{public_key}");
    Ok(())
}

/// Start the Porkpie SSH agent.
///
/// Loads SSH keys from the unlocked vault, starts a Unix domain socket
/// listener, and prints `SSH_AUTH_SOCK` for the user to export.
pub async fn run_agent(context: &CommandContext) -> Result<()> {
    let vault = unlock_current_vault(context).await?;

    let items = vault.list_items().map_err(crate::errors::map_core_error)?;

    let mut ssh_keys_found = false;
    let mut agent = porkpie_agent::Agent::new();

    for item in items {
        if let ItemType::SSHKey(secret) = &item.data {
            ssh_keys_found = true;

            // Parse the private key. We support two formats:
            // 1. Raw 64-char hex string (32 bytes) -> direct Ed25519 seed
            // 2. Base64-encoded raw key
            let private_key_bytes = if secret.private_key.len() == 64 {
                // Try hex first
                if let Ok(bytes) = hex::decode(&secret.private_key) {
                    if bytes.len() == 32 {
                        bytes
                    } else {
                        eprintln!(
                            "Warning: SSH key '{}' private key is not a valid 32-byte hex seed (decoded to {} bytes). Skipping.",
                            secret.name, bytes.len()
                        );
                        continue;
                    }
                } else {
                    eprintln!(
                        "Warning: SSH key '{}' private key is not valid hex. Skipping.",
                        secret.name
                    );
                    continue;
                }
            } else {
                // Try base64
                let decoded =
                    match base64::engine::general_purpose::STANDARD.decode(&secret.private_key) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            eprintln!(
                            "Warning: SSH key '{}' private key is not valid base64: {e}. Skipping.",
                            secret.name
                        );
                            continue;
                        }
                    };
                if decoded.len() == 32 {
                    decoded
                } else {
                    eprintln!(
                        "Warning: SSH key '{}' private key decoded to {} bytes (expected 32). Skipping.",
                        secret.name, decoded.len()
                    );
                    continue;
                }
            };

            let seed: [u8; 32] = match private_key_bytes.try_into() {
                Ok(arr) => arr,
                Err(_) => {
                    eprintln!(
                        "Warning: SSH key '{}' private key is not 32 bytes. Skipping.",
                        secret.name
                    );
                    continue;
                }
            };

            let signer = porkpie_agent::Ed25519Signer::from_seed(&seed);
            let comment = secret
                .comment
                .clone()
                .unwrap_or_else(|| secret.name.clone());

            agent.add_identity(porkpie_agent::AgentIdentity {
                comment,
                signer: Box::new(signer),
                allowed_hosts: secret.allowed_hosts.clone(),
                require_confirmation: secret.require_confirmation,
            });
        }
    }

    if !ssh_keys_found {
        println!("No SSH key items found in the current vault.");
        println!("Add an SSH key item with `porkpie add SSHKey` and then run `porkpie ssh-agent`.");
        return Ok(());
    }

    // Set up approval callback for confirmation-required keys
    agent.set_approval_callback(Box::new(|comment, _preview| {
        println!("SSH agent signing request for key: {comment}");
        print!("Approve? [y/N] ");
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => input.trim().eq_ignore_ascii_case("y"),
            Err(_) => false,
        }
    }));

    // Choose a socket path
    let socket_path = if let Ok(path) = std::env::var("PORKPIE_SSH_AGENT_SOCK") {
        std::path::PathBuf::from(path)
    } else {
        std::env::temp_dir().join("porkpie-ssh-agent.sock")
    };

    println!("\nExport the following environment variable to use the agent:");
    println!("  export SSH_AUTH_SOCK={}", socket_path.display());
    println!();
    println!("Then test with: ssh -T git@github.com");
    println!();
    println!("Press Ctrl+C to stop the agent.");

    // Start the Unix socket listener
    #[cfg(unix)]
    {
        if let Err(e) = porkpie_agent::run_unix_socket(agent, &socket_path) {
            return Err(CliError::InvalidArgument(format!("SSH agent failed: {e}")));
        }
    }

    #[cfg(not(unix))]
    {
        println!("SSH agent is not supported on this platform.");
    }

    Ok(())
}
