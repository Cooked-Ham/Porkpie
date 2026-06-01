use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroize;

pub const LOCAL_SECRET_KEY_LEN: usize = 32;

#[derive(Clone, Serialize, Deserialize)]
pub struct LocalSecretKey(#[serde(with = "hex_bytes")] [u8; LOCAL_SECRET_KEY_LEN]);

impl LocalSecretKey {
    pub fn generate() -> Self {
        let mut bytes = [0u8; LOCAL_SECRET_KEY_LEN];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let bytes = hex_decode(hex_str).map_err(|e| format!("invalid hex: {e}"))?;
        if bytes.len() != LOCAL_SECRET_KEY_LEN {
            return Err(format!(
                "expected {} bytes, got {}",
                LOCAL_SECRET_KEY_LEN,
                bytes.len()
            ));
        }
        let mut array = [0u8; LOCAL_SECRET_KEY_LEN];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }

    pub fn to_hex(&self) -> String {
        hex_encode(&self.0)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self(*bytes)
    }
}

impl Drop for LocalSecretKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for LocalSecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LocalSecretKey([redacted])")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecoveryKit {
    pub vault_id: String,
    pub local_secret_key: String,
    pub created_at: i64,
    pub instructions: Vec<String>,
    pub warning: String,
}

impl std::fmt::Debug for RecoveryKit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveryKit")
            .field("vault_id", &self.vault_id)
            .field("local_secret_key", &"[redacted]")
            .field("created_at", &self.created_at)
            .field("instructions", &self.instructions)
            .field("warning", &self.warning)
            .finish()
    }
}

impl RecoveryKit {
    pub fn new(vault_id: &str, secret_key: &LocalSecretKey, created_at: i64) -> Self {
        Self {
            vault_id: vault_id.to_string(),
            local_secret_key: secret_key.to_hex(),
            created_at,
            instructions: vec![
                "Store this recovery kit in a secure, offline location.".to_string(),
                "The local secret key is required alongside your master password to unlock your vault.".to_string(),
                "Without both the master password AND this local secret key, your vault cannot be recovered.".to_string(),
                "Never share this recovery kit or store it in plaintext on a networked device.".to_string(),
            ],
            warning: "WARNING: Losing this local secret key means you cannot recover your vault if you forget your master password. Anyone with both this key and your master password can access your vault.".to_string(),
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err("odd-length hex string".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("invalid hex byte at {i}: {e}"))
        })
        .collect()
}

mod hex_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        serializer.serialize_str(&hex)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<[u8; super::LOCAL_SECRET_KEY_LEN], D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        if !hex.len().is_multiple_of(2) {
            return Err(serde::de::Error::custom("odd-length hex string"));
        }
        let bytes: Result<Vec<u8>, D::Error> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(serde::de::Error::custom))
            .collect();
        let bytes = bytes?;
        if bytes.len() != super::LOCAL_SECRET_KEY_LEN {
            return Err(serde::de::Error::custom(format!(
                "expected {} bytes, got {}",
                super::LOCAL_SECRET_KEY_LEN,
                bytes.len()
            )));
        }
        let mut array = [0u8; super::LOCAL_SECRET_KEY_LEN];
        array.copy_from_slice(&bytes);
        Ok(array)
    }
}
