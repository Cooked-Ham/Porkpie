use crate::errors::{map_sqlx_error, Result};
use crate::migrations::run_migrations;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

const DEFAULT_DATABASE_URL: &str = "sqlite:porkpie.db";

/// Connect to the default local SQLite database and run migrations.
pub async fn connect() -> Result<SqlitePool> {
    let database_url =
        std::env::var("PORKPIE_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    connect_database(&database_url).await
}

/// Connect to a SQLite database URL and run migrations.
pub async fn connect_database(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(map_sqlx_error)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(map_sqlx_error)?;

    run_migrations(&pool).await?;
    Ok(pool)
}
