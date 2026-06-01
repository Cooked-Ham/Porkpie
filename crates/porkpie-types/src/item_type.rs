use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginSecret {
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIKeySecret {
    pub name: String,
    pub key: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHKeySecret {
    pub name: String,
    pub public_key: String,
    pub private_key: String,
    pub passphrase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureNoteSecret {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSecret {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSecret {
    pub engine: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySecret {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareLicenseSecret {
    pub product: String,
    pub key: String,
    pub version: Option<String>,
    pub expiry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCodesSecret {
    pub codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSecret {
    pub fields: std::collections::HashMap<String, String>,
}
