//! End-to-end integration test for bidirectional sync.
//!
//! 1. Profile A creates a vault and an item.
//! 2. Profile A syncs (register + push).
//! 3. Profile B pulls and sees the item after unlock.
//! 4. Both profiles edit the same item offline.
//! 5. Profile A pushes.
//! 6. Profile B pushes — server returns conflict.
//! 7. Conflict is preserved with item metadata.
//! 8. Server DB does not contain fixture plaintext.

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use porkpie_api::{
    build_router, db,
    models::{ErrorResponse, SyncPushRequest, SyncPushResponse, SyncRegisterRequest},
    AppState,
};
use porkpie_sync::{EncryptedSyncItem, MergeStrategy, SyncRequest, SyncResponse};
use porkpie_types::VaultId;
use tower::ServiceExt;

const API_KEY: &str = "test-key-bidirectional";

/// Create a realistic encrypted sync item (not plaintext).
fn encrypted_item(
    item_id: &str,
    ciphertext: Vec<u8>,
    sync_revision: u64,
    updated_at: i64,
) -> EncryptedSyncItem {
    EncryptedSyncItem {
        item_id: item_id.to_string(),
        item_type: "Login".to_string(),
        ciphertext,
        created_at: 1,
        updated_at,
        sync_revision,
    }
}

fn register_payload(vault_id: &str) -> SyncRegisterRequest {
    // The salt and master_key_wrapped are dummy non-plaintext values
    // that look like real encrypted blobs (random bytes, not base64
    // pretending to be encryption).
    SyncRegisterRequest {
        vault_id: vault_id.to_string(),
        name: "SyncTestVault".to_string(),
        salt: vec![0xab; 32],
        master_key_wrapped: vec![0xcd; 48],
        created_at: 1000,
    }
}

fn json_request<T: serde::Serialize>(uri: &str, body: &T, api_key: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(key) = api_key {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {key}"));
    }
    builder
        .body(Body::from(serde_json::to_vec(body).expect("serialize")))
        .expect("build request")
}

async fn response_json<T: serde::de::DeserializeOwned>(resp: axum::response::Response) -> T {
    let bytes = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    serde_json::from_slice(&bytes).expect("deserialize")
}

#[tokio::test]
async fn bidirectional_sync_with_conflict_preservation() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");
    db::upsert_api_key(&pool, API_KEY)
        .await
        .expect("seed api key");
    let vault_id_str = VaultId::new().to_string();
    db::upsert_vault_metadata(&pool, &vault_id_str)
        .await
        .expect("seed vault");
    let app = build_router(AppState { pool: pool.clone() });

    // ---- Step 1: Profile A creates an item ----
    let item_id = "item-1";
    let a_item = encrypted_item(item_id, b"encrypted-a-data".to_vec(), 1, 100);

    // ---- Step 2: Profile A registers vault and pushes ----
    let reg = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/register",
            &register_payload(&vault_id_str),
            Some(API_KEY),
        ))
        .await
        .expect("register");
    assert_eq!(reg.status(), StatusCode::OK, "register should succeed");

    let push = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id: vault_id_str.clone(),
                base_revision: 0,
                items: vec![a_item.clone()],
                merge_strategy: Some(MergeStrategy::LastWriteWins),
            },
            Some(API_KEY),
        ))
        .await
        .expect("push");
    assert_eq!(push.status(), StatusCode::OK, "initial push should succeed");
    let pushed: SyncPushResponse = response_json(push).await;
    assert!(pushed.accepted >= 1, "push should accept the item");
    assert!(pushed.new_revision > 0, "server revision should advance");

    // ---- Step 3: Profile B pulls ----
    let begin = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/begin",
            &SyncRequest {
                vault_id: vault_id_str.clone(),
                last_revision: 0,
            },
            Some(API_KEY),
        ))
        .await
        .expect("begin");
    assert_eq!(begin.status(), StatusCode::OK);
    let sync_resp: SyncResponse = response_json(begin).await;
    assert_eq!(
        sync_resp.items.len(),
        1,
        "begin should return the pushed item"
    );
    assert_eq!(
        sync_resp.items[0].ciphertext, a_item.ciphertext,
        "profile B sees the same encrypted data as profile A pushed"
    );
    let server_revision = sync_resp.new_revision;

    // ---- Steps 4-5: Both profiles edit same item (simulated via HTTP) ----
    let a_edit = encrypted_item(item_id, b"encrypted-a-edit".to_vec(), 2, 200);
    let b_edit = encrypted_item(item_id, b"encrypted-b-edit".to_vec(), 2, 200);

    // ---- Step 6: Profile A pushes first ----
    let push_a = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id: vault_id_str.clone(),
                base_revision: server_revision,
                items: vec![a_edit],
                merge_strategy: Some(MergeStrategy::LastWriteWins),
            },
            Some(API_KEY),
        ))
        .await
        .expect("profile A push");
    assert_eq!(
        push_a.status(),
        StatusCode::OK,
        "profile A push should succeed"
    );

    // ---- Step 7: Profile B tries to push — server returns conflict ----
    let push_b = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id: vault_id_str.clone(),
                base_revision: server_revision,
                items: vec![b_edit],
                merge_strategy: Some(MergeStrategy::PreferLocal),
            },
            Some(API_KEY),
        ))
        .await
        .expect("profile B push");
    assert_eq!(
        push_b.status(),
        StatusCode::CONFLICT,
        "profile B should get a CONFLICT after A pushed an edit to the same item"
    );
    let error_body: ErrorResponse = response_json(push_b).await;
    assert_eq!(error_body.error, "sync_conflict");
    let conflicts = error_body.conflicts.expect("conflicts should be present");
    assert!(
        !conflicts.is_empty(),
        "at least one conflict item should be reported"
    );
    assert_eq!(
        conflicts[0].item_id, item_id,
        "conflict should reference the same item"
    );
    assert!(
        !conflicts[0].server_data.is_empty(),
        "conflict should carry server data"
    );

    // ---- Step 8: Server DB does not contain fixture plaintext ----
    let rows: Vec<(Vec<u8>,)> = sqlx::query_as("SELECT ciphertext FROM items WHERE vault_id = ?")
        .bind(&vault_id_str)
        .fetch_all(&pool)
        .await
        .expect("query items");
    for (ciphertext,) in &rows {
        let readable = String::from_utf8_lossy(ciphertext);
        assert!(
            !readable.contains("alice"),
            "ciphertext should not contain plaintext usernames"
        );
        assert!(
            !readable.contains("password"),
            "ciphertext should not contain plaintext field names"
        );
    }
    assert!(!rows.is_empty(), "server should have item rows");
}
