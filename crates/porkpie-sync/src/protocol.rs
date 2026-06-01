use crate::conflict::{detect_conflicts, merge_items, ConflictItem, MergeStrategy};
use crate::errors::{Result, SyncError};
use serde::{Deserialize, Serialize};

/// Request sent by a client to begin sync for a vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncRequest {
    pub vault_id: String,
    pub last_revision: u64,
}

/// Encrypted item envelope used by sync clients and the API server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedSyncItem {
    pub item_id: String,
    pub item_type: String,
    pub ciphertext: Vec<u8>,
    pub created_at: i64,
    pub updated_at: i64,
    pub sync_revision: u64,
}

/// Server response for a sync begin request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncResponse {
    pub items: Vec<EncryptedSyncItem>,
    pub new_revision: u64,
    pub conflicts: Vec<ConflictItem>,
}

/// Client-side result after applying a server response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncOutcome {
    pub items: Vec<EncryptedSyncItem>,
    pub new_revision: u64,
    pub conflicts: Vec<ConflictItem>,
}

/// Merge local encrypted items with server changes and report conflicts.
pub fn sync_vault(
    local_items: Vec<EncryptedSyncItem>,
    server_response: &SyncResponse,
    base_revision: u64,
    strategy: MergeStrategy,
) -> Result<SyncOutcome> {
    if server_response.new_revision < base_revision {
        return Err(SyncError::InvalidRevisionWindow {
            base_revision,
            server_revision: server_response.new_revision,
        });
    }

    let mut conflicts = detect_conflicts(&local_items, &server_response.items, base_revision);
    conflicts.extend(server_response.conflicts.clone());

    if !conflicts.is_empty() && strategy != MergeStrategy::LastWriteWins {
        return Err(SyncError::Conflict {
            count: conflicts.len(),
        });
    }

    let items = merge_items(local_items, server_response.items.clone(), strategy);
    Ok(SyncOutcome {
        items,
        new_revision: server_response.new_revision,
        conflicts,
    })
}
