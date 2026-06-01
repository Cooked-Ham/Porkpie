use clap::{CommandFactory, Parser};
use porkpie_cli::{Cli, Commands, ItemCommands};
use porkpie_types::VaultId;

#[test]
fn help_text_renders() {
    Cli::command().debug_assert();
    let mut command = Cli::command();
    let mut help = Vec::new();
    command.write_long_help(&mut help).expect("help renders");
    let help = String::from_utf8(help).expect("help is utf8");

    assert!(help.contains("Local-first password manager"));
    assert!(help.contains("init"));
    assert!(help.contains("unlock"));
    assert!(help.contains("sync"));
}

#[test]
fn invalid_args_are_caught() {
    let result = Cli::try_parse_from(["porkpie", "not-a-command"]);

    assert!(result.is_err());
}

#[test]
fn binary_reports_version() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_porkpie"))
        .arg("--version")
        .output()
        .expect("run porkpie binary");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("version output is utf8");
    assert!(stdout.contains("porkpie 0.1.0"));
}

#[test]
fn parses_global_options_and_subcommands() {
    let cli = Cli::parse_from([
        "porkpie",
        "--database-url",
        "sqlite::memory:",
        "--session-path",
        "session.json",
        "item",
        "get",
        "550e8400-e29b-41d4-a716-446655440000",
    ]);

    assert_eq!(cli.database_url.as_deref(), Some("sqlite::memory:"));
    assert_eq!(
        cli.session_path.as_deref(),
        Some(std::path::Path::new("session.json"))
    );
    assert!(matches!(
        cli.command,
        Commands::Item(ItemCommands::Get { .. })
    ));
}

#[test]
fn parses_sync_options() {
    let cli = Cli::parse_from([
        "porkpie",
        "sync",
        "--server",
        "http://127.0.0.1:8000",
        "--api-key",
        "dev",
    ]);

    assert!(matches!(
        cli.command,
        Commands::Sync {
            server: Some(_),
            api_key: Some(_)
        }
    ));
}

#[test]
fn session_tracks_unlocked_vault_and_lock_state() {
    let vault_id = VaultId::new();
    let mut session = porkpie_cli::session::SessionState::unlocked(vault_id);

    assert_eq!(
        session.require_unlocked_vault().expect("unlocked"),
        vault_id
    );

    session.lock();
    assert!(session.require_unlocked_vault().is_err());
}
