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

    // Create the tokio runtime and keep it alive for the entire app.
    // The UI's initial_load future (and all other SQLx-backed vault I/O)
    // needs a tokio reactor running. The runtime is dropped automatically
    // when main() returns after the Dioxus window closes.
    let _runtime = tokio::runtime::Runtime::new().expect("tokio runtime");

    let dioxus_config = config.into_dioxus_config();
    dioxus_desktop::launch_cfg(porkpie_ui::App, dioxus_config);
}
