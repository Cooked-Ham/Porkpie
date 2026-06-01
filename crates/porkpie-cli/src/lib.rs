//! Command-line interface support for Porkpie.

pub mod commands;
pub mod errors;
pub mod interactive;
pub mod session;

use clap::{Parser, Subcommand};
use errors::Result;

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
    Write { uri: String, value: String },
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
    /// Export an encrypted backup.
    Export {
        /// Optional output path for the encrypted backup JSON.
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    /// Import an encrypted backup.
    Import { file: std::path::PathBuf },
    /// Sync with a remote server.
    Sync {
        /// Sync API base URL. Defaults to PORKPIE_SYNC_URL or http://127.0.0.1:8000.
        #[arg(long)]
        server: Option<String>,
        /// Bearer API key. Defaults to PORKPIE_API_KEY.
        #[arg(long)]
        api_key: Option<String>,
    },
}

/// Item subcommands.
#[derive(Debug, Subcommand)]
pub enum ItemCommands {
    /// List items in the current vault (redacted).
    List,
    /// Get item details (redacted by default).
    Get { id: String },
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
        Commands::Write { uri, value } => commands::write::run(&context, &uri, &value).await,
        Commands::Copy { uri } => commands::copy::run(&context, &uri).await,
        Commands::Run { env, command } => commands::run_cmd::run(&context, env, command).await,
        Commands::Add { item_type } => commands::add::run(&context, &item_type).await,
        Commands::Edit { id } => commands::edit::run(&context, &id).await,
        Commands::Delete { id } => commands::delete::run(&context, &id).await,
        Commands::Export { output } => commands::export::run(&context, output).await,
        Commands::Import { file } => commands::import::run(&context, &file).await,
        Commands::Sync { server, api_key } => commands::sync::run(&context, server, api_key).await,
    }
}
