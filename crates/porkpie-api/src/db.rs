use crate::errors::{ApiError, Result};
use crate::models::VaultMetadataResponse;
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
    migrate_items_to_composite_pk(pool).await?;
    migrate_api_keys_metadata(pool).await?;
    migrate_vault_kdf_columns(pool).await?;
    Ok(())
}

async fn migrate_items_to_composite_pk(pool: &SqlitePool) -> Result<()> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'items'")
            .fetch_optional(pool)
            .await?;

    if let Some((sql,)) = row {
        if sql.contains("id TEXT PRIMARY KEY") && !sql.contains("PRIMARY KEY (vault_id, id)") {
            pool.execute(
                r#"
                CREATE TABLE items_v2 (
                    id TEXT NOT NULL,
                    vault_id TEXT NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
                    item_type TEXT NOT NULL,
                    ciphertext BLOB NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    sync_revision INTEGER NOT NULL DEFAULT 0,
                    created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (vault_id, id)
                )
            "#,
            )
            .await?;
            pool.execute("INSERT OR IGNORE INTO items_v2 SELECT * FROM items")
                .await?;
            pool.execute("DROP TABLE items").await?;
            pool.execute("ALTER TABLE items_v2 RENAME TO items").await?;
            pool.execute(
                "CREATE INDEX IF NOT EXISTS idx_items_vault_revision ON items(vault_id, sync_revision);"
            )
            .await?;
        }
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
/// Returns the key ID and hash.
pub async fn upsert_api_key(
    pool: &SqlitePool,
    api_key: &str,
    label: &str,
) -> Result<(i64, String)> {
    let key_hash = hash_api_key(api_key);
    let id: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO api_keys (api_key_hash, label, active, created_at)
        VALUES (?, ?, 1, strftime('%s', 'now'))
        ON CONFLICT(api_key_hash) DO UPDATE SET active = 1, revoked_at = NULL
        RETURNING id
        "#,
    )
    .bind(&key_hash)
    .bind(label)
    .fetch_one(pool)
    .await?;

    Ok((id.0, key_hash))
}

