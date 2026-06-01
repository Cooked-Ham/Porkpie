use crate::errors::{map_sqlx_error, Result};
use sqlx::{Executor, SqlitePool};

/// Run idempotent SQLite schema migrations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    pool.execute("PRAGMA foreign_keys = ON;")
        .await
        .map_err(map_sqlx_error)?;

    for statement in MIGRATIONS {
        pool.execute(*statement).await.map_err(map_sqlx_error)?;
    }

    migrate_items_to_composite_pk(pool).await?;
    migrate_vaults_kdf_params(pool).await?;
    Ok(())
}

/// Client-side migrations.
///
/// The `porkpie-store` crate manages the schema for local vaults and sync
/// state. API-only tables (`api_keys`, `audit_log`) are defined in the
/// `porkpie-api` crate because they are server-specific and not needed for
/// local/desktop storage.
const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS vaults (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL UNIQUE,
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
    CREATE TABLE IF NOT EXISTS sync_state (
        vault_id TEXT PRIMARY KEY NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
        last_synced_revision INTEGER,
        last_synced_at INTEGER
    );
    "#,
    "CREATE INDEX IF NOT EXISTS idx_items_vault_revision ON items(vault_id, sync_revision);",
    "CREATE INDEX IF NOT EXISTS idx_items_type ON items(item_type);",
    "CREATE INDEX IF NOT EXISTS idx_vaults_name ON vaults(name);",
];

async fn migrate_items_to_composite_pk(pool: &SqlitePool) -> Result<()> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'items'")
            .fetch_optional(pool)
            .await
            .map_err(map_sqlx_error)?;

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
            .await
            .map_err(map_sqlx_error)?;
            pool.execute("INSERT OR IGNORE INTO items_v2 SELECT * FROM items")
                .await
                .map_err(map_sqlx_error)?;
            pool.execute("DROP TABLE items")
                .await
                .map_err(map_sqlx_error)?;
            pool.execute("ALTER TABLE items_v2 RENAME TO items")
                .await
                .map_err(map_sqlx_error)?;
            pool.execute(
                "CREATE INDEX IF NOT EXISTS idx_items_vault_revision ON items(vault_id, sync_revision);",
            )
            .await
            .map_err(map_sqlx_error)?;
        }
    }
    Ok(())
}

async fn migrate_vaults_kdf_params(pool: &SqlitePool) -> Result<()> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'vaults'")
            .fetch_optional(pool)
            .await
            .map_err(map_sqlx_error)?;

    if let Some((sql,)) = row {
        if !sql.contains("kdf_time_cost") {
            sqlx::query("ALTER TABLE vaults ADD COLUMN kdf_time_cost INTEGER NOT NULL DEFAULT 2")
                .execute(pool)
                .await
                .map_err(map_sqlx_error)?;
            sqlx::query(
                "ALTER TABLE vaults ADD COLUMN kdf_mem_cost INTEGER NOT NULL DEFAULT 19456",
            )
            .execute(pool)
            .await
            .map_err(map_sqlx_error)?;
            sqlx::query("ALTER TABLE vaults ADD COLUMN kdf_parallelism INTEGER NOT NULL DEFAULT 1")
                .execute(pool)
                .await
                .map_err(map_sqlx_error)?;
        }
    }
    Ok(())
}
