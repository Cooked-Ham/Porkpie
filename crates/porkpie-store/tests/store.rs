mod common;

use common::test_pool;
use porkpie_core::Vault;
use porkpie_store::{
    delete_item, delete_vault, load_item, load_item_record, load_items, load_vault, store_item,
    store_vault, update_item, EncryptedItemData, StoreError,
};
use porkpie_types::{ItemId, Timestamp};

#[tokio::test]
async fn store_and_load_vault_works() {
    let pool = test_pool().await.expect("test database should connect");
    let vault = Vault::create("correct horse battery staple").expect("vault should be created");

    store_vault(&pool, &vault)
        .await
        .expect("vault should store");
    let loaded = load_vault(&pool, &vault.id)
        .await
        .expect("vault should load");

    assert_eq!(loaded.id, vault.id);
    assert_eq!(loaded.created_at, vault.created_at);
    assert_eq!(loaded.salt, vault.salt);
    assert_eq!(loaded.master_key_wrapped, vault.master_key_wrapped);
    assert_eq!(loaded.sync_revision, vault.sync_revision);

    let mut locked_vault = loaded.into_locked_vault();
    locked_vault
        .unlock("correct horse battery staple")
        .expect("loaded metadata should unlock");
}

#[tokio::test]
async fn store_load_update_delete_item_works() {
    let pool = test_pool().await.expect("test database should connect");
    let vault = Vault::create("correct horse battery staple").expect("vault should be created");
    store_vault(&pool, &vault)
        .await
        .expect("vault should store");

    let item = encrypted_item(vault.id, b"encrypted-item-v1".to_vec());
    store_item(&pool, &item).await.expect("item should store");

    assert_eq!(
        load_item(&pool, &item.id)
            .await
            .expect("item ciphertext should load"),
        b"encrypted-item-v1".to_vec()
    );

    let loaded_record = load_item_record(&pool, &item.id)
        .await
        .expect("item row should load");
    assert_eq!(loaded_record, item);

    let listed = load_items(&pool, &vault.id)
        .await
        .expect("vault items should load");
    assert_eq!(listed, vec![(item.id, b"encrypted-item-v1".to_vec())]);

    update_item(&pool, &item.id, b"encrypted-item-v2")
        .await
        .expect("item should update");
    assert_eq!(
        load_item(&pool, &item.id)
            .await
            .expect("updated ciphertext should load"),
        b"encrypted-item-v2".to_vec()
    );

    delete_item(&pool, &item.id)
        .await
        .expect("item should delete");
    assert!(matches!(
        load_item(&pool, &item.id).await,
        Err(StoreError::ItemNotFound(_))
    ));
}

#[tokio::test]
async fn delete_vault_cascades_items() {
    let pool = test_pool().await.expect("test database should connect");
    let vault = Vault::create("correct horse battery staple").expect("vault should be created");
    store_vault(&pool, &vault)
        .await
        .expect("vault should store");
    let item = encrypted_item(vault.id, b"encrypted-item".to_vec());
    store_item(&pool, &item).await.expect("item should store");

    delete_vault(&pool, &vault.id)
        .await
        .expect("vault should delete");

    assert!(matches!(
        load_vault(&pool, &vault.id).await,
        Err(StoreError::VaultNotFound(_))
    ));
    assert!(matches!(
        load_item(&pool, &item.id).await,
        Err(StoreError::ItemNotFound(_))
    ));
}

fn encrypted_item(vault_id: porkpie_types::VaultId, ciphertext: Vec<u8>) -> EncryptedItemData {
    let now = Timestamp::now();
    EncryptedItemData::new(
        ItemId::new(),
        vault_id,
        "SecureNote",
        ciphertext,
        now,
        now,
        0,
    )
}
