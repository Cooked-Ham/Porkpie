use porkpie_core::{CoreError, Item, LocalSecretKey, Vault};
use porkpie_types::{ItemId, ItemType, LoginSecret, SecureNoteSecret};

fn test_secret_key() -> LocalSecretKey {
    LocalSecretKey::from_hex("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        .unwrap()
}

fn login_item(password: &str) -> Item {
    Item::new(ItemType::Login(LoginSecret {
        username: "ada".to_string(),
        password: password.to_string(),
        url: Some("https://example.test".to_string()),
        notes: None,
    }))
}

fn note_item(content: &str) -> Item {
    Item::new(ItemType::SecureNote(SecureNoteSecret {
        title: "note".to_string(),
        content: content.to_string(),
    }))
}

#[test]
fn create_list_update_delete_items_work() {
    let (mut vault, _) = Vault::create(
        "TestVault",
        "correct horse battery staple",
        &test_secret_key(),
    )
    .expect("vault should be created");

    let id = vault
        .create_item(login_item("first-secret"))
        .expect("item should be created");
    assert_eq!(vault.sync_revision, 1);

    let created = vault.get_item(id).expect("item should exist");
    assert_eq!(created.id, id);

    let listed = vault.list_items().expect("vault should list items");
    assert_eq!(listed.len(), 1);

    let created_at = created.created_at;
    vault
        .update_item(id, note_item("replacement"))
        .expect("item should update");
    let updated = vault.get_item(id).expect("item should still exist");
    assert_eq!(updated.id, id);
    assert_eq!(updated.created_at, created_at);
    assert!(updated.updated_at >= created_at);
    assert_eq!(vault.sync_revision, 2);

    vault.delete_item(id).expect("item should delete");
    assert!(matches!(vault.get_item(id), Err(CoreError::ItemNotFound)));
    assert_eq!(vault.sync_revision, 3);
}

#[test]
fn missing_items_return_specific_error() {
    let (vault, _) = Vault::create(
        "TestVault",
        "correct horse battery staple",
        &test_secret_key(),
    )
    .expect("vault should be created");

    assert!(matches!(
        vault.get_item(ItemId::new()),
        Err(CoreError::ItemNotFound)
    ));
}

#[test]
fn update_missing_item_does_not_bump_revision() {
    let (mut vault, _) = Vault::create(
        "TestVault",
        "correct horse battery staple",
        &test_secret_key(),
    )
    .expect("vault should be created");

    assert!(matches!(
        vault.update_item(ItemId::new(), note_item("replacement")),
        Err(CoreError::ItemNotFound)
    ));
    assert_eq!(vault.sync_revision, 0);
}
