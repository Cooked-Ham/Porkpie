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

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod launch_config;
mod tray;

use launch_config::LaunchConfig;

fn main() {
    let config = LaunchConfig::load();

    // Load settings so we can configure the window close behaviour before
    // the event loop starts. The UI will load the same file again in
    // `initial_load` and overwrite the in-memory state with the same values.
    let settings_path = porkpie_ui::settings_store::default_settings_path();
    let settings = porkpie_ui::settings_store::load_or_default(&settings_path);

    // Create the tokio runtime and keep it alive for the entire app.
    let _runtime = tokio::runtime::Runtime::new().expect("tokio runtime");

    let dioxus_config = config.into_dioxus_config(settings.close_to_tray);

    let _tray = tray::create_tray_icon();

    dioxus_desktop::launch_cfg(porkpie_ui::App, dioxus_config);
}
