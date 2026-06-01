//! Desktop shell for the shared Porkpie Dioxus UI.
//!
//! This crate is the desktop launch surface. It owns a real Dioxus
//! desktop application that opens a WebView2 window and renders the
//! `porkpie-ui` `App` component. All vault I/O flows through the same
//! `porkpie-ui::vault_store` backend that the web shell uses; on the
//! desktop the backend is backed by a real SQLite database stored next
//! to the binary.

pub use porkpie_ui::App;
