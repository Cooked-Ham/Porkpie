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
    db::upsert_api_key(&pool, API_KEY)
        .await
        .expect("seed api key");
    db::revoke_api_key(&pool, API_KEY)
        .await
        .expect("revoke api key");

    let app = build_router(AppState { pool });

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
    db::upsert_api_key(&pool, API_KEY)
        .await
        .expect("seed api key");
    build_router(AppState { pool })
}

async fn seeded_app() -> (axum::Router, String) {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");
    db::upsert_api_key(&pool, API_KEY)
        .await
        .expect("seed api key");
    let vault_id = VaultId::new().to_string();
    db::upsert_vault_metadata(&pool, &vault_id)
        .await
        .expect("seed vault");
    (build_router(AppState { pool }), vault_id)
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
