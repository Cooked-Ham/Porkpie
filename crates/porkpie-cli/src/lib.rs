//! Command-line interface support for Porkpie.

pub mod commands;
pub mod errors;
pub mod interactive;
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
    Read { uri: String },
    /// Write a value to a single field via pie:// URI.
    /// WARNING: passing the value as a CLI argument exposes it to shell history and process lists.
    /// Use --stdin or --prompt for safer secret entry.
    Write {
        uri: String,
        /// Value to write. Omit if using --stdin or --prompt.
        value: Option<String>,
        /// Read value from stdin (hidden, no echo).
        #[arg(long, conflicts_with = "value")]
        stdin: bool,
        /// Prompt for value interactively (hidden, no echo).
        #[arg(long, conflicts_with = "value")]
        prompt: bool,
    },
    /// Copy a field value to clipboard via pie:// URI.
    Copy { uri: String },
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
    /// Start the SSH agent (not yet integrated with OpenSSH).
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

/// Run the parsed CLI command.
pub async fn run(cli: Cli) -> Result<()> {
    let context = commands::CommandContext::new(cli.database_url, cli.session_path);

    match cli.command {
        Commands::Init => commands::init::run(&context).await,
        Commands::Unlock => commands::unlock::run(&context).await,
        Commands::Lock => commands::lock::run(&context).await,
        Commands::Item(ItemCommands::List) => commands::list::run(&context).await,
        Commands::Item(ItemCommands::Get { id }) => commands::get::run(&context, &id).await,
        Commands::Read { uri } => commands::read::run(&context, &uri).await,
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
        Commands::Copy { uri } => commands::copy::run(&context, &uri).await,
        Commands::Run { env, command } => commands::run_cmd::run(&context, env, command).await,
        Commands::Add { item_type } => commands::add::run(&context, &item_type).await,
        Commands::Edit { id } => commands::edit::run(&context, &id).await,
        Commands::Delete { id } => commands::delete::run(&context, &id).await,
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
        Commands::SshAgent => commands::ssh::run_agent().await,
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
