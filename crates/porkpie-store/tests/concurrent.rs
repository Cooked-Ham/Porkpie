mod common;

use common::test_pool;
use porkpie_core::Vault;
use porkpie_store::{load_vault, store_vault};

#[tokio::test]
async fn connection_pool_supports_concurrent_reads() {
    let pool = test_pool().await.expect("test database should connect");
    let vault = Vault::create("correct horse battery staple").expect("vault should be created");
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
