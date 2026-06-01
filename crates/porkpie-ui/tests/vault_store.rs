use porkpie_core::LocalSecretKey;
use porkpie_types::{ItemType, LoginSecret, Timestamp};
use porkpie_ui::vault_store::VaultBackend;

#[tokio::test]
async fn vault_create_unlock_list_round_trip() {
    let backend = match VaultBackend::connect_sqlite("sqlite::memory:").await {
        Ok(backend) => backend,
        Err(_) => {
            // WASM build or environment without sqlite: nothing to do.
            return;
        }
    };

    let secret_key = LocalSecretKey::generate();
    let (summary, _recovery) = match backend
        .create_vault("Personal", "correct horse battery staple", &secret_key)
        .await
    {
        Ok(result) => result,
        Err(error) => panic!("create_vault failed: {error:?}"),
    };
    assert_eq!(summary.name, "Personal");

    // Duplicate name must fail.
    let duplicate = backend
        .create_vault("Personal", "another password 1234567890", &secret_key)
        .await;
    assert!(duplicate.is_err(), "duplicate vault name must be rejected");

    // List summaries.
    let summaries = backend
        .list_vault_summaries()
        .await
        .expect("list summaries");
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].name, "Personal");

    // Unlock with correct password + secret key.
    let handle = backend
        .unlock_vault("Personal", "correct horse battery staple", &secret_key)
        .await
        .expect("unlock with correct credentials");
    assert_eq!(handle.summary.name, "Personal");

    // Create an item.
    let login = ItemType::Login(LoginSecret {
        username: "alice".to_string(),
        password: "s3cret-password".to_string(),
        url: Some("https://example.com".to_string()),
        notes: None,
    });
    let created = handle
        .create_item(login.clone())
        .await
        .expect("create item");
    assert!(matches!(created.data, ItemType::Login(_)));

    // List items.
    let items = handle.list_items().await.expect("list items");
    assert_eq!(items.len(), 1);

    // Get the item back.
    let fetched = handle.get_item(created.id).await.expect("get item");
    match fetched.data {
        ItemType::Login(secret) => {
            assert_eq!(secret.username, "alice");
            assert_eq!(secret.password, "s3cret-password");
        }
        other => panic!("unexpected item type: {other:?}"),
    }

    // Update the item.
    let updated = ItemType::Login(LoginSecret {
        username: "alice".to_string(),
        password: "new-password".to_string(),
        url: None,
        notes: None,
    });
    let updated_item = handle
        .update_item(created.id, updated)
        .await
        .expect("update item");
    assert!(updated_item.updated_at.0 >= created.updated_at.0);

    // Delete the item.
    handle.delete_item(created.id).await.expect("delete item");
    let after_delete = handle.list_items().await.expect("list items after delete");
    assert!(after_delete.is_empty());

    // Lock the vault.
    handle.lock().await.expect("lock");
}

#[tokio::test]
async fn wrong_password_does_not_unlock() {
    let backend = match VaultBackend::connect_sqlite("sqlite::memory:").await {
        Ok(backend) => backend,
        Err(_) => return,
    };
    let secret_key = LocalSecretKey::generate();
    backend
        .create_vault("Locked", "real-password-1234567890", &secret_key)
        .await
        .expect("create_vault");

    let result = backend
        .unlock_vault("Locked", "wrong-password", &secret_key)
        .await;
    assert!(result.is_err(), "wrong password must fail to unlock");
}

#[tokio::test]
async fn unbacked_returns_unavailable() {
    let backend = VaultBackend::Unavailable;
    let result = backend.list_vault_summaries().await;
    assert!(result.is_err());
    // Timestamp not used here; keep the import alive for future tests.
    let _ = Timestamp::now();
}
