use porkpie_types::{ItemId, ItemType, Timestamp};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub data: ItemType,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Item")
            .field("id", &self.id)
            .field("data", &"[redacted]")
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl Item {
    /// Create an item with a fresh identifier and current timestamps.
    pub fn new(data: ItemType) -> Self {
        let now = Timestamp::now();
        Self {
            id: ItemId::new(),
            data,
            created_at: now,
            updated_at: now,
        }
    }

    /// Replace the item's identifier and timestamps for insertion into a vault.
    pub(crate) fn into_created(mut self, id: ItemId, now: Timestamp) -> Self {
        self.id = id;
        self.created_at = now;
        self.updated_at = now;
        self
    }

    /// Replace mutable fields for an update while preserving original creation time.
    pub(crate) fn into_updated(
        mut self,
        id: ItemId,
        created_at: Timestamp,
        now: Timestamp,
    ) -> Self {
        self.id = id;
        self.created_at = created_at;
        self.updated_at = now;
        self
    }

    /// Zeroize decrypted secret material before an item leaves unlocked memory.
    pub(crate) fn zeroize_secret_material(&mut self) {
        match &mut self.data {
            ItemType::Login(secret) => {
                secret.username.zeroize();
                secret.password.zeroize();
                if let Some(url) = &mut secret.url {
                    url.zeroize();
                }
                if let Some(notes) = &mut secret.notes {
                    notes.zeroize();
                }
            }
            ItemType::APIKey(secret) => {
                secret.name.zeroize();
                secret.key.zeroize();
                secret.provider.zeroize();
            }
            ItemType::SSHKey(secret) => {
                secret.name.zeroize();
                secret.public_key.zeroize();
                secret.private_key.zeroize();
                if let Some(passphrase) = &mut secret.passphrase {
                    passphrase.zeroize();
                }
                if let Some(comment) = &mut secret.comment {
                    comment.zeroize();
                }
                for host in &mut secret.allowed_hosts {
                    host.zeroize();
                }
                secret.allowed_hosts.clear();
            }
            ItemType::SecureNote(secret) => {
                secret.title.zeroize();
                secret.content.zeroize();
            }
            ItemType::Server(secret) => {
                secret.hostname.zeroize();
                secret.username.zeroize();
                if let Some(password) = &mut secret.password {
                    password.zeroize();
                }
                if let Some(notes) = &mut secret.notes {
                    notes.zeroize();
                }
            }
            ItemType::Database(secret) => {
                secret.engine.zeroize();
                secret.host.zeroize();
                secret.username.zeroize();
                secret.password.zeroize();
                secret.database.zeroize();
            }
            ItemType::Identity(secret) => {
                secret.name.zeroize();
                secret.email.zeroize();
                if let Some(phone) = &mut secret.phone {
                    phone.zeroize();
                }
                if let Some(address) = &mut secret.address {
                    address.zeroize();
                }
            }
            ItemType::SoftwareLicense(secret) => {
                secret.product.zeroize();
                secret.key.zeroize();
                if let Some(version) = &mut secret.version {
                    version.zeroize();
                }
                if let Some(expiry) = &mut secret.expiry {
                    expiry.zeroize();
                }
            }
            ItemType::RecoveryCodes(secret) => {
                for code in &mut secret.codes {
                    code.zeroize();
                }
                secret.codes.clear();
            }
            ItemType::Custom(secret) => {
                let fields = std::mem::take(&mut secret.fields);
                for (mut key, mut value) in fields {
                    key.zeroize();
                    value.zeroize();
                }
            }
        }
    }
}
