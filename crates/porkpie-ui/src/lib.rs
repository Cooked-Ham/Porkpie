#![allow(non_snake_case)]

//! Shared Dioxus user interface for Porkpie desktop and web shells.

pub mod app;
pub mod components;
pub mod pages;
pub mod state;
pub mod utils;
pub mod vault_store;

pub use app::App;
pub use vault_store::{DecryptedItem, ItemSummary, VaultBackend, VaultSummary};
