use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use porkpie_api::{
    build_router, db,
    models::{ErrorResponse, SyncPushRequest, SyncPushResponse},
    AppState,
};
use porkpie_sync::{EncryptedSyncItem, MergeStrategy, SyncRequest, SyncResponse};
use porkpie_types::{ItemId, VaultId};
use tower::ServiceExt;

const API_KEY: &str = "test-key";

#[tokio::test]
async fn health_returns_ok_without_authentication() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn sync_routes_reject_missing_api_key() {
    let app = test_app().await;

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/begin",
            &SyncRequest {
                vault_id: VaultId::new().to_string(),
                last_revision: 0,
            },
            None,
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn sync_begin_returns_not_found_for_unknown_vault() {
    let app = test_app().await;

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/begin",
            &SyncRequest {
                vault_id: VaultId::new().to_string(),
                last_revision: 0,
            },
            Some(API_KEY),
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn push_stores_encrypted_data_and_begin_returns_changes() {
    let (app, vault_id) = seeded_app().await;
    let item = encrypted_item(ItemId::new().to_string(), b"encrypted only".to_vec(), 0, 1);

    let push_response = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id: vault_id.clone(),
                base_revision: 0,
                items: vec![item.clone()],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push response");

    assert_eq!(push_response.status(), StatusCode::OK);
    let pushed: SyncPushResponse = response_json(push_response).await;
    assert_eq!(pushed.accepted, 1);
    assert_eq!(pushed.new_revision, 1);

    let begin_response = app
        .oneshot(json_request(
            "/api/v1/sync/begin",
            &SyncRequest {
                vault_id,
                last_revision: 0,
            },
            Some(API_KEY),
        ))
        .await
        .expect("begin response");

    assert_eq!(begin_response.status(), StatusCode::OK);
    let sync: SyncResponse = response_json(begin_response).await;
    assert_eq!(sync.items.len(), 1);
    assert_eq!(sync.items[0].ciphertext, item.ciphertext);
}

#[tokio::test]
async fn push_reports_conflict_when_server_changed_after_base_revision() {
    let (app, vault_id) = seeded_app().await;
    let item_id = ItemId::new().to_string();

    let first = encrypted_item(item_id.clone(), b"server".to_vec(), 0, 1);
    let first_response = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id: vault_id.clone(),
                base_revision: 0,
                items: vec![first],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("first push");
    assert_eq!(first_response.status(), StatusCode::OK);

    let local = encrypted_item(item_id, b"local".to_vec(), 0, 2);
    let conflict_response = app
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id,
                base_revision: 0,
                items: vec![local],
                merge_strategy: Some(MergeStrategy::PreferLocal),
            },
            Some(API_KEY),
        ))
        .await
        .expect("conflict response");

    assert_eq!(conflict_response.status(), StatusCode::CONFLICT);
    let error: ErrorResponse = response_json(conflict_response).await;
    assert_eq!(error.error, "sync_conflict");
    assert_eq!(error.conflicts.expect("conflict data").len(), 1);
}

