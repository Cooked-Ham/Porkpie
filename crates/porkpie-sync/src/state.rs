use serde::{Deserialize, Serialize};

/// Client-side cursor for a vault sync stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncCursor {
    pub last_revision: u64,
}

impl SyncCursor {
    /// Create a cursor at the beginning of the stream.
    pub fn new() -> Self {
        Self { last_revision: 0 }
    }

    /// Advance the cursor after a successful sync response.
    pub fn advance(&mut self, revision: u64) {
        self.last_revision = self.last_revision.max(revision);
    }
}

impl Default for SyncCursor {
    fn default() -> Self {
        Self::new()
    }
}
