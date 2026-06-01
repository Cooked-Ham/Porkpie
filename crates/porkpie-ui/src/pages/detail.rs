use crate::components::{
    button::Button, modal::Modal, password_input::PasswordInput, text_input::TextInput,
};
use crate::state::{AppState, Screen};
use crate::vault_store::DecryptedItem;
#[cfg(not(target_arch = "wasm32"))]
use crate::vault_store::VaultStoreError;
use dioxus::prelude::*;
use porkpie_types::{
    APIKeySecret, CustomSecret, DatabaseSecret, IdentitySecret, ItemType, LoginSecret,
    RecoveryCodesSecret, SSHKeySecret, SecureNoteSecret, ServerSecret, SoftwareLicenseSecret,
};

/// Item detail and edit surface. Shows decrypted data for a selected item
/// or an empty form when creating a new one. Save persists via the unlocked
/// vault handle. Delete prompts for confirmation.
pub fn ItemDetailPage<'a>(cx: Scope<'a, ItemDetailPageProps>) -> Element<'a> {
    let state_ref = &cx.props.state;

    let form_type = use_state(cx, String::new);
    let form_title = use_state(cx, String::new);
    let form_field_a = use_state(cx, String::new);
    let form_field_b = use_state(cx, String::new);
    let form_field_c = use_state(cx, String::new);
    let form_notes = use_state(cx, String::new);
    let form_error = use_state(cx, || None::<String>);
    let show_delete_confirm = use_state(cx, || false);
    let submitting = use_state(cx, || false);

    let current_vault = state_ref.with(|s| s.current_vault.clone());
    let is_unlocked = current_vault.is_some();
    let item_id = state_ref.with(|s| match &s.screen {
        Screen::Detail(id) => Some(*id),
        _ => None,
    });
    let is_new = state_ref.with(|s| matches!(s.screen, Screen::NewItem));

    // Load the item from the unlocked handle when the screen changes.
    let state_for_load = state_ref.clone();
    let item_id_for_load = item_id;
    use_future(cx, &(item_id_for_load, is_new), |(id, is_new)| {
        let state = state_for_load.clone();
        async move {
            if is_new {
                // Clear form for a new item.
                state.with_mut(|s| {
                    s.current_item = None;
                });
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(id) = id {
                        let handle_opt = state.with(|s| s.unlocked_handle.as_ref().cloned());
                        if let Some(handle) = handle_opt {
                            match handle.get_item(id).await {
                                Ok(decrypted) => {
                                    state.with_mut(|s| {
                                        s.current_item = Some(decrypted);
                                    });
                                }
                                Err(error) => {
                                    state.with_mut(|s| {
                                        s.error = Some(format!("Could not load item: {error}"));
                                    });
                                }
                            }
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = id;
                }
            }
        }
    });

    // Sync form values from the loaded item.
    let loaded = state_ref.with(|s| s.current_item.clone());
    let form_type_setter = form_type.clone();
    let form_title_setter = form_title.clone();
    let form_field_a_setter = form_field_a.clone();
    let form_field_b_setter = form_field_b.clone();
    let form_field_c_setter = form_field_c.clone();
    let form_notes_setter = form_notes.clone();
    use_future(cx, &(loaded.clone(),), |(loaded,)| {
        let form_type_setter = form_type_setter.clone();
        let form_title_setter = form_title_setter.clone();
        let form_field_a_setter = form_field_a_setter.clone();
        let form_field_b_setter = form_field_b_setter.clone();
        let form_field_c_setter = form_field_c_setter.clone();
        let form_notes_setter = form_notes_setter.clone();
        async move {
            if let Some(item) = loaded {
                let (label, title, a, b, c, notes) = decompose_item(&item);
                form_type_setter.set(label);
                form_title_setter.set(title);
                form_field_a_setter.set(a);
                form_field_b_setter.set(b);
                form_field_c_setter.set(c);
                form_notes_setter.set(notes);
            }
        }
    });

    let on_save = move |_| {
        if *submitting.get() {
            return;
        }
        let item_type_label = form_type.get().clone();
        let title = form_title.get().clone();
        let field_a = form_field_a.get().clone();
        let field_b = form_field_b.get().clone();
        let field_c = form_field_c.get().clone();
        let notes = form_notes.get().clone();

        let item_type = match build_item_type(
            &item_type_label,
            &title,
            &field_a,
            &field_b,
            &field_c,
            &notes,
        ) {
            Ok(item) => item,
            Err(error) => {
                form_error.set(Some(error));
                return;
            }
        };

        submitting.set(true);
        form_error.set(None);
        let state_handle = state_ref.clone();
        let error_handle = form_error.clone();
        let submitting_handle = submitting.clone();
        let is_new_for_save = is_new;
        let item_id_for_save = item_id;
        cx.spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let handle_opt = state_handle.with(|s| s.unlocked_handle.as_ref().cloned());
                let Some(handle) = handle_opt else {
                    error_handle.set(Some("Vault is locked".to_string()));
                    submitting_handle.set(false);
                    return;
                };
                let result = if is_new_for_save {
                    handle.create_item(item_type).await
                } else if let Some(id) = item_id_for_save {
                    handle.update_item(id, item_type).await
                } else {
                    Err(VaultStoreError::ItemNotFound)
                };
                match result {
                    Ok(item) => {
                        let new_id = item.id;
                        let items_handle = handle.clone();
                        let items = match items_handle.list_items().await {
                            Ok(items) => items,
                            Err(error) => {
                                error_handle.set(Some(format!("Could not list items: {error}")));
                                submitting_handle.set(false);
                                return;
                            }
                        };
                        state_handle.with_mut(|s| {
                            s.items = items;
                            s.status = Some("Item saved".to_string());
                            s.screen = Screen::Detail(new_id);
                            s.current_item = None;
                        });
                    }
                    Err(error) => {
                        error_handle.set(Some(format!("Could not save item: {error}")));
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = (state_handle, item_id_for_save, is_new_for_save, item_type);
                error_handle.set(Some("WASM backend not available".to_string()));
            }
            submitting_handle.set(false);
        });
    };

    let on_copy_password = move |_| {
        let state = state_ref.clone();
        let error_handle = form_error.clone();
        cx.spawn(async move {
            let value: Option<String> = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let field = "password".to_string();
                    let handle_opt = state.with(|s| s.unlocked_handle.as_ref().cloned());
                    if let Some(handle) = handle_opt {
                        if let Some(id) = state.with(|s| match s.screen {
                            Screen::Detail(id) => Some(id),
                            _ => None,
                        }) {
                            if let Ok(item) = handle.get_item(id).await {
                                item.data.get_field(&field).ok()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    None
                }
            };
            if let Some(value) = value {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Err(error) = crate::utils::clipboard::copy_to_clipboard(&value) {
                        error_handle.set(Some(format!("Clipboard error: {error}")));
                    } else {
                        state.with_mut(|s| {
                            s.status = Some("Copied to clipboard".to_string());
                        });
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = value;
                }
            } else {
                error_handle.set(Some("Could not copy".to_string()));
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = state;
            }
        });
    };

    let on_delete_confirmed = move |_| {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let state = state_ref.clone();
            let submitting_handle = submitting.clone();
            let error_handle = form_error.clone();
            show_delete_confirm.set(false);
            let item_id_for_delete = item_id;
            cx.spawn(async move {
                submitting_handle.set(true);
                if let Some(id) = item_id_for_delete {
                    let handle_opt = state.with(|s| s.unlocked_handle.as_ref().cloned());
                    if let Some(handle) = handle_opt {
                        match handle.delete_item(id).await {
                            Ok(()) => {
                                state.with_mut(|s| {
                                    s.items.retain(|i| i.id != id);
                                    s.status = Some("Item deleted".to_string());
                                    s.screen = Screen::List;
                                });
                            }
                            Err(error) => {
                                error_handle.set(Some(format!("Could not delete item: {error}")));
                            }
                        }
                    } else {
                        error_handle.set(Some("Vault is locked".to_string()));
                    }
                }
                submitting_handle.set(false);
            });
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = item_id;
        }
    };

    let on_back = move |_| {
        let state = state_ref.clone();
        state.with_mut(|s| {
            s.current_item = None;
            s.screen = Screen::List;
        });
    };

    let type_label = form_type.get().clone();
    let title = form_title.get().clone();
    let field_a = form_field_a.get().clone();
    let field_b = form_field_b.get().clone();
    let field_c = form_field_c.get().clone();
    let notes = form_notes.get().clone();
    let form_type_setter = form_type.clone();
    let form_title_setter = form_title.clone();
    let form_field_a_setter = form_field_a.clone();
    let form_field_b_setter = form_field_b.clone();
    let form_field_c_setter = form_field_c.clone();
    let form_notes_setter = form_notes.clone();
    let form_error_get = form_error.get().clone();

    let heading = if is_new || title.trim().is_empty() {
        "New item".to_string()
    } else {
        title.clone()
    };

    cx.render(rsx! {
        section { class: "screen", id: "detail",
            div { class: "screen-header split",
                div {
                    p { class: "eyebrow", if is_new { "New item" } else { "Item detail" } }
                    h1 { "{heading}" }
                }
                div { class: "actions",
                    Button { label: "Back", variant: "btn-secondary", on_click: on_back }
                }
            }
            if !is_unlocked {
                rsx! {
                    div { class: "panel",
                        p { class: "muted", "Vault is locked. Unlock a vault first to create or edit items." }
                    }
                }
            } else {
                rsx! {
                    form { class: "panel form-grid two-col",
                        label { class: "field",
                            span { "Type" }
                            select {
                                class: "input",
                                value: "{type_label}",
                                onchange: move |event| form_type_setter.set(event.value.clone()),
                                option { value: "Login", "Login" }
                                option { value: "APIKey", "APIKey" }
                                option { value: "SSHKey", "SSHKey" }
                                option { value: "SecureNote", "SecureNote" }
                                option { value: "Server", "Server" }
                                option { value: "Database", "Database" }
                                option { value: "Identity", "Identity" }
                                option { value: "SoftwareLicense", "SoftwareLicense" }
                                option { value: "RecoveryCodes", "RecoveryCodes" }
                                option { value: "Custom", "Custom" }
                            }
                        }
                        TextInput {
                            label: "Title / Name",
                            value: "{title}",
                            placeholder: "Display name for this item",
                            on_input: move |value: String| form_title_setter.set(value)
                        }
                        if type_label == "Login" {
                            rsx! {
                                TextInput { label: "Username", value: "{field_a}", on_input: move |value: String| form_field_a_setter.set(value) }
                                PasswordInput { label: "Password", value: "{field_b}", on_input: move |value: String| form_field_b_setter.set(value), auto_complete: "off" }
                                TextInput { label: "URL", value: "{field_c}", on_input: move |value: String| form_field_c_setter.set(value) }
                            }
                        } else if type_label == "APIKey" {
                            rsx! {
                                TextInput { label: "Provider", value: "{field_a}", on_input: move |value: String| form_field_a_setter.set(value) }
                                PasswordInput { label: "Key", value: "{field_b}", on_input: move |value: String| form_field_b_setter.set(value), auto_complete: "off" }
                            }
                        } else if type_label == "SSHKey" {
                            rsx! {
                                TextInput { label: "Public key", value: "{field_a}", on_input: move |value: String| form_field_a_setter.set(value) }
                                PasswordInput { label: "Private key", value: "{field_b}", on_input: move |value: String| form_field_b_setter.set(value), auto_complete: "off" }
                                TextInput { label: "Passphrase (optional)", value: "{field_c}", on_input: move |value: String| form_field_c_setter.set(value) }
                            }
                        } else if type_label == "SecureNote" {
                            rsx! {
                                label { class: "field full",
                                    span { "Content" }
                                    textarea {
                                        class: "input textarea",
                                        value: "{field_a}",
                                        oninput: move |event| form_field_a_setter.set(event.value.clone())
                                    }
                                }
                            }
                        } else if type_label == "Server" {
                            rsx! {
                                TextInput { label: "Hostname", value: "{field_a}", on_input: move |value: String| form_field_a_setter.set(value) }
                                TextInput { label: "Port", value: "{field_b}", on_input: move |value: String| form_field_b_setter.set(value) }
                                TextInput { label: "Username", value: "{field_c}", on_input: move |value: String| form_field_c_setter.set(value) }
                            }
                        } else if type_label == "Database" {
                            rsx! {
                                TextInput { label: "Engine", value: "{field_a}", on_input: move |value: String| form_field_a_setter.set(value) }
                                TextInput { label: "Host", value: "{field_b}", on_input: move |value: String| form_field_b_setter.set(value) }
                                TextInput { label: "Database", value: "{field_c}", on_input: move |value: String| form_field_c_setter.set(value) }
                            }
                        } else if type_label == "Identity" {
                            rsx! {
                                TextInput { label: "Email", value: "{field_a}", on_input: move |value: String| form_field_a_setter.set(value) }
                                TextInput { label: "Phone", value: "{field_b}", on_input: move |value: String| form_field_b_setter.set(value) }
                                TextInput { label: "Address", value: "{field_c}", on_input: move |value: String| form_field_c_setter.set(value) }
                            }
                        } else if type_label == "SoftwareLicense" {
                            rsx! {
                                TextInput { label: "Product", value: "{field_a}", on_input: move |value: String| form_field_a_setter.set(value) }
                                PasswordInput { label: "License key", value: "{field_b}", on_input: move |value: String| form_field_b_setter.set(value), auto_complete: "off" }
                            }
                        } else if type_label == "RecoveryCodes" {
                            rsx! {
                                label { class: "field full",
                                    span { "Codes (comma separated)" }
                                    textarea {
                                        class: "input textarea",
                                        value: "{field_a}",
                                        oninput: move |event| form_field_a_setter.set(event.value.clone())
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                TextInput { label: "Value", value: "{field_a}", on_input: move |value: String| form_field_a_setter.set(value) }
                            }
                        }
                        label { class: "field full",
                            span { "Notes" }
                            textarea {
                                class: "input textarea",
                                value: "{notes}",
                                oninput: move |event| form_notes_setter.set(event.value.clone())
                            }
                        }
                        form_error_get.as_ref().map(|err| rsx! {
                            div { class: "inline-error", role: "alert", "{err}" }
                        }),
                        div { class: "actions full",
                            Button { label: "Save", variant: "btn-primary", disabled: *submitting.get(), on_click: on_save }
                            if !is_new {
                                rsx! {
                                    Button { label: "Copy password", variant: "btn-secondary", on_click: on_copy_password }
                                    Button { label: "Delete", variant: "btn-danger", on_click: move |_| show_delete_confirm.set(true) }
                                }
                            }
                        }
                    }
                }
            }
            if *show_delete_confirm.get() {
                rsx! {
                    Modal {
                        title: "Delete item",
                        message: "This will permanently remove the item from the vault. Are you sure?",
                        confirm_label: "Delete",
                        cancel_label: "Cancel",
                        danger: true,
                        on_confirm: on_delete_confirmed,
                        on_cancel: move |_| show_delete_confirm.set(false)
                    }
                }
            }
        }
    })
}

#[allow(unused_assignments)]
fn decompose_item(item: &DecryptedItem) -> (String, String, String, String, String, String) {
    let mut label = String::new();
    let mut title = String::new();
    let mut a = String::new();
    let mut b = String::new();
    let mut c = String::new();
    let mut notes = String::new();
    match &item.data {
        ItemType::Login(secret) => {
            label = "Login".to_string();
            title = secret.username.clone();
            a = secret.username.clone();
            b = secret.password.clone();
            c = secret.url.clone().unwrap_or_default();
            notes = secret.notes.clone().unwrap_or_default();
        }
        ItemType::APIKey(secret) => {
            label = "APIKey".to_string();
            title = secret.name.clone();
            a = secret.provider.clone();
            b = secret.key.clone();
        }
        ItemType::SSHKey(secret) => {
            label = "SSHKey".to_string();
            title = secret.name.clone();
            a = secret.public_key.clone();
            b = secret.private_key.clone();
            c = secret.passphrase.clone().unwrap_or_default();
            notes = secret.comment.clone().unwrap_or_default();
        }
        ItemType::SecureNote(secret) => {
            label = "SecureNote".to_string();
            title = secret.title.clone();
            a = secret.content.clone();
        }
        ItemType::Server(secret) => {
            label = "Server".to_string();
            title = secret.hostname.clone();
            a = secret.hostname.clone();
            b = secret.port.to_string();
            c = secret.username.clone();
            notes = secret.notes.clone().unwrap_or_default();
        }
        ItemType::Database(secret) => {
            label = "Database".to_string();
            title = format!("{}/{}", secret.engine, secret.database);
            a = secret.engine.clone();
            b = secret.host.clone();
            c = secret.database.clone();
        }
        ItemType::Identity(secret) => {
            label = "Identity".to_string();
            title = secret.name.clone();
            a = secret.email.clone();
            b = secret.phone.clone().unwrap_or_default();
            c = secret.address.clone().unwrap_or_default();
        }
        ItemType::SoftwareLicense(secret) => {
            label = "SoftwareLicense".to_string();
            title = secret.product.clone();
            a = secret.product.clone();
            b = secret.key.clone();
        }
        ItemType::RecoveryCodes(secret) => {
            label = "RecoveryCodes".to_string();
            title = format!("{} recovery codes", secret.codes.len());
            a = secret.codes.join(",");
        }
        ItemType::Custom(secret) => {
            label = "Custom".to_string();
            if let Some((key, value)) = secret.fields.iter().next() {
                title = key.clone();
                a = value.clone();
            }
        }
    }
    (label, title, a, b, c, notes)
}

fn build_item_type(
    type_label: &str,
    title: &str,
    field_a: &str,
    field_b: &str,
    field_c: &str,
    notes: &str,
) -> std::result::Result<ItemType, String> {
    match type_label {
        "Login" => Ok(ItemType::Login(LoginSecret {
            username: title.to_string(),
            password: field_b.to_string(),
            url: if field_c.is_empty() {
                None
            } else {
                Some(field_c.to_string())
            },
            notes: if notes.is_empty() {
                None
            } else {
                Some(notes.to_string())
            },
        })),
        "APIKey" => Ok(ItemType::APIKey(APIKeySecret {
            name: title.to_string(),
            key: field_b.to_string(),
            provider: field_a.to_string(),
        })),
        "SSHKey" => Ok(ItemType::SSHKey(SSHKeySecret {
            name: title.to_string(),
            public_key: field_a.to_string(),
            private_key: field_b.to_string(),
            passphrase: if field_c.is_empty() {
                None
            } else {
                Some(field_c.to_string())
            },
            comment: if notes.is_empty() {
                None
            } else {
                Some(notes.to_string())
            },
            allowed_hosts: vec![],
            require_confirmation: false,
        })),
        "SecureNote" => Ok(ItemType::SecureNote(SecureNoteSecret {
            title: title.to_string(),
            content: field_a.to_string(),
        })),
        "Server" => {
            let port: u16 = field_b
                .parse()
                .map_err(|_| "Port must be a number".to_string())?;
            Ok(ItemType::Server(ServerSecret {
                hostname: field_a.to_string(),
                port,
                username: field_c.to_string(),
                password: None,
                notes: if notes.is_empty() {
                    None
                } else {
                    Some(notes.to_string())
                },
            }))
        }
        "Database" => Ok(ItemType::Database(DatabaseSecret {
            engine: field_a.to_string(),
            host: field_b.to_string(),
            port: 0,
            username: String::new(),
            password: String::new(),
            database: field_c.to_string(),
        })),
        "Identity" => Ok(ItemType::Identity(IdentitySecret {
            name: title.to_string(),
            email: field_a.to_string(),
            phone: if field_b.is_empty() {
                None
            } else {
                Some(field_b.to_string())
            },
            address: if field_c.is_empty() {
                None
            } else {
                Some(field_c.to_string())
            },
        })),
        "SoftwareLicense" => Ok(ItemType::SoftwareLicense(SoftwareLicenseSecret {
            product: field_a.to_string(),
            key: field_b.to_string(),
            version: None,
            expiry: None,
        })),
        "RecoveryCodes" => {
            let codes: Vec<String> = field_a
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
            Ok(ItemType::RecoveryCodes(RecoveryCodesSecret { codes }))
        }
        "Custom" => {
            let mut fields = std::collections::HashMap::new();
            fields.insert(title.to_string(), field_a.to_string());
            Ok(ItemType::Custom(CustomSecret { fields }))
        }
        _ => Err(format!("Unknown item type: {type_label}")),
    }
}

#[derive(Props, PartialEq)]
pub struct ItemDetailPageProps {
    pub state: UseRef<AppState>,
}
