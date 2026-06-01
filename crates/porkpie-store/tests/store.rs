mod common;

use common::test_pool;
use porkpie_core::{LocalSecretKey, Vault};
use porkpie_store::{
    delete_item, delete_vault, load_item, load_item_record, load_items, load_vault, store_item,
    store_vault, update_item, EncryptedItemData, StoreError,
};
use porkpie_types::{ItemId, Timestamp, VaultId};

fn test_secret_key() -> LocalSecretKey {
    LocalSecretKey::from_hex("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        .unwrap()
}

#[tokio::test]
async fn store_and_load_vault_works() {
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
    let loaded = load_vault(&pool, &vault.id)
        .await
        .expect("vault should load");

    assert_eq!(loaded.id, vault.id);
    assert_eq!(loaded.created_at, vault.created_at);
    assert_eq!(loaded.salt, vault.salt);
    assert_eq!(
        loaded.master_key_wrapped,
        vault.master_key_wrapped().clone()
    );
    assert_eq!(loaded.sync_revision, vault.sync_revision());

    let mut locked_vault = loaded.into_locked_vault();
    locked_vault
        .unlock("correct horse battery staple", &test_secret_key())
        .expect("loaded metadata should unlock");
}

#[tokio::test]
async fn store_load_update_delete_item_works() {
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

    let item = encrypted_item(vault.id, b"encrypted-item-v1".to_vec());
    store_item(&pool, &item).await.expect("item should store");

    assert_eq!(
        load_item(&pool, &vault.id, &item.id)
            .await
            .expect("item ciphertext should load"),
        b"encrypted-item-v1".to_vec()
    );

    let loaded_record = load_item_record(&pool, &vault.id, &item.id)
        .await
        .expect("item row should load");
    assert_eq!(loaded_record, item);

    let listed = load_items(&pool, &vault.id)
        .await
        .expect("vault items should load");
    assert_eq!(listed, vec![(item.id, b"encrypted-item-v1".to_vec())]);

    update_item(&pool, &vault.id, &item.id, b"encrypted-item-v2")
        .await
        .expect("item should update");
    assert_eq!(
        load_item(&pool, &vault.id, &item.id)
            .await
            .expect("updated ciphertext should load"),
        b"encrypted-item-v2".to_vec()
    );

    delete_item(&pool, &vault.id, &item.id)
        .await
        .expect("item should delete");
    assert!(matches!(
        load_item(&pool, &vault.id, &item.id).await,
        Err(StoreError::ItemNotFound(_))
    ));
}

#[tokio::test]
async fn delete_vault_cascades_items() {
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
        load_item(&pool, &vault.id, &item.id).await,
        Err(StoreError::ItemNotFound(_))
    ));
}

#[tokio::test]
async fn same_item_id_in_two_vaults_does_not_collide() {
    let pool = test_pool().await.expect("test database should connect");

    let (vault_a, _) = Vault::create("VaultA", "correct horse battery staple", &test_secret_key())
        .expect("vault A should be created");
    let (vault_b, _) = Vault::create("VaultB", "correct horse battery staple", &test_secret_key())
        .expect("vault B should be created");

    store_vault(&pool, &vault_a)
        .await
        .expect("vault A should store");
    store_vault(&pool, &vault_b)
        .await
        .expect("vault B should store");

    let shared_id = ItemId::new();
    let item_a = EncryptedItemData::new(
        shared_id,
        vault_a.id,
        "SecureNote",
        b"vault-a-ciphertext".to_vec(),
        Timestamp::now(),
        Timestamp::now(),
        0,
    );
    let item_b = EncryptedItemData::new(
        shared_id,
        vault_b.id,
        "SecureNote",
        b"vault-b-ciphertext".to_vec(),
        Timestamp::now(),
        Timestamp::now(),
        0,
    );

    store_item(&pool, &item_a)
        .await
        .expect("item A should store");
    store_item(&pool, &item_b)
        .await
        .expect("item B should store");

    assert_eq!(
        load_item(&pool, &vault_a.id, &shared_id)
            .await
            .expect("load item A"),
        b"vault-a-ciphertext".to_vec()
    );
    assert_eq!(
        load_item(&pool, &vault_b.id, &shared_id)
            .await
            .expect("load item B"),
        b"vault-b-ciphertext".to_vec()
    );

    assert_eq!(
        load_item_record(&pool, &vault_a.id, &shared_id)
            .await
            .expect("load record A")
            .ciphertext,
        b"vault-a-ciphertext".to_vec()
    );
    assert_eq!(
        load_item_record(&pool, &vault_b.id, &shared_id)
            .await
            .expect("load record B")
            .ciphertext,
        b"vault-b-ciphertext".to_vec()
    );

    update_item(&pool, &vault_a.id, &shared_id, b"vault-a-updated")
        .await
        .expect("update item A");
    assert_eq!(
        load_item(&pool, &vault_a.id, &shared_id)
            .await
            .expect("load updated A"),
        b"vault-a-updated".to_vec()
    );
    assert_eq!(
        load_item(&pool, &vault_b.id, &shared_id)
            .await
            .expect("load B unchanged"),
        b"vault-b-ciphertext".to_vec()
    );

    delete_item(&pool, &vault_a.id, &shared_id)
        .await
        .expect("delete item A");
    assert!(matches!(
        load_item(&pool, &vault_a.id, &shared_id).await,
        Err(StoreError::ItemNotFound(_))
    ));
    assert_eq!(
        load_item(&pool, &vault_b.id, &shared_id)
            .await
            .expect("load B after A deleted"),
        b"vault-b-ciphertext".to_vec()
    );
}

