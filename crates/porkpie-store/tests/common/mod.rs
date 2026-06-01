use porkpie_store::{connect_database, Result};
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn test_pool() -> Result<SqlitePool> {
    let mut path = std::env::temp_dir();
    path.push(format!("porkpie-store-test-{}.db", Uuid::new_v4()));
    let database_url = format!("sqlite://{}", path.display().to_string().replace('\\', "/"));
    connect_database(&database_url).await
}
