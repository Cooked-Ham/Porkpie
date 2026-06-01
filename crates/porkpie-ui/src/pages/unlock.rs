use crate::components::{button::Button, password_input::PasswordInput, text_input::TextInput};
use crate::state::AppState;
#[cfg(not(target_arch = "wasm32"))]
use crate::state::Screen;
use crate::vault_store::{VaultBackend, VaultStoreError};
use dioxus::prelude::*;
use porkpie_types::LocalSecretKey;

/// Unlock screen for an existing vault. Loads the list of vaults from
/// the backend and offers a dropdown selector plus a secret key field.
pub fn UnlockPage<'a>(cx: Scope<'a, UnlockPageProps>) -> Element<'a> {
    let backend = cx.props.backend.clone();
    let state_ref = &cx.props.state;

    let name = use_state(cx, String::new);
    let password = use_state(cx, String::new);
    let secret_key_hex = use_state(cx, String::new);
    let submitting = use_state(cx, || false);
    let error = use_state(cx, || None::<String>);

    let name_setter = name.clone();
    let password_setter = password.clone();
    let secret_key_setter = secret_key_hex.clone();
    let submitting_setter = submitting.clone();
    let error_setter = error.clone();
    let submitting_reader = submitting.clone();
    let name_reader = name.clone();
    let password_reader = password.clone();
    let secret_key_reader = secret_key_hex.clone();

    let submit = move |_| {
        if *submitting_reader.get() {
            return;
        }
        let raw_name = name_reader.get().clone();
        let raw_password = password_reader.get().clone();
        let raw_secret_key = secret_key_reader.get().clone();

        if raw_name.trim().is_empty() {
            error_setter.set(Some("Select or enter a vault name".to_string()));
            return;
        }
        if raw_password.is_empty() {
            error_setter.set(Some("Master password is required".to_string()));
            return;
        }
        if raw_secret_key.trim().is_empty() {
            error_setter.set(Some("Local secret key is required".to_string()));
            return;
        }

        let secret_key = match LocalSecretKey::from_hex(&raw_secret_key) {
            Ok(key) => key,
            Err(parse_error) => {
                error_setter.set(Some(format!("Invalid local secret key: {parse_error}")));
                return;
            }
        };

        submitting_setter.set(true);
        error_setter.set(None);
        let backend_handle = backend.clone();
        let error_handle = error_setter.clone();
        #[cfg(not(target_arch = "wasm32"))]
        let state_handle = state_ref.clone();
        #[cfg(not(target_arch = "wasm32"))]
        let submitting_handle = submitting_setter.clone();
        #[cfg(not(target_arch = "wasm32"))]
        let raw_name_for_async = raw_name.clone();
        cx.spawn(async move {
            let backend = backend_handle.read().clone();
            let result = backend
                .unlock_vault(&raw_name, &raw_password, &secret_key)
                .await;
            match result {
                #[cfg(not(target_arch = "wasm32"))]
                Ok(handle) => {
                    let items = match handle.list_items().await {
                        Ok(items) => items,
                        Err(error) => {
                            error_handle.set(Some(format!("Failed to list items: {error}")));
                            submitting_handle.set(false);
                            return;
                        }
                    };
                    let summary = handle.summary.clone();
                    state_handle.with_mut(|state| {
                        state.error = None;
                        if !state.vaults.iter().any(|v| v.id == summary.id) {
                            state.vaults.push(summary.clone());
                        }
                        state.current_vault = Some(summary);
                        state.unlocked_handle = Some(handle);
                        state.items = items;
                        state.current_item = None;
                        state.screen = Screen::List;
                        state.status = Some(format!("Unlocked vault '{}'", raw_name_for_async));
                    });
                }
                #[cfg(target_arch = "wasm32")]
                Ok(_) => {
                    error_handle.set(Some(
                        "Vault unlock is not available in this build".to_string(),
                    ));
                }
                Err(VaultStoreError::WrongPassword) => {
                    error_handle.set(Some(
                        "Wrong master password or local secret key".to_string(),
                    ));
                }
                Err(VaultStoreError::VaultNotFound(name)) => {
                    error_handle.set(Some(format!("No vault named '{name}' exists")));
                }
                Err(VaultStoreError::Unavailable) => {
                    error_handle.set(Some("Database backend is not available".to_string()));
                }
                Err(other) => {
                    error_handle.set(Some(format!("Could not unlock vault: {other}")));
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                submitting_handle.set(false);
            }
        });
    };

    let vault_options = state_ref.with(|state| state.vaults.clone());
    let default_value = name.get().clone();

    cx.render(rsx! {
        section { class: "screen", id: "unlock",
            div { class: "screen-header",
                p { class: "eyebrow", "Locked" }
                h1 { "Unlock vault" }
                p { class: "muted", "Choose a vault, then provide the master password and the local secret key. Both are required." }
            }
            form { class: "panel form-grid",
                if !vault_options.is_empty() {
                    rsx! {
                        label { class: "field",
                            span { "Existing vaults" }
                            select {
                                class: "input",
                                onchange: move |event| {
                                    name_setter.set(event.value.clone());
                                },
                                option { value: "", "Select a vault..." }
                                vault_options.iter().map(|vault| rsx! {
                                    option { value: "{vault.name}", "{vault.name}" }
                                })
                            }
                        }
                    }
                } else {
                    rsx! {
                        TextInput {
                            label: "Vault name",
                            value: "{default_value}",
                            placeholder: "Vault name",
                            on_input: move |value: String| name_setter.set(value),
                            auto_complete: "off"
                        }
                    }
                }
                PasswordInput {
                    label: "Master password",
                    value: "{password.get()}",
                    on_input: move |value: String| password_setter.set(value),
                    auto_complete: "current-password"
                }
                TextInput {
                    label: "Local secret key (hex)",
                    value: "{secret_key_hex.get()}",
                    placeholder: "64 hex characters",
                    on_input: move |value: String| secret_key_setter.set(value),
                    auto_complete: "off",
                    input_type: "password"
                }
                error.get().as_ref().map(|err| rsx! {
                    div { class: "inline-error", role: "alert", "{err}" }
                }),
                div { class: "actions",
                    Button {
                        label: if *submitting.get() { "Unlocking..." } else { "Unlock" },
                        variant: "btn-primary",
                        disabled: *submitting.get(),
                        on_click: submit
                    }
                    a { class: "btn btn-secondary", href: "#onboarding", "Create new" }
                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
pub struct UnlockPageProps {
    pub state: UseRef<AppState>,
    pub backend: UseRef<VaultBackend>,
}
