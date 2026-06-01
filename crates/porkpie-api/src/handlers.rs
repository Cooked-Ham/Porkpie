use crate::{
    db,
    errors::{ApiError, Result},
    models::{HealthResponse, StatusResponse, SyncPushRequest, SyncPushResponse},
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

fn unix_timestamp() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}
