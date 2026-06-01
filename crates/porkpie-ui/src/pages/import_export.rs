use crate::components::{
    button::Button, modal::Modal, password_input::PasswordInput, text_input::TextInput,
};
use crate::state::AppState;
use crate::vault_store::VaultBackend;
#[cfg(not(target_arch = "wasm32"))]
use crate::vault_store::VaultStoreError;
use dioxus::prelude::*;
use porkpie_types::LocalSecretKey;

/// Backup import/export screen. The encrypted export path uses real
/// `porkpie-import` encryption; the plaintext export requires explicit
/// confirmation before any plaintext is materialised.
pub fn ImportExportPage<'a>(cx: Scope<'a, ImportExportPageProps>) -> Element<'a> {
    let backend = cx.props.backend.clone();
    let state_ref = &cx.props.state;

    let status = use_state(cx, || None::<String>);
    let error = use_state(cx, || None::<String>);
    let export_path = use_state(cx, || None::<String>);
    let import_password = use_state(cx, String::new);
    let import_secret_key = use_state(cx, String::new);
    let submitting = use_state(cx, || false);
    let show_plaintext_confirm = use_state(cx, || false);

    let status_setter = status.clone();
    let error_setter = error.clone();
    let state_for_export = state_ref.clone();
    let backend_for_export = backend.clone();
    let on_encrypted_export = move |_| {
        status_setter.set(Some("Preparing encrypted backup...".to_string()));
        error_setter.set(None);
        let _backend_handle = backend_for_export.clone();
        let state_handle = state_for_export.clone();
        let error_handle = error_setter.clone();
        let status_handle = status_setter.clone();
        let export_path_handle = export_path.clone();
        cx.spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let handle_opt = state_handle.with(|s| s.unlocked_handle.as_ref().cloned());
                let Some(handle) = handle_opt else {
                    error_handle.set(Some("Unlock a vault before exporting".to_string()));
                    return;
                };
                match handle.export_encrypted().await {
                    Ok(export) => {
                        let dialog = rfd::FileDialog::new()
                            .set_title("Save encrypted Porkpie backup")
                            .set_file_name(&export.suggested_filename)
                            .add_filter("JSON", &["json"]);
                        if let Some(path) = dialog.save_file() {
                            match std::fs::write(&path, &export.json) {
                                Ok(_) => {
                                    status_handle.set(Some(format!(
                                        "Encrypted backup saved to {}",
                                        path.display()
                                    )));
                                    export_path_handle
                                        .set(Some(path.to_string_lossy().to_string()));
                                }
                                Err(e) => {
                                    error_handle.set(Some(format!("Could not write file: {e}")));
                                }
                            }
                        } else {
                            status_handle.set(Some("Save cancelled".to_string()));
                        }
                    }
                    Err(error) => {
                        error_handle.set(Some(format!("Export failed: {error}")));
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = (state_handle, _backend_handle);
                error_handle.set(Some(
                    "Backup export is not available in this build".to_string(),
                ));
            }
        });
    };

    let on_plaintext_export = move |_| {
        show_plaintext_confirm.set(true);
    };

    let on_plaintext_confirmed = move |_| {
        show_plaintext_confirm.set(false);
        status.set(Some("Preparing plaintext backup...".to_string()));
        error.set(None);
        let state_handle = state_ref.clone();
        let error_handle = error.clone();
        let status_handle = status.clone();
        let export_path_handle = export_path.clone();
        cx.spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let handle_opt = state_handle.with(|s| s.unlocked_handle.as_ref().cloned());
                let Some(handle) = handle_opt else {
                    error_handle.set(Some("Unlock a vault before exporting".to_string()));
                    return;
                };
                match handle.export_plaintext(true).await {
                    Ok(plain) => {
                        let json = serde_json::to_string_pretty(&plain)
                            .unwrap_or_else(|e| format!("plaintext export: {e}"));
                        let dialog = rfd::FileDialog::new()
                            .set_title("Save plaintext Porkpie backup")
                            .set_file_name(format!("{}_plaintext_backup.json", plain.vault_name))
                            .add_filter("JSON", &["json"]);
                        if let Some(path) = dialog.save_file() {
                            match std::fs::write(&path, &json) {
                                Ok(_) => {
                                    status_handle.set(Some(format!(
                                        "Plaintext backup saved to {}. Handle with care.",
                                        path.display()
                                    )));
                                    export_path_handle
                                        .set(Some(path.to_string_lossy().to_string()));
                                }
                                Err(e) => {
                                    error_handle.set(Some(format!("Could not write file: {e}")));
                                }
                            }
                        } else {
                            status_handle.set(Some("Save cancelled".to_string()));
                        }
                    }
                    Err(error) => {
                        error_handle.set(Some(format!("Plaintext export failed: {error}")));
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = state_handle;
                error_handle.set(Some(
                    "Plaintext export is not available in this build".to_string(),
                ));
            }
        });
    };

    let state_for_import = state_ref.clone();
    let status_for_import = status.clone();
    let error_for_import = error.clone();
    let import_password_setter = import_password.clone();
    let import_secret_key_setter = import_secret_key.clone();
    let on_import = move |_| {
        let password = import_password.get().clone();
        let secret_key_hex = import_secret_key.get().clone();
        if password.is_empty() {
            error_for_import.set(Some("Backup password is required".to_string()));
            return;
        }
        let secret_key = match LocalSecretKey::from_hex(&secret_key_hex) {
            Ok(key) => key,
            Err(parse_error) => {
                error_for_import.set(Some(format!("Invalid local secret key: {parse_error}")));
                return;
            }
        };
        status_for_import.set(Some("Importing backup...".to_string()));
        error_for_import.set(None);
        submitting.set(true);
        let state_handle: UseRef<AppState> = state_for_import.clone();
        let error_handle: UseState<Option<String>> = error_for_import.clone();
        let submitting_handle: UseState<bool> = submitting.clone();
        let backend_handle: UseRef<VaultBackend> = backend.clone();
        let status_handle: UseState<Option<String>> = status_for_import.clone();
        cx.spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let handle_opt = state_handle.with(|s| s.unlocked_handle.as_ref().cloned());
                let Some(handle) = handle_opt else {
                    error_handle.set(Some("Unlock a vault before importing".to_string()));
                    submitting_handle.set(false);
                    return;
                };

                let dialog = rfd::FileDialog::new()
                    .set_title("Select encrypted Porkpie backup")
                    .add_filter("JSON", &["json", "enc"]);
                let path = match dialog.pick_file() {
                    Some(p) => p,
                    None => {
                        status_handle.set(Some("Import cancelled".to_string()));
                        submitting_handle.set(false);
                        return;
                    }
                };

                let raw_json = match std::fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(e) => {
                        error_handle.set(Some(format!("Could not read file: {e}")));
                        submitting_handle.set(false);
                        return;
                    }
                };

                let _ = backend_handle;
                match handle
                    .import_encrypted_with_keys(
                        &password,
                        &secret_key,
                        &raw_json,
                        Default::default(),
                    )
                    .await
                {
                    Ok(summary) => {
                        let items = match handle.list_items().await {
                            Ok(items) => items,
                            Err(error) => {
                                error_handle.set(Some(format!("List items failed: {error}")));
                                submitting_handle.set(false);
                                return;
                            }
                        };
                        state_handle.with_mut(|s| {
                            s.items = items;
                            s.status = Some(format!(
                                "Imported {} items, skipped {} duplicates",
                                summary.imported, summary.skipped
                            ));
                        });
                        status_handle.set(Some(format!(
                            "Imported {} items (skipped {} duplicates)",
                            summary.imported, summary.skipped
                        )));
                    }
                    Err(VaultStoreError::WrongPassword) => {
                        error_handle.set(Some(
                            "Wrong password or local secret key for backup".to_string(),
                        ));
                    }
                    Err(VaultStoreError::Json(message)) => {
                        error_handle.set(Some(format!("Invalid backup JSON: {message}")));
                    }
                    Err(other) => {
                        error_handle.set(Some(format!("Import failed: {other}")));
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = (state_handle, backend_handle, password, secret_key);
                error_handle.set(Some("Import is not available in this build".to_string()));
            }
            submitting_handle.set(false);
        });
    };

    let status_text = status.get().clone();
    let error_text = error.get().clone();
    let saved_path = export_path.get().clone();
    #[cfg(not(target_arch = "wasm32"))]
    let is_unlocked = state_ref.with(|s| s.unlocked_handle.is_some());
    #[cfg(target_arch = "wasm32")]
    let is_unlocked = false;

    cx.render(rsx! {
        section { class: "screen", id: "backup",
            div { class: "screen-header",
                p { class: "eyebrow", "Backup" }
                h1 { "Import and export" }
                p { class: "muted", "Encrypted backups are end-to-end encrypted. Plaintext exports expose secrets and require explicit confirmation." }
            }
            div { class: "panel form-grid",
                div { class: "backup-row",
                    div {
                        h2 { "Export encrypted backup" }
                        p { class: "muted", "Produces a JSON file that re-encrypts every item with the same vault key." }
                    }
                    Button {
                        label: "Export encrypted",
                        variant: "btn-primary",
                        disabled: !is_unlocked,
                        on_click: on_encrypted_export
                    }
                }
                div { class: "backup-row",
                    div {
                        h2 { "Export plaintext backup" }
                        p { class: "muted", "Writes secrets in plaintext. Only for emergency offline storage." }
                    }
                    Button {
                        label: "Export plaintext",
                        variant: "btn-danger",
                        disabled: !is_unlocked,
                        on_click: on_plaintext_export
                    }
                }
                saved_path.as_ref().map(|path| rsx! {
                    div { class: "toast", role: "status", "Saved to: {path}" }
                }),
                div { class: "backup-row",
                    div {
                        h2 { "Import encrypted backup" }
                        p { class: "muted", "Select a JSON backup file from a previous export." }
                    }
                }
                PasswordInput {
                    label: "Backup master password",
                    value: "{import_password.get()}",
                    on_input: move |value: String| import_password_setter.set(value)
                }
                TextInput {
                    label: "Local secret key (hex)",
                    value: "{import_secret_key.get()}",
                    input_type: "password",
                    on_input: move |value: String| import_secret_key_setter.set(value)
                }
                div { class: "actions",
                    Button {
                        label: if *submitting.get() { "Importing..." } else { "Import" },
                        variant: "btn-primary",
                        disabled: !is_unlocked || *submitting.get(),
                        on_click: on_import
                    }
                }
                status_text.as_ref().map(|msg| rsx! {
                    div { class: "toast", role: "status", "{msg}" }
                }),
                error_text.as_ref().map(|msg| rsx! {
                    div { class: "inline-error", role: "alert", "{msg}" }
                })
            }
            if *show_plaintext_confirm.get() {
                rsx! {
                    Modal {
                        title: "Export plaintext backup?",
                        message: "This will write every secret in cleartext. Anyone with this file can read your data. Confirm only if you intend to print or store it offline in a safe location.",
                        confirm_label: "I understand, export plaintext",
                        cancel_label: "Cancel",
                        danger: true,
                        on_confirm: on_plaintext_confirmed,
                        on_cancel: move |_| show_plaintext_confirm.set(false)
                    }
                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
pub struct ImportExportPageProps {
    pub state: UseRef<AppState>,
    pub backend: UseRef<VaultBackend>,
}
