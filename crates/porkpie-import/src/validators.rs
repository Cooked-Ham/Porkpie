use crate::errors::{ImportError, Result};

/// Validate that a required field is not empty.
pub fn required(value: &str, field: &'static str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(ImportError::MissingField(field))
    } else {
        Ok(trimmed.to_string())
    }
}

/// Normalize CSV item type names for importer matching.
pub fn normalize_item_type(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '_', '-'], "")
}