/// Return true when the API key hash matches an active entry.
///
/// Uses constant-time comparison to avoid timing side-channels.
/// All active hashes are fetched from the database and compared
/// in Rust with `subtle::ConstantTimeEq`.
pub async fn api_key_exists(pool: &SqlitePool, api_key: &str) -> Result<bool> {
    let key_hash = hash_api_key(api_key);
    let expected_bytes =
        hex::decode(&key_hash).map_err(|_| ApiError::Internal("invalid hash".to_string()))?;
    if expected_bytes.len() != 32 {
        return Err(ApiError::Internal("invalid hash length".to_string()));
    }

    let rows: Vec<(String,)> = sqlx::query_as("SELECT api_key_hash FROM api_keys WHERE active = 1")
        .fetch_all(pool)
        .await?;

    for (stored_hash,) in rows {
        if let Ok(stored_bytes) = hex::decode(&stored_hash) {
            if stored_bytes.len() == 32
                && subtle::ConstantTimeEq::ct_eq(expected_bytes.as_slice(), stored_bytes.as_slice())
                    .into()
            {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Deactivate an API key so it can no longer authenticate.
pub async fn revoke_api_key(pool: &SqlitePool, api_key: &str) -> Result<()> {
    let key_hash = hash_api_key(api_key);
    revoke_api_key_by_hash(pool, &key_hash).await
}

/// Deactivate an API key by its pre-computed hash.
pub async fn revoke_api_key_by_hash(pool: &SqlitePool, key_hash: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE api_keys SET active = 0, revoked_at = strftime('%s', 'now') WHERE api_key_hash = ?
        "#,
    )
    .bind(key_hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// Deactivate an API key by its ID.
/// Returns the hash of the revoked key.
pub async fn revoke_api_key_by_id(pool: &SqlitePool, key_id: i64) -> Result<String> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT api_key_hash FROM api_keys WHERE id = ? AND active = 1")
            .bind(key_id)
            .fetch_optional(pool)
            .await?;

    let key_hash = row.ok_or(ApiError::NotFound)?;

    sqlx::query(
        r#"
        UPDATE api_keys SET active = 0, revoked_at = strftime('%s', 'now') WHERE id = ?
        "#,
    )
    .bind(key_id)
    .execute(pool)
    .await?;

    Ok(key_hash.0)
}

/// Count active API keys.
pub async fn count_active_api_keys(pool: &SqlitePool) -> Result<i64> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api_keys WHERE active = 1")
        .fetch_one(pool)
        .await?;
    Ok(count.0)
}

/// Check if an API key has admin privileges.
pub async fn api_key_is_admin(pool: &SqlitePool, api_key: &str) -> Result<bool> {
    let key_hash = hash_api_key(api_key);
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT is_admin FROM api_keys WHERE api_key_hash = ? AND active = 1")
            .bind(&key_hash)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0 != 0).unwrap_or(false))
}

/// Update the admin flag for an API key by its hash.
pub async fn set_api_key_admin(pool: &SqlitePool, key_hash: &str, is_admin: bool) -> Result<()> {
    sqlx::query("UPDATE api_keys SET is_admin = ? WHERE api_key_hash = ?")
        .bind(if is_admin { 1 } else { 0 })
        .bind(key_hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update last_used_at for an API key.
pub async fn touch_api_key(pool: &SqlitePool, api_key: &str) -> Result<()> {
    let key_hash = hash_api_key(api_key);
    sqlx::query("UPDATE api_keys SET last_used_at = strftime('%s', 'now') WHERE api_key_hash = ?")
        .bind(&key_hash)
        .execute(pool)
        .await?;
    Ok(())
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

/// Register a vault with its full cryptographic metadata. The
/// `upsert_vault_metadata()` helper only creates a stub; this
/// one replaces the salt and wrapped key with real values from
/// the client.
#[allow(clippy::too_many_arguments)]
pub async fn register_vault(
    pool: &SqlitePool,
    vault_id: &str,
    name: &str,
    salt: &[u8],
    master_key_wrapped: &[u8],
    created_at: i64,
    kdf_time_cost: u32,
    kdf_mem_cost: u32,
    kdf_parallelism: u32,
) -> Result<()> {
    validate_vault_id(vault_id)?;
    sqlx::query(
        r#"
        INSERT INTO vaults (id, name, created_at, salt, master_key_wrapped, sync_revision, kdf_time_cost, kdf_mem_cost, kdf_parallelism)
        VALUES (?, ?, ?, ?, ?, 0, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            salt = excluded.salt,
            master_key_wrapped = excluded.master_key_wrapped,
            created_at = excluded.created_at,
            kdf_time_cost = excluded.kdf_time_cost,
            kdf_mem_cost = excluded.kdf_mem_cost,
            kdf_parallelism = excluded.kdf_parallelism
        "#,
    )
    .bind(vault_id)
    .bind(name)
    .bind(created_at)
    .bind(salt)
    .bind(master_key_wrapped)
    .bind(kdf_time_cost)
    .bind(kdf_mem_cost)
    .bind(kdf_parallelism)
    .execute(pool)
    .await?;

    log_audit(pool, vault_id, "vault_register").await?;
    Ok(())
}

/// Load a vault's metadata (encrypted blobs only) for the sync
/// pull response that Peer B needs to reconstruct the locked
/// vault on their side.
pub async fn load_vault_metadata(
    pool: &SqlitePool,
    vault_id: &str,
) -> Result<VaultMetadataResponse> {
    validate_vault_exists(pool, vault_id).await?;

    let row = sqlx::query_as::<_, (String, String, i64, Vec<u8>, Vec<u8>, i64, i64, i64, i64)>(
        r#"
        SELECT id, name, created_at, salt, master_key_wrapped, sync_revision, kdf_time_cost, kdf_mem_cost, kdf_parallelism
        FROM vaults
        WHERE id = ?
        "#,
    )
    .bind(vault_id)
    .fetch_one(pool)
    .await?;

    Ok(VaultMetadataResponse {
        vault_id: row.0,
        name: row.1,
        created_at: row.2,
        salt: row.3,
        master_key_wrapped: row.4,
        sync_revision: i64_to_u64(row.5),
        kdf_time_cost: i64_to_u32(row.6),
        kdf_mem_cost: i64_to_u32(row.7),
        kdf_parallelism: i64_to_u32(row.8),
    })
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

/// Detect obvious plaintext patterns in what should be an encrypted blob.
/// Reject payloads that contain human-readable secret field names or JSON
/// structure, because real ciphertext should be opaque binary data.
fn detect_plaintext_payload(ciphertext: &[u8]) -> Option<&'static str> {
    let text = String::from_utf8_lossy(ciphertext);
    // Check for obvious JSON structure that indicates plaintext.
    if text.contains("\"") && text.contains(":") && text.contains("{") {
        const SENSITIVE_FIELDS: &[&str] = &[
            "username",
            "password",
            "private_key",
            "api_key",
            "totp",
            "notes",
        ];
        for field in SENSITIVE_FIELDS {
            if text.contains(field) {
                return Some(field);
            }
        }
    }
    None
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
    if let Some(field) = detect_plaintext_payload(&item.ciphertext) {
        return Err(ApiError::Validation(format!(
            "ciphertext appears to contain plaintext field '{field}'"
        )));
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
        ON CONFLICT(vault_id, id) DO UPDATE SET
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

fn i64_to_u32(value: i64) -> u32 {
    u32::try_from(value).unwrap_or(0)
}

fn u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS vaults (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL DEFAULT '',
        created_at INTEGER NOT NULL,
        salt BLOB NOT NULL,
        master_key_wrapped BLOB NOT NULL,
        sync_revision INTEGER NOT NULL DEFAULT 0,
        kdf_time_cost INTEGER NOT NULL DEFAULT 2,
        kdf_mem_cost INTEGER NOT NULL DEFAULT 19456,
        kdf_parallelism INTEGER NOT NULL DEFAULT 1,
        created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS items (
        id TEXT NOT NULL,
        vault_id TEXT NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
        item_type TEXT NOT NULL,
        ciphertext BLOB NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        sync_revision INTEGER NOT NULL DEFAULT 0,
        created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (vault_id, id)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS api_keys (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        api_key_hash TEXT NOT NULL UNIQUE,
        label TEXT NOT NULL DEFAULT '',
        active INTEGER NOT NULL DEFAULT 1,
        is_admin INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL,
        revoked_at INTEGER,
        last_used_at INTEGER
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

async fn migrate_vault_kdf_columns(pool: &SqlitePool) -> Result<()> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'vaults'")
            .fetch_optional(pool)
            .await?;

    if let Some((sql,)) = row {
        if !sql.contains("kdf_time_cost") {
            sqlx::query("ALTER TABLE vaults ADD COLUMN kdf_time_cost INTEGER NOT NULL DEFAULT 2")
                .execute(pool)
                .await?;
        }
        if !sql.contains("kdf_mem_cost") {
            sqlx::query(
                "ALTER TABLE vaults ADD COLUMN kdf_mem_cost INTEGER NOT NULL DEFAULT 19456",
            )
            .execute(pool)
            .await?;
        }
        if !sql.contains("kdf_parallelism") {
            sqlx::query("ALTER TABLE vaults ADD COLUMN kdf_parallelism INTEGER NOT NULL DEFAULT 1")
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

async fn migrate_api_keys_metadata(pool: &SqlitePool) -> Result<()> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'api_keys'")
            .fetch_optional(pool)
            .await?;

    if let Some((sql,)) = row {
        if !sql.contains("label") {
            sqlx::query("ALTER TABLE api_keys ADD COLUMN label TEXT NOT NULL DEFAULT ''")
                .execute(pool)
                .await?;
            sqlx::query("ALTER TABLE api_keys ADD COLUMN revoked_at INTEGER")
                .execute(pool)
                .await?;
            sqlx::query("ALTER TABLE api_keys ADD COLUMN last_used_at INTEGER")
                .execute(pool)
                .await?;
        }
        if !sql.contains("is_admin") {
            sqlx::query("ALTER TABLE api_keys ADD COLUMN is_admin INTEGER NOT NULL DEFAULT 0")
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}
