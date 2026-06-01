use thiserror::Error;

/// Result alias for sync protocol operations.
pub type Result<T> = std::result::Result<T, SyncError>;

/// Errors returned by sync protocol helpers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SyncError {
    #[error("sync conflict detected for {count} item(s)")]
    Conflict { count: usize },
    #[error("invalid revision window: local base {base_revision}, server {server_revision}")]
    InvalidRevisionWindow {
        base_revision: u64,
        server_revision: u64,
    },
}
