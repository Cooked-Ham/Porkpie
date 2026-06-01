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

    Ok(())
}

const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS vaults (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL UNIQUE,
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
    CREATE TABLE IF NOT EXISTS sync_state (
        vault_id TEXT PRIMARY KEY NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
        last_synced_revision INTEGER,
        last_synced_at INTEGER
    );
    "#,
    "CREATE INDEX IF NOT EXISTS idx_items_vault_id ON items(vault_id);",
    "CREATE INDEX IF NOT EXISTS idx_items_type ON items(item_type);",
    "CREATE INDEX IF NOT EXISTS idx_vaults_name ON vaults(name);",
];
