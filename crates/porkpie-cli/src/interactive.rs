use crate::errors::{CliError, Result};
use dialoguer::{Confirm, Input, Password, Select};
use porkpie_core::Item;
use porkpie_types::{
    APIKeySecret, CustomSecret, DatabaseSecret, IdentitySecret, ItemType, LoginSecret,
    RecoveryCodesSecret, SSHKeySecret, SecureNoteSecret, ServerSecret, SoftwareLicenseSecret,
};
use std::collections::HashMap;

const MIN_MASTER_PASSWORD_LEN: usize = 16;

/// Prompt for a new master password and confirmation.
pub fn prompt_new_master_password() -> Result<String> {
    let password = Password::new()
        .with_prompt("Master password")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()?;

    if password.len() < MIN_MASTER_PASSWORD_LEN {
        return Err(CliError::PasswordTooShort {
            min: MIN_MASTER_PASSWORD_LEN,
        });
    }

    Ok(password)
}

/// Prompt for an existing master password.
pub fn prompt_master_password() -> Result<String> {
    Ok(Password::new().with_prompt("Master password").interact()?)
}

/// Prompt for a vault id.
pub fn prompt_vault_id() -> Result<String> {
    Ok(Input::<String>::new()
        .with_prompt("Vault ID")
        .interact_text()?)
}

/// Prompt for a vault name.
pub fn prompt_vault_name() -> Result<String> {
    let name = Input::<String>::new()
        .with_prompt("Vault name")
        .interact_text()?;
    if name.trim().is_empty() {
        return Err(CliError::InvalidArgument(
            "vault name cannot be empty".to_string(),
        ));
    }
    Ok(name.trim().to_string())
}

/// Prompt for item data using a requested type name.
pub fn prompt_item(item_type: &str) -> Result<Item> {
    Ok(Item::new(prompt_item_type(item_type, None)?))
}

/// Prompt to edit an existing item while preserving its item type.
pub fn prompt_updated_item(existing: &Item) -> Result<Item> {
    let type_name = item_type_name(&existing.data);
    Ok(Item::new(prompt_item_type(
        type_name,
        Some(&existing.data),
    )?))
}

/// Confirm a destructive operation.
pub fn confirm_delete(id: &str) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(format!("Delete item {id}?"))
        .default(false)
        .interact()?)
}

/// Confirm a dangerous plaintext export.
pub fn confirm_dangerous_plaintext_export() -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(
            "WARNING: This will write ALL decrypted secrets to a plaintext JSON file. \
Are you sure you want to proceed?",
        )
        .default(false)
        .interact()?)
}

/// Return a stable display name for an item type.
pub fn item_type_name(item_type: &ItemType) -> &'static str {
    match item_type {
        ItemType::Login(_) => "Login",
        ItemType::APIKey(_) => "APIKey",
        ItemType::SSHKey(_) => "SSHKey",
        ItemType::SecureNote(_) => "SecureNote",
        ItemType::Server(_) => "Server",
        ItemType::Database(_) => "Database",
        ItemType::Identity(_) => "Identity",
        ItemType::SoftwareLicense(_) => "SoftwareLicense",
        ItemType::RecoveryCodes(_) => "RecoveryCodes",
        ItemType::Custom(_) => "Custom",
    }
}

/// Return a compact item title for list output.
pub fn item_title(item_type: &ItemType) -> &str {
    match item_type {
        ItemType::Login(secret) => &secret.username,
        ItemType::APIKey(secret) => &secret.name,
        ItemType::SSHKey(secret) => &secret.name,
        ItemType::SecureNote(secret) => &secret.title,
        ItemType::Server(secret) => &secret.hostname,
        ItemType::Database(secret) => &secret.database,
        ItemType::Identity(secret) => &secret.name,
        ItemType::SoftwareLicense(secret) => &secret.product,
        ItemType::RecoveryCodes(_) => "Recovery codes",
        ItemType::Custom(_) => "Custom item",
    }
}

