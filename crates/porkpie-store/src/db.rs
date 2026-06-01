use crate::errors::{map_sqlx_error, Result, StoreError};
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

/// Build a SQLx-compatible SQLite URL from a filesystem path.
///
/// On Windows `PathBuf` contains backslashes which SQLx cannot parse as a URL.
/// This helper converts the path to a forward-slash string before building the
/// `sqlite://` URI.
pub fn sqlite_url_from_path(path: &std::path::Path) -> String {
    let path_str = path.as_os_str().to_string_lossy().replace('\\', "/");
    format!("sqlite://{path_str}?mode=rwc")
}

/// Run a startup self-check on the database.
///
/// 1. Open the database.
/// 2. Run migrations.
/// 3. Execute `SELECT 1`.
///
/// Returns a detailed error categorising the exact failure so the caller can
/// surface a helpful message in the UI or CLI.
pub async fn startup_self_check(database_url: &str) -> Result<SqlitePool> {
    #[cfg(debug_assertions)]
    eprintln!("[porkpie] startup self-check: database_url={database_url}");

    // Step 1: Validate the URL can be parsed.
    let options = match SqliteConnectOptions::from_str(database_url) {
        Ok(opts) => opts,
        Err(error) => {
            #[cfg(debug_assertions)]
            eprintln!("[porkpie] startup self-check failed: invalid URL: {error}");
            return Err(StoreError::InvalidUrl(error.to_string()));
        }
    }
    .create_if_missing(true)
    .foreign_keys(true)
    .journal_mode(SqliteJournalMode::Wal);

    // Step 2: Create parent directories for file-based URLs.
    // SQLx create_if_missing only creates the file, not the parent directories.
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        let path = path.split('?').next().unwrap_or(path);
        if let Some(parent) = std::path::Path::new(path).parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                #[cfg(debug_assertions)]
                eprintln!("[porkpie] startup self-check failed: cannot create directory {parent:?}: {error}");
                return Err(StoreError::CannotCreateDirectory(format!(
                    "{parent:?}: {error}"
                )));
            }
        }
    }

    // Step 3: Open the pool.
    let pool = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            let message = error.to_string();
            #[cfg(debug_assertions)]
            eprintln!("[porkpie] startup self-check failed: cannot open database: {message}");
            if message.to_ascii_lowercase().contains("permission denied") {
                return Err(StoreError::PermissionDenied(message));
            }
            return Err(StoreError::CannotOpenDatabase(message));
        }
    };

    // Step 4: Run migrations.
    if let Err(error) = run_migrations(&pool).await {
        #[cfg(debug_assertions)]
        eprintln!("[porkpie] startup self-check failed: migration failed: {error}");
        return Err(StoreError::MigrationFailed(error.to_string()));
    }

    // Step 5: Execute a health query.
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => {}
        Err(error) => {
            #[cfg(debug_assertions)]
            eprintln!("[porkpie] startup self-check failed: health query failed: {error}");
            return Err(StoreError::CannotOpenDatabase(error.to_string()));
        }
    }

    #[cfg(debug_assertions)]
    eprintln!("[porkpie] startup self-check: passed");
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn sqlite_url_from_path_replaces_backslashes() {
        let path = PathBuf::from("C:\\Users\\Alice\\AppData\\Roaming\\Porkpie\\porkpie.db");
        let url = sqlite_url_from_path(&path);
        assert_eq!(
            url,
            "sqlite://C:/Users/Alice/AppData/Roaming/Porkpie/porkpie.db?mode=rwc"
        );
    }

    #[test]
    fn sqlite_url_from_path_unix_unchanged() {
        let path = PathBuf::from("/home/alice/.local/share/porkpie/porkpie.db");
        let url = sqlite_url_from_path(&path);
        assert_eq!(
            url,
            "sqlite:///home/alice/.local/share/porkpie/porkpie.db?mode=rwc"
        );
    }

    #[tokio::test]
    async fn startup_self_check_passes_on_memory_db() {
        let pool = startup_self_check("sqlite::memory:")
            .await
            .expect("should pass");
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("health query should work");
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn startup_self_check_fails_on_invalid_url() {
        // An HTTP URL is not a valid SQLite URL; SQLx will parse it but
        // SQLite cannot open the file, yielding CannotOpenDatabase.
        let result = startup_self_check("http://example.com/db.sqlite").await;
        assert!(
            matches!(
                result,
                Err(StoreError::InvalidUrl(_)) | Err(StoreError::CannotOpenDatabase(_))
            ),
            "expected InvalidUrl or CannotOpenDatabase error, got {result:?}"
        );
    }

    #[tokio::test]
    async fn startup_self_check_fails_on_unopenable_path() {
        // A path whose parent is a file (not a directory) cannot be created.
        // Use a temp file as the parent so the directory creation fails.
        let temp_file = std::env::temp_dir().join("porkpie-test-file-parent");
        let _ = std::fs::remove_file(&temp_file);
        std::fs::write(&temp_file, "x").expect("create temp file");
        let path = temp_file.join("porkpie.db");
        let url = sqlite_url_from_path(&path);
        let result = startup_self_check(&url).await;
        let _ = std::fs::remove_file(&temp_file);
        assert!(
            matches!(
                result,
                Err(StoreError::CannotCreateDirectory(_)) | Err(StoreError::CannotOpenDatabase(_))
            ),
            "expected CannotCreateDirectory or CannotOpenDatabase error, got {result:?}"
        );
    }

    #[tokio::test]
    async fn startup_self_check_creates_parent_directory() {
        let parent = std::env::temp_dir().join("porkpie-test-startup-check");
        let path = parent.join("deep").join("porkpie.db");
        let url = sqlite_url_from_path(&path);
        let pool = startup_self_check(&url)
            .await
            .expect("should create parent and open");
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("health query should work");
        assert_eq!(row.0, 1);
        // Clean up
        let _ = std::fs::remove_dir_all(&parent);
    }
}
