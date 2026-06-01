use crate::item_type::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldError {
    FieldNotFound(String),
    InvalidFieldValue(String),
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldNotFound(name) => write!(f, "field '{}' not found", name),
            Self::InvalidFieldValue(msg) => write!(f, "invalid field value: {}", msg),
        }
    }
}

impl std::error::Error for FieldError {}

impl ItemType {
    pub fn get_field(&self, field_name: &str) -> Result<String, FieldError> {
        match self {
            ItemType::Login(s) => match field_name {
                "username" => Ok(s.username.clone()),
                "password" => Ok(s.password.clone()),
                "url" => Ok(s.url.clone().unwrap_or_default()),
                "notes" => Ok(s.notes.clone().unwrap_or_default()),
                _ => Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::APIKey(s) => match field_name {
                "name" => Ok(s.name.clone()),
                "key" => Ok(s.key.clone()),
                "provider" => Ok(s.provider.clone()),
                _ => Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::SSHKey(s) => match field_name {
                "name" => Ok(s.name.clone()),
                "public_key" => Ok(s.public_key.clone()),
                "private_key" => Ok(s.private_key.clone()),
                "passphrase" => Ok(s.passphrase.clone().unwrap_or_default()),
                "comment" => Ok(s.comment.clone().unwrap_or_default()),
                "allowed_hosts" => Ok(s.allowed_hosts.join(",")),
                _ => Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::SecureNote(s) => match field_name {
                "title" => Ok(s.title.clone()),
                "content" => Ok(s.content.clone()),
                _ => Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::Server(s) => match field_name {
                "hostname" => Ok(s.hostname.clone()),
                "port" => Ok(s.port.to_string()),
                "username" => Ok(s.username.clone()),
                "password" => Ok(s.password.clone().unwrap_or_default()),
                "notes" => Ok(s.notes.clone().unwrap_or_default()),
                _ => Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::Database(s) => match field_name {
                "engine" => Ok(s.engine.clone()),
                "host" => Ok(s.host.clone()),
                "port" => Ok(s.port.to_string()),
                "username" => Ok(s.username.clone()),
                "password" => Ok(s.password.clone()),
                "database" => Ok(s.database.clone()),
                _ => Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::Identity(s) => match field_name {
                "name" => Ok(s.name.clone()),
                "email" => Ok(s.email.clone()),
                "phone" => Ok(s.phone.clone().unwrap_or_default()),
                "address" => Ok(s.address.clone().unwrap_or_default()),
                _ => Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::SoftwareLicense(s) => match field_name {
                "product" => Ok(s.product.clone()),
                "key" => Ok(s.key.clone()),
                "version" => Ok(s.version.clone().unwrap_or_default()),
                "expiry" => Ok(s.expiry.clone().unwrap_or_default()),
                _ => Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::RecoveryCodes(s) => match field_name {
                "codes" => Ok(s.codes.join(",")),
                _ => {
                    if let Some(idx) = field_name.strip_prefix("code_") {
                        let idx: usize = idx
                            .parse()
                            .map_err(|_| FieldError::FieldNotFound(field_name.to_string()))?;
                        s.codes
                            .get(idx)
                            .cloned()
                            .ok_or_else(|| FieldError::FieldNotFound(field_name.to_string()))
                    } else {
                        Err(FieldError::FieldNotFound(field_name.to_string()))
                    }
                }
            },
            ItemType::Custom(s) => s
                .fields
                .get(field_name)
                .cloned()
                .ok_or_else(|| FieldError::FieldNotFound(field_name.to_string())),
        }
    }

    pub fn set_field(&mut self, field_name: &str, value: &str) -> Result<(), FieldError> {
        match self {
            ItemType::Login(s) => match field_name {
                "username" => s.username = value.to_string(),
                "password" => s.password = value.to_string(),
                "url" => {
                    s.url = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "notes" => {
                    s.notes = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                _ => return Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::APIKey(s) => match field_name {
                "name" => s.name = value.to_string(),
                "key" => s.key = value.to_string(),
                "provider" => s.provider = value.to_string(),
                _ => return Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::SSHKey(s) => match field_name {
                "name" => s.name = value.to_string(),
                "public_key" => s.public_key = value.to_string(),
                "private_key" => s.private_key = value.to_string(),
                "passphrase" => {
                    s.passphrase = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "comment" => {
                    s.comment = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "allowed_hosts" => {
                    s.allowed_hosts = value
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                        .collect();
                }
                _ => return Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::SecureNote(s) => match field_name {
                "title" => s.title = value.to_string(),
                "content" => s.content = value.to_string(),
                _ => return Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::Server(s) => match field_name {
                "hostname" => s.hostname = value.to_string(),
                "port" => {
                    s.port = value.parse().map_err(|_| {
                        FieldError::InvalidFieldValue("port must be a number".to_string())
                    })?
                }
                "username" => s.username = value.to_string(),
                "password" => {
                    s.password = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "notes" => {
                    s.notes = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                _ => return Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::Database(s) => match field_name {
                "engine" => s.engine = value.to_string(),
                "host" => s.host = value.to_string(),
                "port" => {
                    s.port = value.parse().map_err(|_| {
                        FieldError::InvalidFieldValue("port must be a number".to_string())
                    })?
                }
                "username" => s.username = value.to_string(),
                "password" => s.password = value.to_string(),
                "database" => s.database = value.to_string(),
                _ => return Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::Identity(s) => match field_name {
                "name" => s.name = value.to_string(),
                "email" => s.email = value.to_string(),
                "phone" => {
                    s.phone = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "address" => {
                    s.address = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                _ => return Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::SoftwareLicense(s) => match field_name {
                "product" => s.product = value.to_string(),
                "key" => s.key = value.to_string(),
                "version" => {
                    s.version = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "expiry" => {
                    s.expiry = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                _ => return Err(FieldError::FieldNotFound(field_name.to_string())),
            },
            ItemType::RecoveryCodes(s) => {
                if field_name == "codes" {
                    s.codes = value
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                        .collect();
                } else if let Some(idx) = field_name.strip_prefix("code_") {
                    let idx: usize = idx
                        .parse()
                        .map_err(|_| FieldError::FieldNotFound(field_name.to_string()))?;
                    if idx < s.codes.len() {
                        s.codes[idx] = value.to_string();
                    } else {
                        return Err(FieldError::FieldNotFound(field_name.to_string()));
                    }
                } else {
                    return Err(FieldError::FieldNotFound(field_name.to_string()));
                }
            }
            ItemType::Custom(s) => {
                s.fields.insert(field_name.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    pub fn list_fields(&self) -> Vec<&'static str> {
        match self {
            ItemType::Login(_) => vec!["username", "password", "url", "notes"],
            ItemType::APIKey(_) => vec!["name", "key", "provider"],
            ItemType::SSHKey(_) => {
                vec![
                    "name",
                    "public_key",
                    "private_key",
                    "passphrase",
                    "comment",
                    "allowed_hosts",
                ]
            }
            ItemType::SecureNote(_) => vec!["title", "content"],
            ItemType::Server(_) => vec!["hostname", "port", "username", "password", "notes"],
            ItemType::Database(_) => {
                vec!["engine", "host", "port", "username", "password", "database"]
            }
            ItemType::Identity(_) => vec!["name", "email", "phone", "address"],
            ItemType::SoftwareLicense(_) => vec!["product", "key", "version", "expiry"],
            ItemType::RecoveryCodes(_) => vec!["codes"],
            ItemType::Custom(_) => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_get_field() {
        let item = ItemType::Login(LoginSecret {
            username: "user@example.com".to_string(),
            password: "secret123".to_string(),
            url: Some("https://example.com".to_string()),
            notes: None,
        });
        assert_eq!(item.get_field("username").unwrap(), "user@example.com");
        assert_eq!(item.get_field("password").unwrap(), "secret123");
        assert_eq!(item.get_field("url").unwrap(), "https://example.com");
        assert_eq!(item.get_field("notes").unwrap(), "");
        assert!(item.get_field("nonexistent").is_err());
    }

    #[test]
    fn login_set_field() {
        let mut item = ItemType::Login(LoginSecret {
            username: "old".to_string(),
            password: "old".to_string(),
            url: None,
            notes: None,
        });
        item.set_field("username", "new").unwrap();
        item.set_field("password", "newpass").unwrap();
        item.set_field("url", "https://new.com").unwrap();
        assert_eq!(item.get_field("username").unwrap(), "new");
        assert_eq!(item.get_field("password").unwrap(), "newpass");
        assert_eq!(item.get_field("url").unwrap(), "https://new.com");
    }

    #[test]
    fn apikey_get_set_field() {
        let mut item = ItemType::APIKey(APIKeySecret {
            name: "test".to_string(),
            key: "key123".to_string(),
            provider: "aws".to_string(),
        });
        assert_eq!(item.get_field("key").unwrap(), "key123");
        item.set_field("key", "newkey456").unwrap();
        assert_eq!(item.get_field("key").unwrap(), "newkey456");
    }

    #[test]
    fn sshkey_get_set_field() {
        let mut item = ItemType::SSHKey(SSHKeySecret {
            name: "server".to_string(),
            public_key: "ssh-rsa AAAA...".to_string(),
            private_key: "-----BEGIN...".to_string(),
            passphrase: Some("phrase".to_string()),
            comment: Some("server key".to_string()),
            allowed_hosts: vec!["github.com".to_string()],
        });
        assert_eq!(item.get_field("private_key").unwrap(), "-----BEGIN...");
        item.set_field("private_key", "new-private-key").unwrap();
        assert_eq!(item.get_field("private_key").unwrap(), "new-private-key");
    }

    #[test]
    fn database_get_set_field() {
        let mut item = ItemType::Database(DatabaseSecret {
            engine: "postgres".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            username: "admin".to_string(),
            password: "dbpass".to_string(),
            database: "mydb".to_string(),
        });
        assert_eq!(item.get_field("password").unwrap(), "dbpass");
        item.set_field("port", "3306").unwrap();
        assert_eq!(item.get_field("port").unwrap(), "3306");
    }

    #[test]
    fn custom_get_set_field() {
        let mut item = ItemType::Custom(CustomSecret {
            fields: std::collections::HashMap::new(),
        });
        item.set_field("custom_field", "custom_value").unwrap();
        assert_eq!(item.get_field("custom_field").unwrap(), "custom_value");
    }

    #[test]
    fn list_fields_login() {
        let item = ItemType::Login(LoginSecret {
            username: "u".to_string(),
            password: "p".to_string(),
            url: None,
            notes: None,
        });
        let fields = item.list_fields();
        assert!(fields.contains(&"username"));
        assert!(fields.contains(&"password"));
    }
}