fn prompt_item_type(item_type: &str, existing: Option<&ItemType>) -> Result<ItemType> {
    match normalize_item_type(item_type).as_str() {
        "login" => prompt_login(existing),
        "apikey" | "api-key" | "api_key" => prompt_api_key(existing),
        "sshkey" | "ssh-key" | "ssh_key" => prompt_ssh_key(existing),
        "securenote" | "secure-note" | "secure_note" | "note" => prompt_secure_note(existing),
        "server" => prompt_server(existing),
        "database" | "db" => prompt_database(existing),
        "identity" => prompt_identity(existing),
        "softwarelicense" | "software-license" | "software_license" | "license" => {
            prompt_software_license(existing)
        }
        "recoverycodes" | "recovery-codes" | "recovery_codes" => prompt_recovery_codes(existing),
        "custom" => prompt_custom(existing),
        other => Err(CliError::UnsupportedItemType(other.to_string())),
    }
}

fn normalize_item_type(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(' ', "")
}

fn prompt_login(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing = match existing {
        Some(ItemType::Login(secret)) => Some(secret),
        _ => None,
    };
    Ok(ItemType::Login(LoginSecret {
        username: prompt_string("Username", existing.map(|s| s.username.as_str()))?,
        password: prompt_string("Password", existing.map(|s| s.password.as_str()))?,
        url: prompt_optional("URL", existing.and_then(|s| s.url.as_deref()))?,
        notes: prompt_optional("Notes", existing.and_then(|s| s.notes.as_deref()))?,
    }))
}

fn prompt_api_key(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing = match existing {
        Some(ItemType::APIKey(secret)) => Some(secret),
        _ => None,
    };
    Ok(ItemType::APIKey(APIKeySecret {
        name: prompt_string("Name", existing.map(|s| s.name.as_str()))?,
        key: prompt_string("Key", existing.map(|s| s.key.as_str()))?,
        provider: prompt_string("Provider", existing.map(|s| s.provider.as_str()))?,
    }))
}

fn prompt_ssh_key(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing = match existing {
        Some(ItemType::SSHKey(secret)) => Some(secret),
        _ => None,
    };
    let allowed_hosts_default = existing.map(|s| s.allowed_hosts.join(","));
    Ok(ItemType::SSHKey(SSHKeySecret {
        name: prompt_string("Name", existing.map(|s| s.name.as_str()))?,
        public_key: prompt_string("Public key", existing.map(|s| s.public_key.as_str()))?,
        private_key: prompt_string("Private key", existing.map(|s| s.private_key.as_str()))?,
        passphrase: prompt_optional("Passphrase", existing.and_then(|s| s.passphrase.as_deref()))?,
        comment: prompt_optional("Comment", existing.and_then(|s| s.comment.as_deref()))?,
        allowed_hosts: prompt_string(
            "Allowed hosts (comma separated)",
            allowed_hosts_default.as_deref(),
        )?
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect(),
    }))
}

fn prompt_secure_note(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing = match existing {
        Some(ItemType::SecureNote(secret)) => Some(secret),
        _ => None,
    };
    Ok(ItemType::SecureNote(SecureNoteSecret {
        title: prompt_string("Title", existing.map(|s| s.title.as_str()))?,
        content: prompt_string("Content", existing.map(|s| s.content.as_str()))?,
    }))
}

fn prompt_server(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing = match existing {
        Some(ItemType::Server(secret)) => Some(secret),
        _ => None,
    };
    Ok(ItemType::Server(ServerSecret {
        hostname: prompt_string("Hostname", existing.map(|s| s.hostname.as_str()))?,
        port: prompt_u16("Port", existing.map(|s| s.port))?,
        username: prompt_string("Username", existing.map(|s| s.username.as_str()))?,
        password: prompt_optional("Password", existing.and_then(|s| s.password.as_deref()))?,
        notes: prompt_optional("Notes", existing.and_then(|s| s.notes.as_deref()))?,
    }))
}

