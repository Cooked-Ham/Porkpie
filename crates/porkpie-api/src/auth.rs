use crate::{db, errors::ApiError, AppState};
use axum::{
    body::Body,
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

/// Key used to store the current API key hash in request extensions.
#[derive(Clone)]
pub struct CurrentApiKeyHash(pub String);

/// Validate a bearer API key and check admin privileges before admin routes.
/// Inserts the current API key hash into request extensions for self-revoke
/// guards downstream.
pub async fn require_admin_api_key(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> std::result::Result<Response, ApiError> {
    let api_key = bearer_token(&request).ok_or(ApiError::Unauthorized)?;
    if !db::api_key_exists(&state.pool, api_key).await? {
        return Err(ApiError::Unauthorized);
    }
    if !db::api_key_is_admin(&state.pool, api_key).await? {
        return Err(ApiError::Forbidden);
    }
    let hash = db::hash_api_key(api_key);
    request.extensions_mut().insert(CurrentApiKeyHash(hash));
    Ok(next.run(request).await)
}

/// Validate a bearer API key before protected sync routes.
/// Inserts the current API key hash into request extensions.
pub async fn require_api_key(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> std::result::Result<Response, ApiError> {
    let api_key = bearer_token(&request).ok_or(ApiError::Unauthorized)?;
    if db::api_key_exists(&state.pool, api_key).await? {
        let hash = db::hash_api_key(api_key);
        request.extensions_mut().insert(CurrentApiKeyHash(hash));
        Ok(next.run(request).await)
    } else {
        Err(ApiError::Unauthorized)
    }
}

fn bearer_token(request: &Request<Body>) -> Option<&str> {
    let value = request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    value
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}
