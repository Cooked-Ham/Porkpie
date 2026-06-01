use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType {
    Login(LoginSecret),
    APIKey(APIKeySecret),
    SSHKey(SSHKeySecret),
    SecureNote(SecureNoteSecret),
    Server(ServerSecret),
    Database(DatabaseSecret),
    Identity(IdentitySecret),
    SoftwareLicense(SoftwareLicenseSecret),
    RecoveryCodes(RecoveryCodesSecret),
    Custom(CustomSecret),
}

impl fmt::Debug for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Login(_) => write!(f, "Login([redacted])"),
            Self::APIKey(_) => write!(f, "APIKey([redacted])"),
            Self::SSHKey(_) => write!(f, "SSHKey([redacted])"),
            Self::SecureNote(_) => write!(f, "SecureNote([redacted])"),
            Self::Server(_) => write!(f, "Server([redacted])"),
            Self::Database(_) => write!(f, "Database([redacted])"),
            Self::Identity(_) => write!(f, "Identity([redacted])"),
            Self::SoftwareLicense(_) => write!(f, "SoftwareLicense([redacted])"),
            Self::RecoveryCodes(_) => write!(f, "RecoveryCodes([redacted])"),
            Self::Custom(_) => write!(f, "Custom([redacted])"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginSecret {
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
}

impl fmt::Debug for LoginSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginSecret")
            .field("username", &"[redacted]")
            .field("password", &"[redacted]")
            .field("url", &"[redacted]")
            .field("notes", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct APIKeySecret {
    pub name: String,
    pub key: String,
    pub provider: String,
}

impl fmt::Debug for APIKeySecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("APIKeySecret")
            .field("name", &"[redacted]")
            .field("key", &"[redacted]")
            .field("provider", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SSHKeySecret {
    pub name: String,
    pub public_key: String,
    pub private_key: String,
    pub passphrase: Option<String>,
}

impl fmt::Debug for SSHKeySecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SSHKeySecret")
            .field("name", &"[redacted]")
            .field("public_key", &"[redacted]")
            .field("private_key", &"[redacted]")
            .field("passphrase", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecureNoteSecret {
    pub title: String,
    pub content: String,
}

impl fmt::Debug for SecureNoteSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureNoteSecret")
            .field("title", &"[redacted]")
            .field("content", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerSecret {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub notes: Option<String>,
}

impl fmt::Debug for ServerSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerSecret")
            .field("hostname", &"[redacted]")
            .field("port", &self.port)
            .field("username", &"[redacted]")
            .field("password", &"[redacted]")
            .field("notes", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseSecret {
    pub engine: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl fmt::Debug for DatabaseSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseSecret")
            .field("engine", &"[redacted]")
            .field("host", &"[redacted]")
            .field("port", &self.port)
            .field("username", &"[redacted]")
            .field("password", &"[redacted]")
            .field("database", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentitySecret {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
}

impl fmt::Debug for IdentitySecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdentitySecret")
            .field("name", &"[redacted]")
            .field("email", &"[redacted]")
            .field("phone", &"[redacted]")
            .field("address", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SoftwareLicenseSecret {
    pub product: String,
    pub key: String,
    pub version: Option<String>,
    pub expiry: Option<String>,
}

impl fmt::Debug for SoftwareLicenseSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftwareLicenseSecret")
            .field("product", &"[redacted]")
            .field("key", &"[redacted]")
            .field("version", &"[redacted]")
            .field("expiry", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryCodesSecret {
    pub codes: Vec<String>,
}

impl fmt::Debug for RecoveryCodesSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecoveryCodesSecret")
            .field("codes", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomSecret {
    pub fields: std::collections::HashMap<String, String>,
}

impl fmt::Debug for CustomSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomSecret")
            .field("fields", &"[redacted]")
            .finish()
    }
}
