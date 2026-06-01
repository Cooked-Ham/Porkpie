use porkpie_sync::{ConflictItem, EncryptedSyncItem, MergeStrategy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: String,
    pub timestamp: i64,
    pub storage: String,
}

/// Request body for registering a vault on the sync server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncRegisterRequest {
    pub vault_id: String,
    pub name: String,
    pub salt: Vec<u8>,
    pub master_key_wrapped: Vec<u8>,
    pub created_at: i64,
}

/// Response body for vault registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncRegisterResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPushRequest {
    pub vault_id: String,
    pub base_revision: u64,
    pub items: Vec<EncryptedSyncItem>,
    pub merge_strategy: Option<MergeStrategy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPushResponse {
    pub accepted: usize,
    pub new_revision: u64,
    pub conflicts: Vec<ConflictItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts: Option<Vec<ConflictItem>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultMetadataResponse {
    pub vault_id: String,
    pub name: String,
    pub salt: Vec<u8>,
    pub master_key_wrapped: Vec<u8>,
    pub created_at: i64,
    pub sync_revision: u64,
}