fn prompt_database(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing = match existing {
        Some(ItemType::Database(secret)) => Some(secret),
        _ => None,
    };
    Ok(ItemType::Database(DatabaseSecret {
        engine: prompt_string("Engine", existing.map(|s| s.engine.as_str()))?,
        host: prompt_string("Host", existing.map(|s| s.host.as_str()))?,
        port: prompt_u16("Port", existing.map(|s| s.port))?,
        username: prompt_string("Username", existing.map(|s| s.username.as_str()))?,
        password: prompt_string("Password", existing.map(|s| s.password.as_str()))?,
        database: prompt_string("Database", existing.map(|s| s.database.as_str()))?,
    }))
}

fn prompt_identity(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing = match existing {
        Some(ItemType::Identity(secret)) => Some(secret),
        _ => None,
    };
    Ok(ItemType::Identity(IdentitySecret {
        name: prompt_string("Name", existing.map(|s| s.name.as_str()))?,
        email: prompt_string("Email", existing.map(|s| s.email.as_str()))?,
        phone: prompt_optional("Phone", existing.and_then(|s| s.phone.as_deref()))?,
        address: prompt_optional("Address", existing.and_then(|s| s.address.as_deref()))?,
    }))
}

fn prompt_software_license(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing = match existing {
        Some(ItemType::SoftwareLicense(secret)) => Some(secret),
        _ => None,
    };
    Ok(ItemType::SoftwareLicense(SoftwareLicenseSecret {
        product: prompt_string("Product", existing.map(|s| s.product.as_str()))?,
        key: prompt_string("Key", existing.map(|s| s.key.as_str()))?,
        version: prompt_optional("Version", existing.and_then(|s| s.version.as_deref()))?,
        expiry: prompt_optional("Expiry", existing.and_then(|s| s.expiry.as_deref()))?,
    }))
}

fn prompt_recovery_codes(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing_codes = match existing {
        Some(ItemType::RecoveryCodes(secret)) => Some(secret.codes.join(",")),
        _ => None,
    };
    let raw_codes = prompt_string("Codes (comma separated)", existing_codes.as_deref())?;
    let codes = raw_codes
        .split(',')
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    Ok(ItemType::RecoveryCodes(RecoveryCodesSecret { codes }))
}

fn prompt_custom(existing: Option<&ItemType>) -> Result<ItemType> {
    let existing_fields = match existing {
        Some(ItemType::Custom(secret)) => {
            let serialized = secret
                .fields
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(",");
            Some(serialized)
        }
        _ => None,
    };
    let raw_fields = prompt_string(
        "Fields (key=value comma separated)",
        existing_fields.as_deref(),
    )?;
    let mut fields = HashMap::new();
    for pair in raw_fields
        .split(',')
        .map(str::trim)
        .filter(|pair| !pair.is_empty())
    {
        if let Some((key, value)) = pair.split_once('=') {
            fields.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(ItemType::Custom(CustomSecret { fields }))
}

fn prompt_string(prompt: &str, default: Option<&str>) -> Result<String> {
    let mut input = Input::<String>::new().with_prompt(prompt.to_string());
    if let Some(default) = default {
        input = input.default(default.to_string());
    }
    Ok(input.interact_text()?)
}

fn prompt_optional(prompt: &str, default: Option<&str>) -> Result<Option<String>> {
    let value = prompt_string(prompt, default)?;
    Ok((!value.trim().is_empty()).then_some(value))
}

fn prompt_u16(prompt: &str, default: Option<u16>) -> Result<u16> {
    let mut input = Input::<u16>::new().with_prompt(prompt.to_string());
    if let Some(default) = default {
        input = input.default(default);
    }
    Ok(input.interact_text()?)
}

/// Ask the user to choose a supported item type.
pub fn select_item_type() -> Result<&'static str> {
    const TYPES: &[&str] = &[
        "Login",
        "APIKey",
        "SSHKey",
        "SecureNote",
        "Server",
        "Database",
        "Identity",
        "SoftwareLicense",
        "RecoveryCodes",
        "Custom",
    ];
    let selection = Select::new().items(TYPES).default(0).interact()?;
    Ok(TYPES[selection])
}
