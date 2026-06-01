use serde::{Deserialize, Serialize};
use std::fmt;

const PIE_SCHEME: &str = "pie://";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PieUri {
    pub vault_name: String,
    pub item_name: String,
    pub field_name: String,
}

impl PieUri {
    pub fn parse(input: &str) -> Result<Self, PieUriError> {
        let path = input
            .strip_prefix(PIE_SCHEME)
            .ok_or(PieUriError::MissingScheme)?;

        let parts: Vec<&str> = path.splitn(3, '/').collect();
        if parts.len() != 3 {
            return Err(PieUriError::InvalidFormat);
        }

        let vault_name = parts[0].to_string();
        let item_name = parts[1].to_string();
        let field_name = parts[2].to_string();

        if vault_name.is_empty() {
            return Err(PieUriError::EmptyVaultName);
        }
        if item_name.is_empty() {
            return Err(PieUriError::EmptyItemName);
        }
        if field_name.is_empty() {
            return Err(PieUriError::EmptyFieldName);
        }

        Ok(Self {
            vault_name,
            item_name,
            field_name,
        })
    }

    pub fn to_string_redacted(&self) -> String {
        format!("pie://{}/{}/[redacted]", self.vault_name, self.item_name)
    }
}

impl fmt::Display for PieUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pie://{}/{}/{}",
            self.vault_name, self.item_name, self.field_name
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PieUriError {
    MissingScheme,
    InvalidFormat,
    EmptyVaultName,
    EmptyItemName,
    EmptyFieldName,
}

impl fmt::Display for PieUriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingScheme => write!(f, "URI must start with 'pie://'"),
            Self::InvalidFormat => {
                write!(f, "URI must be in format 'pie://vault/item/field'")
            }
            Self::EmptyVaultName => write!(f, "vault name cannot be empty"),
            Self::EmptyItemName => write!(f, "item name cannot be empty"),
            Self::EmptyFieldName => write!(f, "field name cannot be empty"),
        }
    }
}

impl std::error::Error for PieUriError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_uri() {
        let uri = PieUri::parse("pie://Personal/GitHub/password").unwrap();
        assert_eq!(uri.vault_name, "Personal");
        assert_eq!(uri.item_name, "GitHub");
        assert_eq!(uri.field_name, "password");
    }

    #[test]
    fn parse_missing_scheme() {
        let result = PieUri::parse("Personal/GitHub/password");
        assert!(matches!(result, Err(PieUriError::MissingScheme)));
    }

    #[test]
    fn parse_invalid_format() {
        let result = PieUri::parse("pie://Personal/GitHub");
        assert!(matches!(result, Err(PieUriError::InvalidFormat)));
    }

    #[test]
    fn parse_empty_vault() {
        let result = PieUri::parse("pie:///GitHub/password");
        assert!(matches!(result, Err(PieUriError::EmptyVaultName)));
    }

    #[test]
    fn parse_empty_item() {
        let result = PieUri::parse("pie://Personal//password");
        assert!(matches!(result, Err(PieUriError::EmptyItemName)));
    }

    #[test]
    fn parse_empty_field() {
        let result = PieUri::parse("pie://Personal/GitHub/");
        assert!(matches!(result, Err(PieUriError::EmptyFieldName)));
    }

    #[test]
    fn display_format() {
        let uri = PieUri {
            vault_name: "Personal".to_string(),
            item_name: "GitHub".to_string(),
            field_name: "password".to_string(),
        };
        assert_eq!(uri.to_string(), "pie://Personal/GitHub/password");
    }

    #[test]
    fn redacted_display() {
        let uri = PieUri {
            vault_name: "Personal".to_string(),
            item_name: "GitHub".to_string(),
            field_name: "password".to_string(),
        };
        assert_eq!(uri.to_string_redacted(), "pie://Personal/GitHub/[redacted]");
    }
}
