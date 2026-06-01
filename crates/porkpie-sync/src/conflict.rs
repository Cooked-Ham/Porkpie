use crate::protocol::EncryptedSyncItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conflict payload containing only encrypted server data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictItem {
    pub item_id: String,
    pub local_revision: u64,
    pub server_revision: u64,
    pub server_data: Vec<u8>,
}

/// Merge behavior when local and remote edits touch the same item.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    #[default]
    PreserveConflict,
    LastWriteWins,
    PreferLocal,
    PreferRemote,
}

/// Find items changed on both sides since the client base revision.
pub fn detect_conflicts(
    local_items: &[EncryptedSyncItem],
    server_items: &[EncryptedSyncItem],
    base_revision: u64,
) -> Vec<ConflictItem> {
    let local_by_id: HashMap<&str, &EncryptedSyncItem> = local_items
        .iter()
        .map(|item| (item.item_id.as_str(), item))
        .collect();

    server_items
        .iter()
        .filter_map(|server_item| {
            let local_item = local_by_id.get(server_item.item_id.as_str())?;
            let both_changed = local_item.sync_revision > base_revision
                && server_item.sync_revision > base_revision
                && local_item.ciphertext != server_item.ciphertext;

            both_changed.then(|| ConflictItem {
                item_id: server_item.item_id.clone(),
                local_revision: local_item.sync_revision,
                server_revision: server_item.sync_revision,
                server_data: server_item.ciphertext.clone(),
            })
        })
        .collect()
}

/// Merge encrypted item envelopes according to the selected strategy.
pub fn merge_items(
    local_items: Vec<EncryptedSyncItem>,
    server_items: Vec<EncryptedSyncItem>,
    strategy: MergeStrategy,
) -> Vec<EncryptedSyncItem> {
    let mut merged: HashMap<String, EncryptedSyncItem> = local_items
        .into_iter()
        .map(|item| (item.item_id.clone(), item))
        .collect();

    for server_item in server_items {
        match merged.get(server_item.item_id.as_str()) {
            None => {
                merged.insert(server_item.item_id.clone(), server_item);
            }
            Some(local_item) => {
                let choose_server = match strategy {
                    MergeStrategy::PreserveConflict => continue,
                    MergeStrategy::PreferLocal => false,
                    MergeStrategy::PreferRemote => true,
                    MergeStrategy::LastWriteWins => server_item.updated_at >= local_item.updated_at,
                };
                if choose_server {
                    merged.insert(server_item.item_id.clone(), server_item);
                }
            }
        }
    }

    let mut items: Vec<EncryptedSyncItem> = merged.into_values().collect();
    items.sort_by(|left, right| left.item_id.cmp(&right.item_id));
    items
}
