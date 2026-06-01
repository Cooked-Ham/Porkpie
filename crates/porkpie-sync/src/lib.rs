//! Revision-based sync protocol for encrypted Porkpie vault data.

pub mod conflict;
pub mod errors;
pub mod protocol;
pub mod state;

pub use conflict::{detect_conflicts, merge_items, ConflictItem, MergeStrategy};
pub use errors::{Result, SyncError};
pub use protocol::{sync_vault, EncryptedSyncItem, SyncOutcome, SyncRequest, SyncResponse};
pub use state::SyncCursor;
