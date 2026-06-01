//! Command-line interface support for Porkpie.
//!
//! Security design:
//! - Secret input uses hidden prompts via `dialoguer::Password`.
//! - CLI arguments that accept plaintext secrets must carry `--stdin` or `--prompt` alternatives.
//! - `read` outputs to stdout by design; use `copy` for safer workflows.
//! - Session state does not store the local secret key; it is stored in the OS keychain.

pub mod commands;
pub mod errors;
pub mod interactive;
pub mod secret_store;
pub mod session;

use clap::{Parser, Subcommand};
use errors::{CliError, Result};
use porkpie_sync::MergeStrategy;

/// Porkpie command-line arguments.
#[derive(Debug, Parser)]
#[command(name = "porkpie")]
#[command(version = "0.1.0")]
#[command(about = "Local-first password manager")]
pub struct Cli {
    /// SQLite database URL. Defaults to PORKPIE_DATABASE_URL or sqlite:porkpie.db.
    #[arg(long, global = true)]
    pub database_url: Option<String>,

    /// Session state file path. Defaults to PORKPIE_SESSION_PATH or .porkpie-session.json.
    #[arg(long, global = true)]
    pub session_path: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Porkpie subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize a new vault.
    Init,
    /// Unlock a vault and remember it as the current session vault.
    Unlock,
    /// Lock the current session.
    Lock,
    /// Item management commands.
    #[command(subcommand)]
    Item(ItemCommands),
    /// Read a single field value from a pie:// URI.
    Read {
        uri: String,
        /// Omit trailing newline (useful for scripts).
        #[arg(long)]
        no_newline: bool,
        /// Suppress TTY warning.
        #[arg(long)]
        quiet: bool,
    },
    /// Write a value to a single field via pie:// URI.
    /// WARNING: passing the value as a CLI argument exposes it to shell history and process lists.
    /// Use --stdin or --prompt for safer secret entry.
    Write {
        uri: String,
        /// Value to write. Omit if using --stdin or --prompt.
        value: Option<String>,
        /// Read value from stdin (hidden, no echo).
        #[arg(long, conflicts_with_all = ["value", "prompt"])]
        stdin: bool,
        /// Prompt for value interactively (hidden, no echo).
        #[arg(long, conflicts_with_all = ["value", "stdin"])]
        prompt: bool,
    },
    /// Copy a field value to clipboard via pie:// URI.
    Copy {
        uri: String,
        /// Clear clipboard after N seconds (0 disables).
        #[arg(long, value_name = "SECONDS")]
        clear_after: Option<u64>,
    },
    /// Run a command with secrets injected as environment variables.
    Run {
        /// Environment variable mappings in the form NAME=pie://vault/item/field
        #[arg(long = "env", value_name = "NAME=PIE_URI", num_args = 1..)]
        env: Vec<String>,
        /// The command to run
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Add a new item.
    Add { item_type: String },
    /// Edit an existing item.
    Edit { id: String },
    /// Delete an item.
    Delete { id: String },
    /// Vault management commands.
    #[command(subcommand)]
    Vault(VaultCommands),
    /// Recovery management commands.
    #[command(subcommand)]
    Recovery(RecoveryCommands),
    /// Backup management commands.
    #[command(subcommand)]
    Backup(BackupCommands),
    /// Export vault data (encrypted by default, plaintext with --dangerous).
    Export {
        /// Export format: encrypted (default) or plaintext.
        #[arg(long, default_value = "encrypted")]
        format: String,
        /// Required to export plaintext. Acknowledges the danger.
        #[arg(long)]
        dangerous: bool,
        /// Optional output path.
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    /// Import an encrypted backup or CSV file.
    Import { file: std::path::PathBuf },
    /// Sync with a remote server.
    Sync {
        /// Sync API base URL. Defaults to PORKPIE_SYNC_URL or http://127.0.0.1:8000.
        #[arg(long)]
        server: Option<String>,
        /// Bearer API key. Defaults to PORKPIE_API_KEY.
        #[arg(long)]
        api_key: Option<String>,
        /// Merge strategy: preserve-conflict, last-write-wins, prefer-local, prefer-remote. Defaults to preserve-conflict.
        #[arg(long, default_value = "preserve-conflict")]
        strategy: String,
    },
    /// SSH key management commands.
    #[command(subcommand)]
    Ssh(SshCommands),
    /// Keychain management commands.
    #[command(subcommand)]
    Keychain(KeychainCommands),
    /// Start the SSH agent.
    SshAgent,
}

/// Item subcommands.
#[derive(Debug, Subcommand)]
pub enum ItemCommands {
    /// List items in the current vault (redacted).
    List,
    /// Get item details (redacted by default).
    Get { id: String },
}

/// Vault subcommands.
#[derive(Debug, Subcommand)]
pub enum VaultCommands {
    /// Change the master password (rewraps vault key, does not re-encrypt items).
    ChangePassword,
    /// Rotate the local secret key (generates new key, new recovery kit).
    RotateLocalSecret,
    /// Rotate the vault key (re-encrypts all items). Requires backup first.
    RotateKey {
        /// Skip automatic backup before rotation.
        #[arg(long)]
        skip_backup: bool,
    },
    /// Calibrate Argon2id parameters to a target unlock time.
    CalibrateKdf {
        /// Target unlock time in milliseconds.
        #[arg(long, default_value = "750")]
        target_ms: u64,
    },
    /// Upgrade KDF profile (e.g., standard to hardened).
    UpgradeKdf {
        /// Profile name: low-memory, standard, hardened, paranoid.
        #[arg(long)]
        profile: String,
    },
}

/// Recovery subcommands.
#[derive(Debug, Subcommand)]
pub enum RecoveryCommands {
    /// Verify a recovery kit structure without printing secrets.
    Verify {
        /// Path to recovery kit JSON.
        #[arg(long)]
        kit: std::path::PathBuf,
    },
    /// Restore a vault from a recovery kit and encrypted backup.
    Restore {
        /// Path to recovery kit JSON.
        #[arg(long)]
        kit: std::path::PathBuf,
        /// Path to encrypted backup.
        #[arg(long)]
        backup: std::path::PathBuf,
    },
}

/// SSH subcommands.
#[derive(Debug, Subcommand)]
pub enum SshCommands {
    /// Display the public key for an SSH key item.
    PublicKey { target: String },
}

/// Backup subcommands.
#[derive(Debug, Subcommand)]
pub enum BackupCommands {
    /// Export an encrypted backup.
    Export {
        /// Optional output path for the encrypted backup JSON.
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    /// Import an encrypted backup.
    Import { file: std::path::PathBuf },
}

/// Keychain subcommands.
#[derive(Debug, Subcommand)]
pub enum KeychainCommands {
    /// Show keychain status (available, backend, vault count).
    Status,
    /// Test keychain storage by writing and reading a test secret.
    Test,
    /// Forget (delete) the local secret key for a vault.
    Forget { vault: String },
}

/// Run the parsed CLI command.
pub async fn run(cli: Cli) -> Result<()> {
    let context = commands::CommandContext::new(cli.database_url, cli.session_path);

    match cli.command {
        Commands::Init => commands::init::run(&context).await,
        Commands::Unlock => commands::unlock::run(&context).await,
        Commands::Lock => commands::lock::run(&context).await,
        Commands::Item(ItemCommands::List) => commands::list::run(&context).await,
        Commands::Item(ItemCommands::Get { id }) => commands::get::run(&context, &id).await,
        Commands::Read {
            uri,
            no_newline,
            quiet,
        } => commands::read::run(&context, &uri, no_newline, quiet).await,
        Commands::Write {
            uri,
            value,
            stdin,
            prompt,
        } => {
            let value = if let Some(v) = value {
                if stdin || prompt {
                    return Err(CliError::InvalidArgument(
                        "cannot specify a value with --stdin or --prompt".to_string(),
                    ));
                }
                v
            } else if stdin {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
                    .map_err(|e| CliError::InvalidArgument(format!("failed to read stdin: {e}")))?;
                buf.trim_end().to_string()
            } else if prompt {
                crate::interactive::prompt_secret("Value", false)?
            } else {
                return Err(CliError::InvalidArgument(
                    "value required unless --stdin or --prompt is used".to_string(),
                ));
            };
            commands::write::run(&context, &uri, &value).await
        }
        Commands::Copy { uri, clear_after } => {
            commands::copy::run(&context, &uri, clear_after).await
        }
        Commands::Run { env, command } => commands::run_cmd::run(&context, env, command).await,
        Commands::Add { item_type } => commands::add::run(&context, &item_type).await,
        Commands::Edit { id } => commands::edit::run(&context, &id).await,
        Commands::Delete { id } => commands::delete::run(&context, &id).await,
        Commands::Vault(VaultCommands::ChangePassword) => {
            commands::vault_cmd::change_password(&context).await
        }
        Commands::Vault(VaultCommands::RotateLocalSecret) => {
            commands::vault_cmd::rotate_local_secret(&context).await
        }
        Commands::Vault(VaultCommands::RotateKey { skip_backup }) => {
            commands::vault_cmd::rotate_key(&context, skip_backup).await
        }
        Commands::Vault(VaultCommands::CalibrateKdf { target_ms }) => {
            commands::vault_cmd::calibrate_kdf(&context, target_ms).await
        }
        Commands::Vault(VaultCommands::UpgradeKdf { profile }) => {
            commands::vault_cmd::upgrade_kdf(&context, &profile).await
        }
        Commands::Recovery(RecoveryCommands::Verify { kit }) => {
            commands::recovery_cmd::verify(&context, &kit).await
        }
        Commands::Recovery(RecoveryCommands::Restore { kit, backup }) => {
            commands::recovery_cmd::restore(&context, &kit, &backup).await
        }
        Commands::Backup(BackupCommands::Export { output }) => {
            commands::export::run_encrypted(&context, output).await
        }
        Commands::Backup(BackupCommands::Import { file }) => {
            commands::import::run(&context, &file).await
        }
        Commands::Export {
            format,
            dangerous,
            output,
        } => commands::export::run(&context, &format, dangerous, output).await,
        Commands::Import { file } => commands::import::run(&context, &file).await,
        Commands::Sync {
            server,
            api_key,
            strategy,
        } => {
            let strategy = parse_strategy(&strategy)
                .ok_or_else(|| CliError::InvalidArgument(
                    format!("unknown strategy '{strategy}'; use preserve-conflict, last-write-wins, prefer-local, or prefer-remote")
                ))?;
            commands::sync::run(&context, server, api_key, strategy).await
        }
        Commands::Ssh(SshCommands::PublicKey { target }) => {
            commands::ssh::run_public_key(&context, &target).await
        }
        Commands::SshAgent => commands::ssh::run_agent(&context).await,
        Commands::Keychain(KeychainCommands::Status) => {
            commands::keychain::run_status(&context).await
        }
        Commands::Keychain(KeychainCommands::Test) => commands::keychain::run_test(&context).await,
        Commands::Keychain(KeychainCommands::Forget { vault }) => {
            commands::keychain::run_forget(&context, &vault).await
        }
    }
}

fn parse_strategy(value: &str) -> Option<MergeStrategy> {
    match value.to_ascii_lowercase().as_str() {
        "preserve-conflict" | "preserveconflict" => Some(MergeStrategy::PreserveConflict),
        "last-write-wins" | "lastwritewins" => Some(MergeStrategy::LastWriteWins),
        "prefer-local" | "preferlocal" => Some(MergeStrategy::PreferLocal),
        "prefer-remote" | "preferremote" => Some(MergeStrategy::PreferRemote),
        _ => None,
    }
}
