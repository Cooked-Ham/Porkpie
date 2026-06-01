use crate::{
    auth::CurrentApiKeyHash,
    db,
    errors::{ApiError, Result},
    models::{
        HealthResponse, StatusResponse, SyncPushRequest, SyncPushResponse, SyncRegisterRequest,
        SyncRegisterResponse, VaultMetadataResponse,
    },
    AppState,
};
use axum::{extract::State, Extension, Json};
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
        request.kdf_time_cost,
        request.kdf_mem_cost,
        request.kdf_parallelism,
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
/// The server hashes it and stores only the hash for future validation.
/// The plaintext key is never stored.
/// The client must save the plaintext key; it cannot be recovered.
pub async fn admin_add_api_key(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let api_key = request
        .get("api_key")
        .and_then(|v| v.as_str())
        .ok_or(ApiError::BadRequest("missing api_key".to_string()))?;
    let label = request.get("label").and_then(|v| v.as_str()).unwrap_or("");
    let (key_id, _key_hash) = db::upsert_api_key(&state.pool, api_key, label).await?;
    db::log_admin_audit(&state.pool, "api_key_add").await?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "key_id": key_id,
        "label": label,
        "message": "API key added. The server stores only the hash. Save the plaintext key now; it cannot be recovered."
    })))
}

/// Admin: revoke an API key by its ID.
///
/// The request body must contain the key_id to revoke.
/// Prevents revoking the last active key unless force=true.
/// Also prevents self-revoke (an admin key cannot revoke itself).
pub async fn admin_revoke_api_key(
    State(state): State<AppState>,
    Extension(current_hash): Extension<CurrentApiKeyHash>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let key_id = request
        .get("key_id")
        .and_then(|v| v.as_i64())
        .ok_or(ApiError::BadRequest("missing key_id".to_string()))?;

    let force = request
        .get("force")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !force {
        let active_count = db::count_active_api_keys(&state.pool).await?;
        if active_count <= 1 {
            return Err(ApiError::BadRequest(
                "Cannot revoke the last active API key. Pass force=true to override.".to_string(),
            ));
        }
    }

    // Check self-revoke before mutation.
    let target_hash = db::api_key_hash_by_id(&state.pool, key_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if current_hash.0 == target_hash {
        db::log_admin_audit(&state.pool, "api_key_self_revoke_denied").await?;
        return Err(ApiError::BadRequest(
            "Cannot revoke the API key currently in use.".to_string(),
        ));
    }

    let _key_hash = db::revoke_api_key_by_id(&state.pool, key_id).await?;
    db::log_admin_audit(&state.pool, "api_key_revoke").await?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "key_id": key_id,
        "message": "API key revoked."
    })))
}

fn unix_timestamp() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}
