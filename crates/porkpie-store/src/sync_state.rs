//! Sync state persistence. Tracks the last revision that was
//! successfully synced to the server, so the next sync only
//! transfers items that changed after that point.

use crate::errors::{map_sqlx_error, Result};
use crate::models::{i64_to_u64, u64_to_i64};
use porkpie_types::{Timestamp, VaultId};
use sqlx::SqlitePool;

/// Load the last known sync state for a vault. Returns `None`
/// when the vault has never been synced.
pub async fn load_sync_state(
    pool: &SqlitePool,
    vault_id: &VaultId,
) -> Result<Option<super::SyncState>> {
    let row = sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
        r#"
        SELECT last_synced_revision, last_synced_at
        FROM sync_state
        WHERE vault_id = ?
        "#,
    )
    .bind(vault_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(row.map(|(revision, synced_at)| super::SyncState {
        vault_id: *vault_id,
        last_synced_revision: revision.map(i64_to_u64),
        last_synced_at: synced_at.map(Timestamp),
    }))
}

/// Persist the sync state for a vault. Creates or replaces the
/// row so the cursor always reflects the last completed sync.
pub async fn save_sync_state(pool: &SqlitePool, state: &super::SyncState) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sync_state (vault_id, last_synced_revision, last_synced_at)
        VALUES (?, ?, ?)
        ON CONFLICT(vault_id) DO UPDATE SET
            last_synced_revision = excluded.last_synced_revision,
            last_synced_at = excluded.last_synced_at
        "#,
    )
    .bind(state.vault_id.to_string())
    .bind(state.last_synced_revision.map(u64_to_i64))
    .bind(state.last_synced_at.map(|t| t.to_millis()))
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(())
}
