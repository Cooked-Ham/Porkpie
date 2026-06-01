use porkpie_api::{config::Config, db};
use sqlx::Row;

#[tokio::test]
async fn api_key_is_stored_as_hash_not_plaintext() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");

    let raw_key = "my-super-secret-api-key-12345";
    db::upsert_api_key(&pool, raw_key)
        .await
        .expect("upsert api key");

    let rows = sqlx::query("SELECT api_key_hash FROM api_keys")
        .fetch_all(&pool)
        .await
        .expect("query api keys");

    assert_eq!(rows.len(), 1);
    let stored_hash: String = rows[0].get("api_key_hash");

    assert_ne!(
        stored_hash, raw_key,
        "raw API key must not be stored in the database"
    );
    assert_eq!(
        stored_hash,
        db::hash_api_key(raw_key),
        "stored value must be the SHA-256 hash of the API key"
    );
    assert_eq!(
        stored_hash.len(),
        64,
        "SHA-256 hex digest must be 64 characters"
    );
}

#[tokio::test]
async fn api_key_validation_uses_hash_comparison() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");

    let raw_key = "correct-key-abc";
    db::upsert_api_key(&pool, raw_key)
        .await
        .expect("upsert api key");

    assert!(
        db::api_key_exists(&pool, raw_key).await.expect("check key"),
        "correct key must validate"
    );
    assert!(
        !db::api_key_exists(&pool, "wrong-key-xyz")
            .await
            .expect("check key"),
        "wrong key must not validate"
    );
}

#[tokio::test]
async fn different_api_keys_produce_different_hashes() {
    let hash_a = db::hash_api_key("key-alpha");
    let hash_b = db::hash_api_key("key-beta");
    assert_ne!(hash_a, hash_b);
}

#[test]
fn config_rejects_missing_api_key_env() {
    // Remove both API_KEY variants from the environment if they exist.
    let previous_api_key = std::env::var("API_KEY").ok();
    let previous_porkpie_api_key = std::env::var("PORKPIE_API_KEY").ok();
    std::env::remove_var("API_KEY");
    std::env::remove_var("PORKPIE_API_KEY");

    let result = Config::from_env();
    assert!(
        result.is_err(),
        "Config must fail when API_KEY / PORKPIE_API_KEY environment variable is missing"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("PORKPIE_API_KEY"),
        "error should mention PORKPIE_API_KEY: {err}"
    );

    // Restore the previous values so other tests are not affected.
    if let Some(value) = previous_api_key {
        std::env::set_var("API_KEY", value);
    }
    if let Some(value) = previous_porkpie_api_key {
        std::env::set_var("PORKPIE_API_KEY", value);
    }
}
