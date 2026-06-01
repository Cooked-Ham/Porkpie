use crate::models::ErrorResponse;
use axum::{http::StatusCode, response::IntoResponse, Json};
use porkpie_sync::ConflictItem;
use thiserror::Error;

/// Result alias for API handlers.
pub type Result<T> = std::result::Result<T, ApiError>;

/// HTTP API errors.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("authentication failed")]
    Unauthorized,
    #[error("vault not found")]
    NotFound,
    #[error("sync conflict")]
    Conflict(Vec<ConflictItem>),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("server error: {0}")]
    Internal(String),
}

impl ApiError {
    /// Return the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_name(&self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::NotFound => "not_found",
            Self::Conflict(_) => "sync_conflict",
            Self::Validation(_) => "validation_error",
            Self::Database(_) => "database_error",
            Self::Internal(_) => "server_error",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let conflicts = match &self {
            Self::Conflict(conflicts) => Some(conflicts.clone()),
            _ => None,
        };
        let body = Json(ErrorResponse {
            error: self.error_name().to_string(),
            message: self.to_string(),
            conflicts,
        });

        (status, body).into_response()
    }
}
