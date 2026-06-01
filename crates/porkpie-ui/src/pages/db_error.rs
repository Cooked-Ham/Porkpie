use crate::components::button::Button;
use crate::state::{AppState, Screen};
use crate::vault_store::VaultBackend;
use dioxus::prelude::*;

/// Database error screen shown when the SQLite backend cannot be opened.
/// Provides actionable recovery options: Retry, Open Data Folder, Reset Local Vault.
pub fn DbErrorPage<'a>(cx: Scope<'a, DbErrorPageProps>) -> Element<'a> {
    let state = cx.props.state.clone();
    let backend = cx.props.backend.clone();

    let error = state.with(|s| s.error.clone());
    let db_path = state.with(|s| s.db_path.clone());
    let show_reset_confirm = use_state(cx, || false);

    let state_for_retry = state.clone();
    let backend_for_retry = backend.clone();
    let on_retry = move |_| {
        let state_handle = state_for_retry.clone();
        let backend_handle = backend_for_retry.clone();
        cx.spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let url = crate::app::database_url_from_env()
                    .unwrap_or_else(|| "sqlite:porkpie.db".to_string());
                let backend = match VaultBackend::connect_sqlite(&url).await {
                    Ok(backend) => backend,
                    Err(error) => {
                        state_handle.with_mut(|state| {
                            state.error = Some(format!("Database error: {error}"));
                            state.db_path = Some(url);
                        });
                        return;
                    }
                };
                backend_handle.set(backend.clone());
                match backend.list_vault_summaries().await {
                    Ok(summaries) => {
                        state_handle.with_mut(|state| {
                            state.vaults = summaries;
                            state.error = None;
                            state.db_path = None;
                            state.screen = if state.vaults.is_empty() {
                                Screen::Onboarding
                            } else {
                                Screen::Unlock
                            };
                        });
                    }
                    Err(error) => {
                        state_handle.with_mut(|state| {
                            state.error = Some(format!("Failed to list vaults: {error}"));
                        });
                    }
                }
            }
        });
    };

    let db_path_for_folder = db_path.clone();
    let on_open_folder = move |_| {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = db_path_for_folder.clone().unwrap_or_default();
            if let Some(parent) = std::path::Path::new(&path).parent() {
                let _ = std::process::Command::new("explorer")
                    .arg(parent.as_os_str())
                    .spawn();
            }
        }
    };

    let on_reset = move |_| {
        show_reset_confirm.set(true);
    };

    let state_for_reset = state.clone();
    let backend_for_reset = backend.clone();
    let on_reset_confirmed = move |_| {
        let state_handle = state_for_reset.clone();
        let backend_handle = backend_for_reset.clone();
        show_reset_confirm.set(false);
        cx.spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let url = crate::app::database_url_from_env()
                    .unwrap_or_else(|| "sqlite:porkpie.db".to_string());
                // Remove the database file if possible
                let path = url.replace("sqlite://", "").replace("?mode=rwc", "");
                let std_path = std::path::Path::new(&path);
                if std_path.exists() {
                    let _ = std::fs::remove_file(std_path);
                    let _ = std::fs::remove_file(std_path.with_extension("db-shm"));
                    let _ = std::fs::remove_file(std_path.with_extension("db-wal"));
                }
                // Reconnect
                let backend = match VaultBackend::connect_sqlite(&url).await {
                    Ok(backend) => backend,
                    Err(error) => {
                        state_handle.with_mut(|state| {
                            state.error = Some(format!("Database error: {error}"));
                            state.db_path = Some(url);
                        });
                        return;
                    }
                };
                backend_handle.set(backend.clone());
                state_handle.with_mut(|state| {
                    state.vaults.clear();
                    state.error = None;
                    state.db_path = None;
                    state.screen = Screen::Onboarding;
                    state.status =
                        Some("Local vault reset. Create a new vault to continue.".to_string());
                });
            }
        });
    };

    cx.render(rsx! {
        section { class: "screen", id: "db-error",
            div { class: "screen-header",
                p { class: "eyebrow", "Error" }
                h1 { "Database problem" }
                p { class: "muted", "Porkpie could not open the local database." }
            }
            div { class: "panel form-grid",
                db_path.as_ref().map(|path| rsx! {
                    div { class: "field",
                        span { "Database path" }
                        div { class: "generated", "{path}" }
                    }
                }),
                error.as_ref().map(|msg| rsx! {
                    div { class: "inline-error", role: "alert", "{msg}" }
                }),
                div { class: "actions",
                    Button { label: "Retry", variant: "btn-primary", on_click: on_retry }
                    Button { label: "Open Data Folder", variant: "btn-secondary", on_click: on_open_folder }
                    Button { label: "Reset Local Vault", variant: "btn-danger", on_click: on_reset }
                }
                p { class: "muted", "Resetting the local vault will delete the database file and all stored vaults. This cannot be undone unless you have a backup." }
            }
            if *show_reset_confirm.get() {
                rsx! {
                    crate::components::modal::Modal {
                        title: "Reset local vault",
                        message: "This will permanently delete the local database file and all vaults stored on this device. Make sure you have backups or recovery kits before proceeding.",
                        confirm_label: "Delete everything",
                        cancel_label: "Cancel",
                        danger: true,
                        on_confirm: on_reset_confirmed,
                        on_cancel: move |_| show_reset_confirm.set(false)
                    }
                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
pub struct DbErrorPageProps {
    pub state: UseRef<AppState>,
    pub backend: UseRef<VaultBackend>,
}