#[tokio::test]
async fn sync_routes_reject_wrong_api_key() {
    let app = test_app().await;

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/begin",
            &SyncRequest {
                vault_id: VaultId::new().to_string(),
                last_revision: 0,
            },
            Some("wrong-api-key-12345"),
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn sync_routes_reject_revoked_api_key() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");
    db::upsert_api_key(&pool, API_KEY, "test")
        .await
        .expect("seed api key");
    db::revoke_api_key(&pool, API_KEY)
        .await
        .expect("revoke api key");

    let app = build_router(AppState {
        pool,
        cors_allowed_origins: vec!["https://app.porkpie.love".to_string()],
    });

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/begin",
            &SyncRequest {
                vault_id: VaultId::new().to_string(),
                last_revision: 0,
            },
            Some(API_KEY),
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn push_rejects_plaintext_username_payload() {
    let (app, vault_id) = seeded_app().await;
    let plaintext = r#"{"username":"alice","password":"secret123"}"#;
    let item = encrypted_item(
        ItemId::new().to_string(),
        plaintext.as_bytes().to_vec(),
        0,
        1,
    );

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id,
                base_revision: 0,
                items: vec![item],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: ErrorResponse = response_json(response).await;
    assert_eq!(error.error, "validation_error");
    assert!(
        error.message.contains("username"),
        "error should mention the detected field: {}",
        error.message
    );
}

#[tokio::test]
async fn push_rejects_plaintext_password_payload() {
    let (app, vault_id) = seeded_app().await;
    let plaintext = r#"{"password":"secret123"}"#;
    let item = encrypted_item(
        ItemId::new().to_string(),
        plaintext.as_bytes().to_vec(),
        0,
        1,
    );

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id,
                base_revision: 0,
                items: vec![item],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: ErrorResponse = response_json(response).await;
    assert!(error.message.contains("password"));
}

#[tokio::test]
async fn push_rejects_plaintext_private_key_payload() {
    let (app, vault_id) = seeded_app().await;
    let plaintext = r#"{"private_key":"-----BEGIN OPENSSH PRIVATE KEY-----"}"#;
    let item = encrypted_item(
        ItemId::new().to_string(),
        plaintext.as_bytes().to_vec(),
        0,
        1,
    );

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id,
                base_revision: 0,
                items: vec![item],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: ErrorResponse = response_json(response).await;
    assert!(error.message.contains("private_key"));
}

#[tokio::test]
async fn push_rejects_plaintext_api_key_payload() {
    let (app, vault_id) = seeded_app().await;
    let plaintext = r#"{"api_key":"sk_live_12345"}"#;
    let item = encrypted_item(
        ItemId::new().to_string(),
        plaintext.as_bytes().to_vec(),
        0,
        1,
    );

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id,
                base_revision: 0,
                items: vec![item],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: ErrorResponse = response_json(response).await;
    assert!(error.message.contains("api_key"));
}

#[tokio::test]
async fn push_rejects_plaintext_totp_payload() {
    let (app, vault_id) = seeded_app().await;
    let plaintext = r#"{"totp":"123456"}"#;
    let item = encrypted_item(
        ItemId::new().to_string(),
        plaintext.as_bytes().to_vec(),
        0,
        1,
    );

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id,
                base_revision: 0,
                items: vec![item],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: ErrorResponse = response_json(response).await;
    assert!(error.message.contains("totp"));
}

#[tokio::test]
async fn push_rejects_plaintext_notes_payload() {
    let (app, vault_id) = seeded_app().await;
    let plaintext = r#"{"notes":"secret notes here"}"#;
    let item = encrypted_item(
        ItemId::new().to_string(),
        plaintext.as_bytes().to_vec(),
        0,
        1,
    );

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id,
                base_revision: 0,
                items: vec![item],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: ErrorResponse = response_json(response).await;
    assert!(error.message.contains("notes"));
}

#[tokio::test]
async fn push_accepts_real_encrypted_binary_blobs() {
    let (app, vault_id) = seeded_app().await;
    // Real encrypted data should not contain JSON-like field names.
    let real_ciphertext = vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
        0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff, 0x00,
    ];
    let item = encrypted_item(ItemId::new().to_string(), real_ciphertext, 0, 1);

    let response = app
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id,
                base_revision: 0,
                items: vec![item],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push response");

    assert_eq!(response.status(), StatusCode::OK);
}

async fn test_app() -> axum::Router {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");
    db::upsert_api_key(&pool, API_KEY, "test")
        .await
        .expect("seed api key");
    build_router(AppState {
        pool,
        cors_allowed_origins: vec!["https://app.porkpie.love".to_string()],
    })
}

async fn seeded_app() -> (axum::Router, String) {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");
    db::upsert_api_key(&pool, API_KEY, "test")
        .await
        .expect("seed api key");
    let vault_id = VaultId::new().to_string();
    db::upsert_vault_metadata(&pool, &vault_id)
        .await
        .expect("seed vault");
    (
        build_router(AppState {
            pool,
            cors_allowed_origins: vec!["https://app.porkpie.love".to_string()],
        }),
        vault_id,
    )
}

fn json_request<T: serde::Serialize>(uri: &str, body: &T, api_key: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(api_key) = api_key {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {api_key}"));
    }

    builder
        .body(Body::from(
            serde_json::to_vec(body).expect("serialize request body"),
        ))
        .expect("build request")
}

async fn response_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    serde_json::from_slice(&bytes).expect("deserialize response")
}

fn encrypted_item(
    item_id: String,
    ciphertext: Vec<u8>,
    sync_revision: u64,
    updated_at: i64,
) -> EncryptedSyncItem {
    EncryptedSyncItem {
        item_id,
        item_type: "Login".to_string(),
        ciphertext,
        created_at: 1,
        updated_at,
        sync_revision,
    }
}

