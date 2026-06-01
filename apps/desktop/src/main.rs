//! Desktop entrypoint. Launches a Dioxus desktop window that hosts the
//! shared Porkpie UI.
//!
//! The window title, the SQLite location, and other launch behaviour
//! are all driven by environment variables so that the same binary can
//! be used for development, manual QA, and packaging.
//!
//! The Dioxus desktop renderer is wired through `dioxus_desktop::launch`
//! so that `cargo run -p porkpie-desktop` produces a working window
//! without requiring the Dioxus CLI.

mod launch_config;

use launch_config::LaunchConfig;

fn main() {
    let config = LaunchConfig::load();

    // Run a startup self-check on the database before handing off to the UI.
    // On failure, print a helpful error to stderr and exit with a non-zero
    // code so the user knows the exact category of the problem.
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime.block_on(async {
        if let Err(error) = porkpie_store::startup_self_check(&config.database_url).await {
            eprintln!("porkpie-desktop: database startup failed: {error}");
            eprintln!("  database_url: {}", config.database_url);
            eprintln!("  HINT: override the path with PORKPIE_DATABASE_URL or PORKPIE_DATA_DIR");
            std::process::exit(1);
        }
    });

    let dioxus_config = config.into_dioxus_config();
    dioxus_desktop::launch_cfg(porkpie_ui::App, dioxus_config);
}
