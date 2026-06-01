mod common;

use common::test_pool;
use porkpie_store::{store_item, EncryptedItemData, StoreError};
use porkpie_types::{ItemId, Timestamp, VaultId};
use sqlx::Row;

#[tokio::test]
async fn foreign_key_constraints_are_enforced() {
    let pool = test_pool().await.expect("test database should connect");
    let now = Timestamp::now();
    let item = EncryptedItemData::new(
        ItemId::new(),
        VaultId::new(),
        "SecureNote",
        b"encrypted-item".to_vec(),
        now,
        now,
        0,
    );

    assert!(matches!(
        store_item(&pool, &item).await,
        Err(StoreError::ConstraintViolation(_))
    ));
}

#[tokio::test]
async fn migrations_create_expected_indexes() {
    let pool = test_pool().await.expect("test database should connect");
    let rows = sqlx::query(
        r#"
        SELECT name
        FROM sqlite_master
        WHERE type = 'index' AND name IN ('idx_items_vault_id', 'idx_items_type')
        ORDER BY name
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("index query should run");

    let names = rows
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["idx_items_type", "idx_items_vault_id"]);
}
