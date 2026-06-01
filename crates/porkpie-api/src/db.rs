use crate::errors::{ApiError, Result};
use porkpie_sync::{ConflictItem, EncryptedSyncItem, MergeStrategy};
use porkpie_types::VaultId;
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Executor, Row, SqlitePool,
};
use std::str::FromStr;

/// Connect to SQLite and create the file when needed.
pub async fn connect(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|error| ApiError::Internal(error.to_string()))?
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(ApiError::from)
}

/// Run idempotent API server migrations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    pool.execute("PRAGMA foreign_keys = ON;").await?;
    for statement in MIGRATIONS {
        pool.execute(*statement).await?;
    }
    Ok(())
}

/// Hash an API key using SHA-256 for storage at rest.
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Insert or refresh a hashed API key.
pub async fn upsert_api_key(pool: &SqlitePool, api_key: &str) -> Result<()> {
    let key_hash = hash_api_key(api_key);
    sqlx::query(
        r#"
        INSERT INTO api_keys (api_key_hash, active, created_at)
        VALUES (?, 1, strftime('%s', 'now'))
        ON CONFLICT(api_key_hash) DO UPDATE SET active = 1
        "#,
    )
    .bind(&key_hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// Return true when the API key hash matches an active entry.
pub async fn api_key_exists(pool: &SqlitePool, api_key: &str) -> Result<bool> {
    let key_hash = hash_api_key(api_key);
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM api_keys WHERE api_key_hash = ? AND active = 1")
            .bind(&key_hash)
            .fetch_one(pool)
            .await?;

    Ok(count.0 > 0)
}

/// Seed vault metadata on the encrypted server store.
pub async fn upsert_vault_metadata(pool: &SqlitePool, vault_id: &str) -> Result<()> {
    validate_vault_id(vault_id)?;
    sqlx::query(
        r#"
        INSERT INTO vaults (id, created_at, salt, master_key_wrapped, sync_revision)
        VALUES (?, strftime('%s', 'now') * 1000, zeroblob(32), x'', 0)
        ON CONFLICT(id) DO NOTHING
        "#,
    )
    .bind(vault_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Load encrypted items changed after a client revision.
pub async fn load_items_since(
    pool: &SqlitePool,
    vault_id: &str,
    last_revision: u64,
) -> Result<(Vec<EncryptedSyncItem>, u64)> {
    validate_vault_exists(pool, vault_id).await?;

    let rows = sqlx::query(
        r#"
        SELECT id, item_type, ciphertext, created_at, updated_at, sync_revision
        FROM items
        WHERE vault_id = ? AND sync_revision > ?
        ORDER BY sync_revision, id
        "#,
    )
    .bind(vault_id)
    .bind(u64_to_i64(last_revision))
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|row| EncryptedSyncItem {
            item_id: row.get::<String, _>("id"),
            item_type: row.get::<String, _>("item_type"),
            ciphertext: row.get::<Vec<u8>, _>("ciphertext"),
            created_at: row.get::<i64, _>("created_at"),
            updated_at: row.get::<i64, _>("updated_at"),
            sync_revision: i64_to_u64(row.get::<i64, _>("sync_revision")),
        })
        .collect();

    Ok((items, vault_revision(pool, vault_id).await?))
}

/// Apply encrypted item changes to the server store.
pub async fn push_items(
    pool: &SqlitePool,
    vault_id: &str,
    base_revision: u64,
    items: &[EncryptedSyncItem],
    strategy: MergeStrategy,
) -> Result<(usize, u64, Vec<ConflictItem>)> {
    validate_vault_exists(pool, vault_id).await?;
    for item in items {
        validate_item(item)?;
    }

    let mut conflicts = Vec::new();
    for item in items {
        if let Some(server_item) = load_item(pool, vault_id, &item.item_id).await? {
            let server_changed = server_item.sync_revision > base_revision;
            let differs = server_item.ciphertext != item.ciphertext;
            if server_changed && differs && strategy != MergeStrategy::LastWriteWins {
                conflicts.push(ConflictItem {
                    item_id: item.item_id.clone(),
                    local_revision: item.sync_revision,
                    server_revision: server_item.sync_revision,
                    server_data: server_item.ciphertext,
                });
            }
        }
    }

    if !conflicts.is_empty() {
        return Ok((0, vault_revision(pool, vault_id).await?, conflicts));
    }

    let mut revision = vault_revision(pool, vault_id).await?;
    let mut accepted = 0usize;
    for item in items {
        revision = revision.saturating_add(1);
        upsert_item(pool, vault_id, item, revision).await?;
        accepted = accepted.saturating_add(1);
    }
    update_vault_revision(pool, vault_id, revision).await?;
    log_audit(pool, vault_id, "sync_push").await?;

    Ok((accepted, revision, Vec::new()))
}

async fn validate_vault_exists(pool: &SqlitePool, vault_id: &str) -> Result<()> {
    validate_vault_id(vault_id)?;
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vaults WHERE id = ?")
        .bind(vault_id)
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        Err(ApiError::NotFound)
    } else {
        Ok(())
    }
}

fn validate_vault_id(vault_id: &str) -> Result<()> {
    VaultId::from_string(vault_id).map_err(|_| ApiError::NotFound)?;
    Ok(())
}

fn validate_item(item: &EncryptedSyncItem) -> Result<()> {
    if item.item_id.trim().is_empty() {
        return Err(ApiError::Validation("item_id is required".to_string()));
    }
    if item.item_type.trim().is_empty() {
        return Err(ApiError::Validation("item_type is required".to_string()));
    }
    if item.ciphertext.is_empty() {
        return Err(ApiError::Validation("ciphertext is required".to_string()));
    }
    Ok(())
}

async fn load_item(
    pool: &SqlitePool,
    vault_id: &str,
    item_id: &str,
) -> Result<Option<EncryptedSyncItem>> {
    let row = sqlx::query(
        r#"
        SELECT id, item_type, ciphertext, created_at, updated_at, sync_revision
        FROM items
        WHERE vault_id = ? AND id = ?
        "#,
    )
    .bind(vault_id)
    .bind(item_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| EncryptedSyncItem {
        item_id: row.get::<String, _>("id"),
        item_type: row.get::<String, _>("item_type"),
        ciphertext: row.get::<Vec<u8>, _>("ciphertext"),
        created_at: row.get::<i64, _>("created_at"),
        updated_at: row.get::<i64, _>("updated_at"),
        sync_revision: i64_to_u64(row.get::<i64, _>("sync_revision")),
    }))
}

async fn upsert_item(
    pool: &SqlitePool,
    vault_id: &str,
    item: &EncryptedSyncItem,
    revision: u64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO items (
            id, vault_id, item_type, ciphertext, created_at, updated_at, sync_revision
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            item_type = excluded.item_type,
            ciphertext = excluded.ciphertext,
            updated_at = excluded.updated_at,
            sync_revision = excluded.sync_revision
        "#,
    )
    .bind(item.item_id.as_str())
    .bind(vault_id)
    .bind(item.item_type.as_str())
    .bind(item.ciphertext.as_slice())
    .bind(item.created_at)
    .bind(item.updated_at)
    .bind(u64_to_i64(revision))
    .execute(pool)
    .await?;

    Ok(())
}

async fn vault_revision(pool: &SqlitePool, vault_id: &str) -> Result<u64> {
    let revision: (i64,) = sqlx::query_as("SELECT sync_revision FROM vaults WHERE id = ?")
        .bind(vault_id)
        .fetch_one(pool)
        .await?;

    Ok(i64_to_u64(revision.0))
}

async fn update_vault_revision(pool: &SqlitePool, vault_id: &str, revision: u64) -> Result<()> {
    sqlx::query("UPDATE vaults SET sync_revision = ? WHERE id = ?")
        .bind(u64_to_i64(revision))
        .bind(vault_id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn log_audit(pool: &SqlitePool, vault_id: &str, event: &str) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (vault_id, event, created_at)
        VALUES (?, ?, strftime('%s', 'now'))
        "#,
    )
    .bind(vault_id)
    .bind(event)
    .execute(pool)
    .await?;

    Ok(())
}

fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

fn u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS vaults (
        id TEXT PRIMARY KEY NOT NULL,
        created_at INTEGER NOT NULL,
        salt BLOB NOT NULL,
        master_key_wrapped BLOB NOT NULL,
        sync_revision INTEGER NOT NULL DEFAULT 0,
        created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS items (
        id TEXT PRIMARY KEY NOT NULL,
        vault_id TEXT NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
        item_type TEXT NOT NULL,
        ciphertext BLOB NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        sync_revision INTEGER NOT NULL DEFAULT 0,
        created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS api_keys (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        api_key_hash TEXT NOT NULL UNIQUE,
        active INTEGER NOT NULL DEFAULT 1,
        created_at INTEGER NOT NULL
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS audit_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        vault_id TEXT,
        event TEXT NOT NULL,
        created_at INTEGER NOT NULL
    );
    "#,
    "CREATE INDEX IF NOT EXISTS idx_items_vault_revision ON items(vault_id, sync_revision);",
    "CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(api_key_hash, active);",
    "CREATE INDEX IF NOT EXISTS idx_audit_vault ON audit_log(vault_id, created_at);",
];
