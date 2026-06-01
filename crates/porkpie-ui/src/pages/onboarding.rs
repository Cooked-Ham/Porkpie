use crate::components::{button::Button, password_input::PasswordInput, text_input::TextInput};
use crate::state::{AppState, Screen};
use crate::utils::secret_key_storage;
use crate::utils::validation::validate_master_password;
use crate::vault_store::{VaultBackend, VaultStoreError, VaultSummary};
use dioxus::prelude::*;
use porkpie_types::LocalSecretKey;

#[derive(Default, Clone)]
struct PendingVault {
    summary: Option<VaultSummary>,
    password: String,
    secret_key_hex: String,
}

/// Onboarding / create-vault screen. Form validation matches the master
/// password policy and confirms the choice. On submit, a fresh local
/// secret key is generated, stored in the OS credential manager, and the
/// new vault is persisted. The user must save the recovery kit before
/// proceeding to the vault.
pub fn OnboardingPage<'a>(cx: Scope<'a, OnboardingPageProps>) -> Element<'a> {
    let backend = cx.props.backend.clone();
    let state = cx.props.state.clone();

    let name = use_state(cx, String::new);
    let password = use_state(cx, String::new);
    let confirmation = use_state(cx, String::new);
    let submitting = use_state(cx, || false);
    let error = use_state(cx, || None::<String>);
    let recovery_kit_json = use_state(cx, || None::<String>);
    let show_save_confirm = use_state(cx, || false);
    let saved_summary = use_state(cx, || None::<(VaultSummary, String)>);
    let pending = use_ref(cx, PendingVault::default);

    let name_setter = name.clone();
    let password_setter = password.clone();
    let confirmation_setter = confirmation.clone();
    let submitting_setter = submitting.clone();
    let error_setter = error.clone();
    let recovery_setter = recovery_kit_json.clone();
    let show_save_confirm_setter = show_save_confirm.clone();
    let saved_summary_setter = saved_summary.clone();

    let backend_for_submit = backend.clone();
    let submit = move |_| {
        if *submitting.get() {
            return;
        }
        let raw_name = name.get().clone();
        let raw_password = password.get().clone();
        let raw_confirmation = confirmation.get().clone();

        if let Err(validation_error) = validate_master_password(&raw_password, &raw_confirmation) {
            error_setter.set(Some(validation_error));
            return;
        }
        if raw_name.trim().is_empty() {
            error_setter.set(Some("Vault name is required".to_string()));
            return;
        }

        let secret_key = LocalSecretKey::generate();
        let secret_key_hex = secret_key.to_hex();

        submitting_setter.set(true);
        error_setter.set(None);
        let backend_handle = backend_for_submit.clone();
        let error_handle = error_setter.clone();
        let submitting_handle = submitting_setter.clone();
        let recovery_handle = recovery_setter.clone();
        let show_save_handle = show_save_confirm_setter.clone();
        let summary_handle = saved_summary_setter.clone();
        let pending_handle = pending.clone();
        cx.spawn(async move {
            let backend = backend_handle.read().clone();
            let result = backend
                .create_vault(&raw_name, &raw_password, &secret_key)
                .await;
            match result {
                Ok((summary, recovery_kit)) => {
                    let json = serde_json::to_string_pretty(&recovery_kit)
                        .unwrap_or_else(|e| format!("recovery kit: {e}"));
                    // Store the secret key in the OS credential manager
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Err(e) = secret_key_storage::store_secret_key(&summary.name, &secret_key) {
                            error_handle.set(Some(format!("Vault created, but could not store secret key in credential manager: {e}. Save the recovery kit below.")));
                        }
                    }
                    pending_handle.write().summary = Some(summary.clone());
                    pending_handle.write().password = raw_password;
                    pending_handle.write().secret_key_hex = secret_key_hex;
                    recovery_handle.set(Some(json.clone()));
                    summary_handle.set(Some((summary, json)));
                    show_save_handle.set(true);
                }
                Err(VaultStoreError::Unavailable) => {
                    error_handle.set(Some("Database backend is not available".to_string()));
                }
                Err(VaultStoreError::VaultAlreadyExists(name)) => {
                    error_handle.set(Some(format!("A vault named '{name}' already exists")));
                }
                Err(other) => {
                    error_handle.set(Some(format!("Could not create vault: {other}")));
                }
            }
            submitting_handle.set(false);
        });
    };

    let state_for_recovery = state.clone();
    let backend_for_recovery = backend.clone();
    let pending_for_recovery = pending.clone();
    let on_recovery_saved = move |_| {
        let state_handle = state_for_recovery.clone();
        let backend_handle = backend_for_recovery.clone();
        let pending_handle = pending_for_recovery.clone();
        cx.spawn(async move {
            let p = pending_handle.read().clone();
            if let Some(summary) = p.summary {
                let secret_key = match LocalSecretKey::from_hex(&p.secret_key_hex) {
                    Ok(k) => k,
                    Err(_) => {
                        state_handle.with_mut(|state| {
                            state.error = Some("Invalid secret key stored during onboarding. Use recovery kit.".to_string());
                            state.screen = Screen::Unlock;
                        });
                        return;
                    }
                };
                let backend = backend_handle.read().clone();
                match backend.unlock_vault(&summary.name, &p.password, &secret_key).await {
                    Ok(handle) => {
                        let items = match handle.list_items().await {
                            Ok(items) => items,
                            Err(_) => vec![],
                        };
                        state_handle.with_mut(|state| {
                            if !state.vaults.iter().any(|v| v.id == summary.id) {
                                state.vaults.push(summary.clone());
                            }
                            state.current_vault = Some(summary);
                            state.unlocked_handle = Some(handle);
                            state.items = items;
                            state.screen = Screen::List;
                            state.status = Some("Vault created and unlocked. Add your first item.".to_string());
                        });
                    }
                    Err(e) => {
                        state_handle.with_mut(|state| {
                            state.error = Some(format!("Vault created but could not unlock: {e}"));
                            state.screen = Screen::Unlock;
                        });
                    }
                }
            }
        });
        show_save_confirm.set(false);
    };

    let state_for_save_kit = state.clone();
    let on_save_kit = move |_| {
        let state_handle = state_for_save_kit.clone();
        if let Some((summary, json)) = saved_summary.get().clone() {
            let filename = format!("{}_recovery_kit.json", summary.name);
            #[cfg(not(target_arch = "wasm32"))]
            {
                let state_for_clip = state_handle.clone();
                let _ = crate::utils::clipboard::copy_to_clipboard(&json);
                state_for_clip.with_mut(|s| {
                    s.toast = Some("Recovery kit copied to clipboard. Paste and save it to a secure file.".to_string());
                });
            }
            let _ = filename;
            state_handle.with_mut(|s| {
                s.toast = Some("Recovery kit copied to clipboard. Paste and save it to a secure file.".to_string());
            });
        }
    };

    let state_for_open_existing = state.clone();
    let on_open_existing = move |_| {
        state_for_open_existing.with_mut(|s| s.screen = Screen::Unlock);
    };

    cx.render(rsx! {
        section { class: "screen", id: "onboarding",
            div { class: "screen-header",
                p { class: "eyebrow", "New vault" }
                h1 { "Create your Porkpie vault" }
                p { class: "muted", "Choose a master password with at least 16 characters. A local secret key will be generated automatically and stored securely on this device." }
            }
            form { class: "panel form-grid",
                TextInput {
                    label: "Vault name",
                    value: "{name.get()}",
                    placeholder: "e.g. Personal",
                    on_input: move |value: String| name_setter.set(value),
                    auto_complete: "off"
                }
                PasswordInput {
                    label: "Master password",
                    value: "{password.get()}",
                    on_input: move |value: String| password_setter.set(value),
                    auto_complete: "new-password"
                }
                PasswordInput {
                    label: "Confirm master password",
                    value: "{confirmation.get()}",
                    on_input: move |value: String| confirmation_setter.set(value),
                    auto_complete: "new-password"
                }
                error.get().as_ref().map(|err| rsx! {
                    div { class: "inline-error", role: "alert", "{err}" }
                }),
                div { class: "actions",
                    Button {
                        label: if *submitting.get() { "Creating..." } else { "Create vault" },
                        variant: "btn-primary",
                        disabled: *submitting.get(),
                        on_click: submit
                    }
                    Button {
                        label: "Open existing",
                        variant: "btn-secondary",
                        on_click: on_open_existing
                    }
                }
            }
            if *show_save_confirm.get() {
                rsx! {
                    div { class: "modal-backdrop",
                        div { class: "modal",
                            h2 { "Save your recovery kit" }
                            p { class: "muted", "Your vault has been created. The local secret key is stored on this device, but you MUST save the recovery kit as a backup. Without it, you cannot recover your vault if this device fails." }
                            div { class: "actions",
                                Button {
                                    label: "Copy recovery kit to clipboard",
                                    variant: "btn-primary",
                                    on_click: on_save_kit
                                }
                                Button {
                                    label: "I have saved my recovery kit",
                                    variant: "btn-secondary",
                                    on_click: on_recovery_saved
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
pub struct OnboardingPageProps {
    pub state: UseRef<AppState>,
    pub backend: UseRef<VaultBackend>,
}
