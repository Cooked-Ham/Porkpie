use porkpie_sync::{
    detect_conflicts, merge_items, sync_vault, EncryptedSyncItem, MergeStrategy, SyncRequest,
    SyncResponse,
};

#[test]
fn sync_request_and_response_are_json_serializable() {
    let request = SyncRequest {
        vault_id: "vault-1".to_string(),
        last_revision: 7,
    };
    let json = serde_json::to_string(&request).expect("serialize request");
    assert!(json.contains("vault-1"));

    let response = SyncResponse {
        items: vec![item("a", "Login", b"ciphertext-a", 8, 10)],
        new_revision: 8,
        conflicts: Vec::new(),
    };
    let decoded: SyncResponse =
        serde_json::from_str(&serde_json::to_string(&response).expect("serialize response"))
            .expect("deserialize response");
    assert_eq!(decoded, response);
}

#[test]
fn conflict_detection_reports_same_item_changed_on_both_sides() {
    let local = vec![item("a", "Login", b"local", 6, 20)];
    let server = vec![item("a", "Login", b"server", 7, 19)];

    let conflicts = detect_conflicts(&local, &server, 5);

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].item_id, "a");
    assert_eq!(conflicts[0].local_revision, 6);
    assert_eq!(conflicts[0].server_revision, 7);
}

#[test]
fn last_write_wins_merges_newer_encrypted_item() {
    let local = vec![item("a", "Login", b"older", 6, 10)];
    let server = vec![item("a", "Login", b"newer", 7, 20)];

    let merged = merge_items(local, server, MergeStrategy::LastWriteWins);

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].ciphertext, b"newer");
}

#[test]
fn sync_vault_returns_merged_state_with_conflict_metadata() {
    let local = vec![item("a", "Login", b"local", 6, 20)];
    let response = SyncResponse {
        items: vec![item("a", "Login", b"server", 7, 19)],
        new_revision: 7,
        conflicts: Vec::new(),
    };

    let outcome = sync_vault(local, &response, 5, MergeStrategy::LastWriteWins)
        .expect("last-write-wins resolves conflict");

    assert_eq!(outcome.conflicts.len(), 1);
    assert_eq!(outcome.items[0].ciphertext, b"local");
    assert_eq!(outcome.new_revision, 7);
}

fn item(
    item_id: &str,
    item_type: &str,
    ciphertext: &[u8],
    sync_revision: u64,
    updated_at: i64,
) -> EncryptedSyncItem {
    EncryptedSyncItem {
        item_id: item_id.to_string(),
        item_type: item_type.to_string(),
        ciphertext: ciphertext.to_vec(),
        created_at: 1,
        updated_at,
        sync_revision,
    }
}