fn encrypted_item(vault_id: VaultId, ciphertext: Vec<u8>) -> EncryptedItemData {
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

#[tokio::test]
async fn transactional_rotation_updates_all_items() {
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

    let item1 = encrypted_item(vault.id, b"item1-v1".to_vec());
    let item2 = encrypted_item(vault.id, b"item2-v1".to_vec());
    store_item(&pool, &item1).await.expect("item1 should store");
    store_item(&pool, &item2).await.expect("item2 should store");

    let new_wrapped = b"new-wrapped-key".to_vec();
    let reencrypted = vec![
        (item1.id, b"item1-v2".to_vec()),
        (item2.id, b"item2-v2".to_vec()),
    ];

    porkpie_store::rotate_vault_key_transactional(
        &pool,
        &vault.id,
        &new_wrapped,
        &reencrypted,
        Some(vault.kdf_params()),
    )
    .await
    .expect("transactional rotation should succeed");

    // Verify both items updated
    assert_eq!(
        load_item(&pool, &vault.id, &item1.id)
            .await
            .expect("load item1"),
        b"item1-v2".to_vec()
    );
    assert_eq!(
        load_item(&pool, &vault.id, &item2.id)
            .await
            .expect("load item2"),
        b"item2-v2".to_vec()
    );

    // Verify vault wrapped key updated
    let loaded_vault = load_vault(&pool, &vault.id).await.expect("load vault");
    assert_eq!(loaded_vault.master_key_wrapped, new_wrapped);
}

#[tokio::test]
async fn transactional_rotation_rolls_back_on_missing_item() {
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

    let item1 = encrypted_item(vault.id, b"item1-v1".to_vec());
    store_item(&pool, &item1).await.expect("item1 should store");

    // Try to rotate with a non-existent item ID
    let fake_id = ItemId::new();
    let new_wrapped = b"new-wrapped-key".to_vec();
    let reencrypted = vec![
        (item1.id, b"item1-v2".to_vec()),
        (fake_id, b"fake-v2".to_vec()),
    ];

    let result = porkpie_store::rotate_vault_key_transactional(
        &pool,
        &vault.id,
        &new_wrapped,
        &reencrypted,
        Some(vault.kdf_params()),
    )
    .await;

    assert!(result.is_err(), "rotation should fail for missing item");

    // Verify item1 was NOT updated (transaction rolled back)
    assert_eq!(
        load_item(&pool, &vault.id, &item1.id)
            .await
            .expect("load item1"),
        b"item1-v1".to_vec()
    );

    // Verify vault wrapped key was NOT updated
    let loaded_vault = load_vault(&pool, &vault.id).await.expect("load vault");
    assert_eq!(
        loaded_vault.master_key_wrapped,
        vault.master_key_wrapped().clone()
    );
}

#[tokio::test]
async fn backup_is_created_before_rotation_and_rotation_succeeds() {
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

    let item1 = encrypted_item(vault.id, b"item1-v1".to_vec());
    let item2 = encrypted_item(vault.id, b"item2-v1".to_vec());
    store_item(&pool, &item1).await.expect("item1 should store");
    store_item(&pool, &item2).await.expect("item2 should store");

    // Create backup (simulating what rotate_key does when skip_backup=false)
    let vault_data = porkpie_core::EncryptedVaultData {
        id: vault.id,
        name: vault.name.clone(),
        created_at: vault.created_at,
        salt: vault.salt,
        master_key_wrapped: vault.master_key_wrapped().to_vec(),
        sync_revision: vault.sync_revision(),
        kdf_params: *vault.kdf_params(),
    };
    let items_for_backup = vec![
        porkpie_core::EncryptedItemData::new(
            item1.id,
            vault.id,
            "SecureNote",
            b"item1-v1".to_vec(),
            item1.created_at,
            item1.updated_at,
            item1.sync_revision,
        ),
        porkpie_core::EncryptedItemData::new(
            item2.id,
            vault.id,
            "SecureNote",
            b"item2-v1".to_vec(),
            item2.created_at,
            item2.updated_at,
            item2.sync_revision,
        ),
    ];
    let backup = porkpie_import::export_backup_file(&vault, vault_data, items_for_backup)
        .expect("export backup");
    let backup_path = std::env::temp_dir().join(format!(
        "porkpie-rotation-backup-test-{}.json.enc",
        vault.id
    ));
    porkpie_import::write_backup_file(&backup_path, &backup).expect("write backup");

    // Verify backup file exists
    assert!(
        backup_path.exists(),
        "backup file should be written to disk"
    );

    // Now perform rotation
    let new_wrapped = b"new-wrapped-key".to_vec();
    let reencrypted = vec![
        (item1.id, b"item1-v2".to_vec()),
        (item2.id, b"item2-v2".to_vec()),
    ];
    porkpie_store::rotate_vault_key_transactional(
        &pool,
        &vault.id,
        &new_wrapped,
        &reencrypted,
        Some(vault.kdf_params()),
    )
    .await
    .expect("transactional rotation should succeed");

    // Verify items were rotated
    assert_eq!(
        load_item(&pool, &vault.id, &item1.id)
            .await
            .expect("load item1"),
        b"item1-v2".to_vec()
    );
    assert_eq!(
        load_item(&pool, &vault.id, &item2.id)
            .await
            .expect("load item2"),
        b"item2-v2".to_vec()
    );

    // Verify backup still exists and can be read
    let read_backup = porkpie_import::read_backup_file(&backup_path).expect("read backup back");
    assert_eq!(read_backup.version, 1);

    // Cleanup
    std::fs::remove_file(&backup_path).expect("remove backup");
}
