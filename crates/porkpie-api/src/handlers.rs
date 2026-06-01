use crate::{
    db,
    errors::{ApiError, Result},
    models::{
        HealthResponse, StatusResponse, SyncPushRequest, SyncPushResponse, SyncRegisterRequest,
        SyncRegisterResponse, VaultMetadataResponse,
    },
    AppState,
};
use axum::{extract::State, Json};
use porkpie_sync::{SyncRequest, SyncResponse};

/// Return server liveness data.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: unix_timestamp(),
    })
}

/// Return server version and storage status.
pub async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: unix_timestamp(),
        storage: "sqlite".to_string(),
    })
}

/// Register a vault on the sync server with its real cryptographic
/// metadata. This is the first sync step: the client sends the
/// vault's id, name, salt, wrapped master key, and creation
/// timestamp. The server stores these as-is and never uses them
/// to decrypt anything.
pub async fn sync_register(
    State(state): State<AppState>,
    Json(request): Json<SyncRegisterRequest>,
) -> Result<Json<SyncRegisterResponse>> {
    db::register_vault(
        &state.pool,
        &request.vault_id,
        &request.name,
        &request.salt,
        &request.master_key_wrapped,
        request.created_at,
    )
    .await?;

    Ok(Json(SyncRegisterResponse { ok: true }))
}

/// Begin sync by returning encrypted changes after the client revision.
pub async fn sync_begin(
    State(state): State<AppState>,
    Json(request): Json<SyncRequest>,
) -> Result<Json<SyncResponse>> {
    let (items, new_revision) =
        db::load_items_since(&state.pool, &request.vault_id, request.last_revision).await?;

    Ok(Json(SyncResponse {
        items,
        new_revision,
        conflicts: Vec::new(),
    }))
}

/// Return vault metadata (encrypted blobs only) so the peer can
/// reconstruct the locked vault locally.
pub async fn vault_metadata(
    State(state): State<AppState>,
    axum::extract::Path(vault_id): axum::extract::Path<String>,
) -> Result<Json<VaultMetadataResponse>> {
    let meta = db::load_vault_metadata(&state.pool, &vault_id).await?;
    Ok(Json(meta))
}

/// Push encrypted item changes to the server store.
pub async fn sync_push(
    State(state): State<AppState>,
    Json(request): Json<SyncPushRequest>,
) -> Result<Json<SyncPushResponse>> {
    let strategy = request.merge_strategy.unwrap_or_default();
    let (accepted, new_revision, conflicts) = db::push_items(
        &state.pool,
        &request.vault_id,
        request.base_revision,
        &request.items,
        strategy,
    )
    .await?;

    if !conflicts.is_empty() {
        return Err(ApiError::Conflict(conflicts));
    }

    Ok(Json(SyncPushResponse {
        accepted,
        new_revision,
        conflicts: Vec::new(),
    }))
}

/// Admin: add a new API key.
///
/// The request body must contain the new API key in plaintext.
/// The server hashes it and stores the hash for future validation.
/// The plaintext key is never stored.
pub async fn admin_add_api_key(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let api_key = request
        .get("api_key")
        .and_then(|v| v.as_str())
        .ok_or(ApiError::BadRequest("missing api_key".to_string()))?;
    db::upsert_api_key(&state.pool, api_key).await?;
    let key_hash = db::hash_api_key(api_key);
    Ok(Json(serde_json::json!({
        "ok": true,
        "key_hash": key_hash,
        "message": "API key added. Store the plaintext key securely; it cannot be recovered."
    })))
}

/// Admin: revoke an API key by its hash.
///
/// The request body must contain the key_hash to revoke.
pub async fn admin_revoke_api_key(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let key_hash = request
        .get("key_hash")
        .and_then(|v| v.as_str())
        .ok_or(ApiError::BadRequest("missing key_hash".to_string()))?;
    db::revoke_api_key_by_hash(&state.pool, key_hash).await?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "message": "API key revoked."
    })))
}

fn unix_timestamp() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}
