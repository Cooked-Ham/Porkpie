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

/// Start the Porkpie SSH agent (foreground).
///
/// On Unix: binds a Unix domain socket at the default or env-specified path.
/// On Windows: binds the named pipe `\\.\pipe\openssh-ssh-agent` after
/// checking that the built-in Windows OpenSSH Authentication Agent service is
/// not running.
pub async fn run_agent_start(context: &CommandContext) -> Result<()> {
    let vault = unlock_current_vault(context).await?;
    run_agent_with_unlocked_vault(&vault).await
}

/// Run the agent with an already-unlocked vault (used by tests and internal
/// callers that have already authenticated).
pub async fn run_agent_with_unlocked_vault(vault: &porkpie_core::Vault) -> Result<()> {
    let items = vault.list_items().map_err(crate::errors::map_core_error)?;

    let mut ssh_keys_found = false;
    let mut loaded_identities = 0usize;
    let mut agent = porkpie_agent::Agent::new();

    for item in items {
        if let ItemType::SSHKey(secret) = &item.data {
            ssh_keys_found = true;

            let signer_result = if secret.private_key.contains("OPENSSH PRIVATE KEY") {
                porkpie_agent::Ed25519Signer::from_openssh(
                    &secret.private_key,
                    secret.passphrase.as_deref(),
                )
            } else if secret.private_key.len() == 64 {
                if let Ok(bytes) = hex::decode(&secret.private_key) {
                    if bytes.len() == 32 {
                        let seed: [u8; 32] = bytes.try_into().map_err(|_| {
                            CliError::InvalidArgument(format!(
                                "SSH key '{}' internal error: length check passed but try_into failed.",
                                secret.name
                            ))
                        })?;
                        Ok(porkpie_agent::Ed25519Signer::from_seed(&seed))
                    } else {
                        Err(porkpie_agent::SignerError::KeyParse(format!(
                            "SSH key '{}' private key is not a valid 32-byte hex seed (decoded to {} bytes).",
                            secret.name, bytes.len()
                        )))
                    }
                } else {
                    Err(porkpie_agent::SignerError::KeyParse(format!(
                        "SSH key '{}' private key is not valid hex.",
                        secret.name
                    )))
                }
            } else {
                match base64::engine::general_purpose::STANDARD.decode(&secret.private_key) {
                    Ok(decoded) => {
                        if decoded.len() == 32 {
                            let seed: [u8; 32] = decoded.try_into().map_err(|_| {
                                CliError::InvalidArgument(format!(
                                    "SSH key '{}' internal error: length check passed but try_into failed.",
                                    secret.name
                                ))
                            })?;
                            Ok(porkpie_agent::Ed25519Signer::from_seed(&seed))
                        } else {
                            Err(porkpie_agent::SignerError::KeyParse(format!(
                                "SSH key '{}' private key decoded to {} bytes (expected 32).",
                                secret.name,
                                decoded.len()
                            )))
                        }
                    }
                    Err(e) => Err(porkpie_agent::SignerError::KeyParse(format!(
                        "SSH key '{}' private key is not valid base64: {e}.",
                        secret.name
                    ))),
                }
            };

            let signer = match signer_result {
                Ok(signer) => signer,
                Err(e) => {
                    eprintln!(
                        "Warning: SSH key '{}' could not be loaded: {}. Skipping.",
                        secret.name, e.0
                    );
                    continue;
                }
            };
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
            loaded_identities += 1;
        }
    }

    if !ssh_keys_found {
        println!("No SSH key items found in the current vault.");
        println!(
            "Add an SSH key item with `porkpie add SSHKey` and then run `porkpie ssh-agent start`."
        );
        return Ok(());
    }

    if loaded_identities == 0 {
        return Err(CliError::InvalidArgument(
            "All SSH key items in the vault failed to load. Check warnings above and fix the keys."
                .to_string(),
        ));
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

    #[cfg(unix)]
    {
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

        if let Err(e) = porkpie_agent::run_unix_socket(agent, &socket_path) {
            return Err(CliError::InvalidArgument(format!("SSH agent failed: {e}")));
        }
    }

    #[cfg(windows)]
    {
        let pipe_name = if let Ok(name) = std::env::var("PORKPIE_SSH_AGENT_SOCK") {
            name
        } else {
            porkpie_agent::windows_pipe::DEFAULT_PIPE_NAME.to_string()
        };

        println!("Porkpie SSH agent will bind the named pipe:");
        println!("  {pipe_name}");
        println!();
        println!("This replaces the Windows OpenSSH Authentication Agent service.");
        println!("If the built-in service is running, disable it first:");
        println!("  Stop-Service ssh-agent");
        println!("  Set-Service ssh-agent -StartupType Disabled");
        println!();
        println!("Then test with: ssh -T git@github.com");
        println!();
        println!("Press Ctrl+C to stop the agent.");

        let agent = std::sync::Arc::new(std::sync::Mutex::new(agent));
        if let Err(e) = porkpie_agent::run_windows_named_pipe(&pipe_name, agent).await {
            return Err(CliError::InvalidArgument(format!("SSH agent failed: {e}")));
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        println!("SSH agent is not supported on this platform.");
    }

    Ok(())
}

/// Print the environment variable or configuration needed to use the agent.
pub async fn run_agent_env(_context: &CommandContext) -> Result<()> {
    #[cfg(unix)]
    {
        let socket_path = if let Ok(path) = std::env::var("PORKPIE_SSH_AGENT_SOCK") {
            std::path::PathBuf::from(path)
        } else {
            std::env::temp_dir().join("porkpie-ssh-agent.sock")
        };
        println!("export SSH_AUTH_SOCK={}", socket_path.display());
    }

    #[cfg(windows)]
    {
        println!("# No environment variable required.");
        println!("# Microsoft OpenSSH uses the default named pipe:");
        println!("#   {}", porkpie_agent::windows_pipe::DEFAULT_PIPE_NAME);
        println!("#");
        println!("# If the built-in Windows OpenSSH Authentication Agent service is running,");
        println!("# disable it first:");
        println!("#   Stop-Service ssh-agent");
        println!("#   Set-Service ssh-agent -StartupType Disabled");
        println!("#");
        println!("# Git for Windows can be forced to use Microsoft OpenSSH:");
        println!("#   git config --global core.sshCommand \"C:/Windows/System32/OpenSSH/ssh.exe\"");
    }

    #[cfg(not(any(unix, windows)))]
    {
        println!("SSH agent is not supported on this platform.");
    }

    Ok(())
}

/// Check whether the agent appears to be running.
pub async fn run_agent_status(_context: &CommandContext) -> Result<()> {
    #[cfg(unix)]
    {
        let socket_path = if let Ok(path) = std::env::var("PORKPIE_SSH_AGENT_SOCK") {
            std::path::PathBuf::from(path)
        } else {
            std::env::temp_dir().join("porkpie-ssh-agent.sock")
        };
        if socket_path.exists() {
            println!("Agent socket exists: {}", socket_path.display());
            println!("Try: ssh-add -L");
        } else {
            println!("Agent socket not found: {}", socket_path.display());
            println!("Run: porkpie ssh-agent start");
        }
    }

    #[cfg(windows)]
    {
        let pipe_name = porkpie_agent::windows_pipe::DEFAULT_PIPE_NAME;
        // Probe: attempt to connect to the pipe as a client.
        // If a server is listening, the open succeeds; otherwise it fails.
        let listening = {
            use tokio::net::windows::named_pipe::ClientOptions;
            ClientOptions::new().open(pipe_name).is_ok()
        };
        if listening {
            println!("Agent pipe is active (connect probe succeeded): {pipe_name}");
            println!("Try: ssh-add -L");
        } else {
            println!("Agent pipe not found (connect probe failed): {pipe_name}");
            println!("Run: porkpie ssh-agent start");
        }

        match porkpie_agent::is_windows_ssh_agent_service_running() {
            Ok(true) => {
                println!("WARNING: The Windows OpenSSH Authentication Agent service is running.");
                println!("It will conflict with Porkpie.  Disable it:");
                println!("  Stop-Service ssh-agent");
                println!("  Set-Service ssh-agent -StartupType Disabled");
            }
            Ok(false) => {
                println!("Windows OpenSSH Authentication Agent service is not running.");
            }
            Err(e) => {
                println!("Could not check Windows ssh-agent service status: {e}");
            }
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        println!("SSH agent is not supported on this platform.");
    }

    Ok(())
}
