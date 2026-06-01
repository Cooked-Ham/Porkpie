use crate::components::{button::Button, password_input::PasswordInput, text_input::TextInput};
use crate::state::{AppState, Screen};
use crate::utils::validation::validate_master_password;
use crate::vault_store::{VaultBackend, VaultStoreError};
use dioxus::prelude::*;
use porkpie_types::LocalSecretKey;

/// Onboarding / create-vault screen. Form validation matches the master
/// password policy and confirms the choice. On submit, a fresh local
/// secret key is generated and the new vault is persisted.
pub fn OnboardingPage<'a>(cx: Scope<'a, OnboardingPageProps>) -> Element<'a> {
    let backend = cx.props.backend.clone();
    let state = cx.props.state.clone();

    let name = use_state(cx, String::new);
    let password = use_state(cx, String::new);
    let confirmation = use_state(cx, String::new);
    let secret_key_hex = use_state(cx, String::new);
    let submitting = use_state(cx, || false);
    let error = use_state(cx, || None::<String>);
    let recovery_kit_json = use_state(cx, || None::<String>);

    let name_setter = name.clone();
    let password_setter = password.clone();
    let confirmation_setter = confirmation.clone();
    let secret_key_setter = secret_key_hex.clone();
    let submitting_setter = submitting.clone();
    let error_setter = error.clone();
    let recovery_setter = recovery_kit_json.clone();

    let submit = move |_| {
        if *submitting.get() {
            return;
        }
        let raw_name = name.get().clone();
        let raw_password = password.get().clone();
        let raw_confirmation = confirmation.get().clone();
        let raw_secret_key = secret_key_hex.get().clone();

        if let Err(validation_error) = validate_master_password(&raw_password, &raw_confirmation) {
            error_setter.set(Some(validation_error));
            return;
        }
        if raw_name.trim().is_empty() {
            error_setter.set(Some("Vault name is required".to_string()));
            return;
        }

        let secret_key = if raw_secret_key.trim().is_empty() {
            LocalSecretKey::generate()
        } else {
            match LocalSecretKey::from_hex(&raw_secret_key) {
                Ok(key) => key,
                Err(parse_error) => {
                    error_setter.set(Some(format!("Invalid local secret key: {parse_error}")));
                    return;
                }
            }
        };

        submitting_setter.set(true);
        error_setter.set(None);
        let backend_handle = backend.clone();
        let state_handle = state.clone();
        let error_handle = error_setter.clone();
        let submitting_handle = submitting_setter.clone();
        let recovery_handle = recovery_setter.clone();
        cx.spawn(async move {
            let backend = backend_handle.read().clone();
            let result = backend
                .create_vault(&raw_name, &raw_password, &secret_key)
                .await;
            match result {
                Ok((summary, recovery_kit)) => {
                    let json = serde_json::to_string_pretty(&recovery_kit)
                        .unwrap_or_else(|e| format!("recovery kit: {e}"));
                    recovery_handle.set(Some(json));
                    state_handle.with_mut(|state| {
                        state.vaults.push(summary.clone());
                        state.current_vault = Some(summary);
                        state.items.clear();
                        state.screen = Screen::List;
                        state.status = Some(
                            "Vault created. Save your recovery kit before locking.".to_string(),
                        );
                    });
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

    cx.render(rsx! {
        section { class: "screen", id: "onboarding",
            div { class: "screen-header",
                p { class: "eyebrow", "New vault" }
                h1 { "Create your Porkpie vault" }
                p { class: "muted", "Choose a master password with at least 16 characters and an optional local secret key. Without both, the vault cannot be unlocked." }
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
                TextInput {
                    label: "Local secret key (optional - auto-generates if blank)",
                    value: "{secret_key_hex.get()}",
                    placeholder: "64 hex characters - leave blank to generate",
                    on_input: move |value: String| secret_key_setter.set(value),
                    auto_complete: "off"
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
                    a { class: "btn btn-secondary", href: "#unlock", "Open existing" }
                }
            }
            recovery_kit_json.get().as_ref().map(|json| rsx! {
                div { class: "panel form-grid",
                    h2 { "Recovery kit" }
                    p { class: "muted", "Save this JSON file in a secure offline location. You will need it together with your master password to recover the vault." }
                    pre { class: "generated", "{json}" }
                }
            })
        }
    })
}

#[derive(Props, PartialEq)]
pub struct OnboardingPageProps {
    pub state: UseRef<AppState>,
    pub backend: UseRef<VaultBackend>,
}
