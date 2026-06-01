//! Web shell for the shared Porkpie Dioxus UI.
//!
//! The crate is a thin Dioxus web entrypoint: the root `App` lives in
//! `porkpie-ui` and the web shell simply hands it to
//! `dioxus_web::launch`. The web build is rendered as a WebAssembly
//! module and bundled with `wasm-bindgen` (see `build-web.ps1`).

pub use porkpie_ui::App;

pub mod launch;
pub use launch::LaunchConfig;
