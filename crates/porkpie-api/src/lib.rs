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
}

/// Build the API router.
pub fn build_router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/api/v1/sync/begin", post(handlers::sync_begin))
        .route("/api/v1/sync/push", post(handlers::sync_push))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/status", get(handlers::status))
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .with_state(state)
}
