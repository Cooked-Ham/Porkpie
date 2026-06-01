//! Tray command queue shared between the desktop shell and the UI.
//!
//! The desktop tray event thread pushes commands into the global
//! [`TRAY_COMMANDS`] queue. The UI root component polls the queue on
//! every render and acts on them (show window, lock vault, quit).

use std::sync::{Arc, LazyLock, Mutex};

/// Commands sent from the tray menu to the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayCommand {
    ShowWindow,
    LockVault,
    Quit,
}

/// Global queue of tray commands waiting to be processed by the UI.
pub static TRAY_COMMANDS: LazyLock<Arc<Mutex<Vec<TrayCommand>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));
