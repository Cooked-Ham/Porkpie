use crate::errors::{ImportError, Result};
use crate::validators::{normalize_item_type, required};
use porkpie_core::EncryptedItemData;
use porkpie_core::{Item, Vault};
use porkpie_types::{APIKeySecret, ItemType, LoginSecret, SecureNoteSecret};
use serde::Deserialize;
use std::io::Read;

/// Result of importing CSV rows into an unlocked vault.
#[derive(Debug, Clone)]
pub struct CsvImportResult {
    pub imported: usize,
    pub encrypted_items: Vec<EncryptedItemData>,
}

/// Parse CSV rows, create vault items, and return encrypted store records.
pub fn import_csv_reader<R: Read>(reader: R, vault: &mut Vault) -> Result<CsvImportResult> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(reader);
    let mut encrypted_items = Vec::new();

    for (index, row) in reader.deserialize::<CsvRow>().enumerate() {
        let row_number = index.saturating_add(2);
        let row = row.map_err(ImportError::Csv)?;
        let item = row
            .try_into_item()
            .map_err(|error| ImportError::InvalidRow {
                row: row_number,
                message: error.to_string(),
            })?;
        encrypted_items.push(create_encrypted_item(vault, item)?);
    }

    Ok(CsvImportResult {
        imported: encrypted_items.len(),
        encrypted_items,
    })
}

/// Import already parsed CSV records using the required column order.
pub fn import_csv_records(records: &[Vec<String>], vault: &mut Vault) -> Result<CsvImportResult> {
    let mut csv_data = String::from("item_type,title,username,password,notes\n");
    for record in records {
        let serialized = record
            .iter()
            .map(|field| {
                let escaped = field.replace('"', "\"\"");
                format!("\"{escaped}\"")
            })
            .collect::<Vec<_>>()
            .join(",");
        csv_data.push_str(&serialized);
        csv_data.push('\n');
    }

    import_csv_reader(csv_data.as_bytes(), vault)
}

#[derive(Debug, Deserialize)]
struct CsvRow {
    item_type: String,
    title: String,
    username: String,
    password: String,
    notes: String,
}

impl CsvRow {
    fn try_into_item(self) -> Result<Item> {
        let item_type = normalize_item_type(&self.item_type);
        let title = required(&self.title, "title")?;

        let data = match item_type.as_str() {
            "login" => ItemType::Login(LoginSecret {
                username: required(&self.username, "username")?,
                password: required(&self.password, "password")?,
                url: None,
                notes: optional_with_title(title, self.notes),
            }),
            "securenote" | "note" => ItemType::SecureNote(SecureNoteSecret {
                title,
                content: required(&self.notes, "notes")?,
            }),
            "apikey" => ItemType::APIKey(APIKeySecret {
                name: title,
                key: required(&self.password, "password")?,
                provider: optional_text(&self.username).unwrap_or_else(|| "imported".to_string()),
            }),
            other => return Err(ImportError::UnsupportedItemType(other.to_string())),
        };

        Ok(Item::new(data))
    }
}

fn create_encrypted_item(vault: &mut Vault, item: Item) -> Result<EncryptedItemData> {
    let item_id = vault.create_item(item)?;
    let stored_item = vault.get_item(item_id)?.clone();
    let ciphertext = vault.encrypt_item(&stored_item)?;
    Ok(EncryptedItemData::new(
        item_id,
        vault.id,
        item_type_name(&stored_item.data),
        ciphertext,
        stored_item.created_at,
        stored_item.updated_at,
        vault.sync_revision(),
    ))
}

fn item_type_name(item_type: &ItemType) -> &'static str {
    match item_type {
        ItemType::Login(_) => "Login",
        ItemType::APIKey(_) => "APIKey",
        ItemType::SecureNote(_) => "SecureNote",
        ItemType::SSHKey(_) => "SSHKey",
        ItemType::Server(_) => "Server",
        ItemType::Database(_) => "Database",
        ItemType::Identity(_) => "Identity",
        ItemType::SoftwareLicense(_) => "SoftwareLicense",
        ItemType::RecoveryCodes(_) => "RecoveryCodes",
        ItemType::Custom(_) => "Custom",
    }
}

fn optional_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn optional_with_title(title: String, notes: String) -> Option<String> {
    let notes = optional_text(&notes)?;
    Some(format!("Title: {title}\n{notes}"))
}