#[tokio::test]
async fn same_item_id_in_two_vaults_does_not_collide() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");
    db::upsert_api_key(&pool, API_KEY, "test")
        .await
        .expect("seed api key");

    let vault_a = VaultId::new().to_string();
    let vault_b = VaultId::new().to_string();
    db::upsert_vault_metadata(&pool, &vault_a)
        .await
        .expect("seed vault A");
    db::upsert_vault_metadata(&pool, &vault_b)
        .await
        .expect("seed vault B");

    let app = build_router(AppState {
        pool,
        cors_allowed_origins: vec!["https://app.porkpie.love".to_string()],
    });
    let item_id = ItemId::new().to_string();

    // Push item with same ID to vault A
    let push_a = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id: vault_a.clone(),
                base_revision: 0,
                items: vec![encrypted_item(
                    item_id.clone(),
                    b"vault_a_ciphertext".to_vec(),
                    0,
                    1,
                )],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push A");
    assert_eq!(push_a.status(), StatusCode::OK);

    // Push item with same ID to vault B
    let push_b = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/push",
            &SyncPushRequest {
                vault_id: vault_b.clone(),
                base_revision: 0,
                items: vec![encrypted_item(
                    item_id.clone(),
                    b"vault_b_ciphertext".to_vec(),
                    0,
                    1,
                )],
                merge_strategy: None,
            },
            Some(API_KEY),
        ))
        .await
        .expect("push B");
    assert_eq!(push_b.status(), StatusCode::OK);

    // Pull vault A and verify only vault A's item is returned
    let begin_a = app
        .clone()
        .oneshot(json_request(
            "/api/v1/sync/begin",
            &SyncRequest {
                vault_id: vault_a.clone(),
                last_revision: 0,
            },
            Some(API_KEY),
        ))
        .await
        .expect("begin A");
    assert_eq!(begin_a.status(), StatusCode::OK);
    let sync_a: SyncResponse = response_json(begin_a).await;
    assert_eq!(sync_a.items.len(), 1);
    assert_eq!(sync_a.items[0].ciphertext, b"vault_a_ciphertext");

    // Pull vault B and verify only vault B's item is returned
    let begin_b = app
        .oneshot(json_request(
            "/api/v1/sync/begin",
            &SyncRequest {
                vault_id: vault_b.clone(),
                last_revision: 0,
            },
            Some(API_KEY),
        ))
        .await
        .expect("begin B");
    assert_eq!(begin_b.status(), StatusCode::OK);
    let sync_b: SyncResponse = response_json(begin_b).await;
    assert_eq!(sync_b.items.len(), 1);
    assert_eq!(sync_b.items[0].ciphertext, b"vault_b_ciphertext");

    // Verify sync revisions are independent
    assert_eq!(sync_a.new_revision, 1);
    assert_eq!(sync_b.new_revision, 1);
}

async fn admin_app() -> axum::Router {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");
    let (_key_id, key_hash) = db::upsert_api_key(&pool, API_KEY, "test")
        .await
        .expect("seed api key");
    db::set_api_key_admin(&pool, &key_hash, true)
        .await
        .expect("set admin");
    build_router(AppState {
        pool,
        cors_allowed_origins: vec!["https://app.porkpie.love".to_string()],
    })
}

async fn non_admin_app() -> axum::Router {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");
    db::upsert_api_key(&pool, API_KEY, "test")
        .await
        .expect("seed api key");
    // Do NOT set admin flag
    build_router(AppState {
        pool,
        cors_allowed_origins: vec!["https://app.porkpie.love".to_string()],
    })
}

#[tokio::test]
async fn admin_add_api_key_rejects_non_admin_key() {
    let app = non_admin_app().await;

    let response = app
        .oneshot(json_request(
            "/api/v1/admin/api-key",
            &serde_json::json!({
                "api_key": "new-api-key-1234567890123456789012345678",
                "label": "test"
            }),
            Some(API_KEY),
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_add_api_key_accepts_admin_key() {
    let app = admin_app().await;

    let response = app
        .oneshot(json_request(
            "/api/v1/admin/api-key",
            &serde_json::json!({
                "api_key": "new-api-key-1234567890123456789012345678",
                "label": "test"
            }),
            Some(API_KEY),
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response_json(response).await;
    assert_eq!(body["ok"], true);
    assert!(body["key_id"].is_number());
    assert!(body["key_hash"].is_string());
}

#[tokio::test]
async fn admin_revoke_api_key_rejects_non_admin_key() {
    let app = non_admin_app().await;

    let response = app
        .oneshot(json_request(
            "/api/v1/admin/api-key/revoke",
            &serde_json::json!({
                "key_id": 1,
                "force": true
            }),
            Some(API_KEY),
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_revoke_api_key_accepts_admin_key() {
    let app = admin_app().await;

    // First add a second key so we can revoke one without triggering the last-key guard
    let add_response = app
        .clone()
        .oneshot(json_request(
            "/api/v1/admin/api-key",
            &serde_json::json!({
                "api_key": "second-key-1234567890123456789012345678",
                "label": "second"
            }),
            Some(API_KEY),
        ))
        .await
        .expect("add response");
    assert_eq!(add_response.status(), StatusCode::OK);
    let body: serde_json::Value = response_json(add_response).await;
    let key_id = body["key_id"].as_i64().expect("key_id should be number");

    // Now revoke the second key
    let revoke_response = app
        .oneshot(json_request(
            "/api/v1/admin/api-key/revoke",
            &serde_json::json!({
                "key_id": key_id,
                "force": false
            }),
            Some(API_KEY),
        ))
        .await
        .expect("revoke response");

    assert_eq!(revoke_response.status(), StatusCode::OK);
    let body: serde_json::Value = response_json(revoke_response).await;
    assert_eq!(body["ok"], true);
}
