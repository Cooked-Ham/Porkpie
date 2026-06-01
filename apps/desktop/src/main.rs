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

    // The database startup check is handled by the UI so that failures
    // surface as a friendly GUI error screen instead of a terminal message.
    // The UI initial_load future connects to SQLite and routes to the
    // DbError screen on failure, offering Retry / Open Data Folder / Reset.
    let dioxus_config = config.into_dioxus_config();
    dioxus_desktop::launch_cfg(porkpie_ui::App, dioxus_config);
}
