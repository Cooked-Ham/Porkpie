use porkpie_sync::{ConflictItem, EncryptedSyncItem, MergeStrategy};
use serde::{Deserialize, Serialize};

/// Liveness response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: i64,
}

/// Server status response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: String,
    pub timestamp: i64,
    pub storage: String,
}

/// Request body for uploading encrypted item changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPushRequest {
    pub vault_id: String,
    pub base_revision: u64,
    pub items: Vec<EncryptedSyncItem>,
    pub merge_strategy: Option<MergeStrategy>,
}

/// Response body after applying encrypted item changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPushResponse {
    pub accepted: usize,
    pub new_revision: u64,
    pub conflicts: Vec<ConflictItem>,
}

/// Standard API error response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts: Option<Vec<ConflictItem>>,
}
