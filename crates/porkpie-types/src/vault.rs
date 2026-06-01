use crate::ids::VaultId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: VaultId,
    pub created_at: i64,
    pub master_key_wrapped: Vec<u8>,
    pub sync_revision: u64,
}
