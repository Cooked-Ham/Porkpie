mod common;

use common::test_pool;
use porkpie_core::{LocalSecretKey, Vault};
use porkpie_store::{load_vault, store_vault};

fn test_secret_key() -> LocalSecretKey {
    LocalSecretKey::from_hex("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        .unwrap()
}

#[tokio::test]
async fn connection_pool_supports_concurrent_reads() {
    let pool = test_pool().await.expect("test database should connect");
    let (vault, _) = Vault::create(
        "TestVault",
        "correct horse battery staple",
        &test_secret_key(),
    )
    .expect("vault should be created");
    store_vault(&pool, &vault)
        .await
        .expect("vault should store");

    let mut tasks = Vec::new();
    for _ in 0..8 {
        let pool = pool.clone();
        let vault_id = vault.id;
        tasks.push(tokio::spawn(async move {
            load_vault(&pool, &vault_id)
                .await
                .map(|loaded| loaded.id == vault_id)
        }));
    }

    for task in tasks {
        let read_succeeded = task.await.expect("reader task should join");
        assert!(read_succeeded.expect("reader should load vault"));
    }
}
