use porkpie_api::{config::Config, db};
use sqlx::Row;

#[tokio::test]
async fn api_key_is_stored_as_hash_not_plaintext() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");

    let raw_key = "my-super-secret-api-key-12345";
    db::upsert_api_key(&pool, raw_key, "default")
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
    db::upsert_api_key(&pool, raw_key, "default")
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

#[tokio::test]
async fn api_key_creation_returns_key_id_and_hash() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");

    let raw_key = "my-test-api-key-123";
    let (key_id, key_hash) = db::upsert_api_key(&pool, raw_key, "test-label")
        .await
        .expect("upsert api key");

    assert!(key_id > 0, "key_id should be positive");
    assert_eq!(key_hash, db::hash_api_key(raw_key));

    // Verify label is stored
    let rows = sqlx::query("SELECT label FROM api_keys WHERE id = ?")
        .bind(key_id)
        .fetch_all(&pool)
        .await
        .expect("query api keys");
    assert_eq!(rows.len(), 1);
    let label: String = rows[0].get("label");
    assert_eq!(label, "test-label");
}

#[tokio::test]
async fn last_used_at_is_updated_on_auth() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");

    let raw_key = "my-test-api-key-456";
    db::upsert_api_key(&pool, raw_key, "default")
        .await
        .expect("upsert api key");

    // Verify last_used_at is NULL initially
    let rows = sqlx::query("SELECT last_used_at FROM api_keys WHERE api_key_hash = ?")
        .bind(db::hash_api_key(raw_key))
        .fetch_all(&pool)
        .await
        .expect("query api keys");
    let last_used: Option<i64> = rows[0].get("last_used_at");
    assert!(last_used.is_none(), "last_used_at should be null initially");

    // Touch the key
    db::touch_api_key(&pool, raw_key)
        .await
        .expect("touch api key");

    // Verify last_used_at is now set
    let rows = sqlx::query("SELECT last_used_at FROM api_keys WHERE api_key_hash = ?")
        .bind(db::hash_api_key(raw_key))
        .fetch_all(&pool)
        .await
        .expect("query api keys");
    let last_used: Option<i64> = rows[0].get("last_used_at");
    assert!(
        last_used.is_some(),
        "last_used_at should be set after touch"
    );
}

#[tokio::test]
async fn revoke_by_id_sets_revoked_at() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");

    let raw_key = "my-test-api-key-789";
    let (key_id, _) = db::upsert_api_key(&pool, raw_key, "default")
        .await
        .expect("upsert api key");

    db::revoke_api_key_by_id(&pool, key_id)
        .await
        .expect("revoke api key");

    // Verify revoked_at is set
    let rows = sqlx::query("SELECT revoked_at, active FROM api_keys WHERE id = ?")
        .bind(key_id)
        .fetch_all(&pool)
        .await
        .expect("query api keys");
    let revoked_at: Option<i64> = rows[0].get("revoked_at");
    let active: i32 = rows[0].get("active");
    assert!(revoked_at.is_some(), "revoked_at should be set");
    assert_eq!(active, 0, "active should be 0");
}

#[tokio::test]
async fn multiple_api_keys_can_coexist() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");

    let key1 = "api-key-alpha";
    let key2 = "api-key-beta";

    db::upsert_api_key(&pool, key1, "alpha")
        .await
        .expect("upsert key1");
    db::upsert_api_key(&pool, key2, "beta")
        .await
        .expect("upsert key2");

    assert!(db::api_key_exists(&pool, key1).await.expect("check key1"));
    assert!(db::api_key_exists(&pool, key2).await.expect("check key2"));

    let count = db::count_active_api_keys(&pool)
        .await
        .expect("count active");
    assert_eq!(count, 2);
}

#[tokio::test]
async fn admin_flag_can_be_set_and_checked() {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect database");
    db::run_migrations(&pool).await.expect("run migrations");

    let raw_key = "admin-test-key";
    let (key_id, key_hash) = db::upsert_api_key(&pool, raw_key, "admin")
        .await
        .expect("upsert api key");

    // Initially not admin
    assert!(
        !db::api_key_is_admin(&pool, raw_key).await.expect("check admin"),
        "new key should not be admin by default"
    );

    // Set admin
    db::set_api_key_admin(&pool, &key_hash, true)
        .await
        .expect("set admin");

    assert!(
        db::api_key_is_admin(&pool, raw_key).await.expect("check admin"),
        "key should be admin after setting"
    );

    // Revoke and check
    db::revoke_api_key_by_id(&pool, key_id).await.expect("revoke");
    assert!(
        !db::api_key_is_admin(&pool, raw_key).await.expect("check admin after revoke"),
        "revoked key should not be admin"
    );
}
