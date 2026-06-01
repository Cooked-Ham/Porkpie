//! Axum HTTP API for encrypted Porkpie vault synchronization.

pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod handlers;
pub mod models;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;

/// Shared API application state.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub cors_allowed_origins: Vec<String>,
}

/// Build the API router.
pub fn build_router(state: AppState) -> Router {
    let cors_layer = build_cors_layer(&state);

    let protected_routes = Router::new()
        .route("/api/v1/sync/register", post(handlers::sync_register))
        .route("/api/v1/sync/begin", post(handlers::sync_begin))
        .route("/api/v1/sync/push", post(handlers::sync_push))
        .route("/api/v1/vault/{vault_id}", get(handlers::vault_metadata))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    let admin_routes = Router::new()
        .route("/api/v1/admin/api-key", post(handlers::admin_add_api_key))
        .route(
            "/api/v1/admin/api-key/revoke",
            post(handlers::admin_revoke_api_key),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_admin_api_key,
        ));

    Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/status", get(handlers::status))
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(cors_layer)
        .with_state(state)
}

fn build_cors_layer(state: &AppState) -> CorsLayer {
    let mut origins = Vec::new();
    for origin in &state.cors_allowed_origins {
        if let Ok(header) = origin.parse::<axum::http::header::HeaderValue>() {
            origins.push(header);
        }
    }
    if origins.is_empty() {
        let default = "https://app.porkpie.love"
            .parse::<axum::http::header::HeaderValue>()
            .expect("valid default origin");
        origins.push(default);
    }
    CorsLayer::new().allow_origin(origins)
}
