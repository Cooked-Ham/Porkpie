use crate::ids::VaultId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: VaultId,
    pub name: String,
    pub created_at: i64,
    pub master_key_wrapped: Vec<u8>,
    pub sync_revision: u64,
}

impl fmt::Debug for Vault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vault")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("created_at", &self.created_at)
            .field("master_key_wrapped", &"[redacted]")
            .field("sync_revision", &self.sync_revision)
            .finish()
    }
}
