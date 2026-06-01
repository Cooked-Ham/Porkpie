use porkpie_core::{CoreError, Item, LocalSecretKey, Vault, VaultState};
use porkpie_types::{ItemType, SecureNoteSecret};

fn note_item(content: &str) -> Item {
    Item::new(ItemType::SecureNote(SecureNoteSecret {
        title: "note".to_string(),
        content: content.to_string(),
    }))
}

fn test_secret_key() -> LocalSecretKey {
    LocalSecretKey::from_hex("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        .unwrap()
}

#[test]
fn create_vault_returns_unlocked_vault() {
    let (vault, _recovery) = Vault::create(
        "TestVault",
        "correct horse battery staple",
        &test_secret_key(),
    )
    .expect("vault should be created");

    assert_eq!(vault.state(), VaultState::Unlocked);
    assert!(!vault.master_key_wrapped.is_empty());
    assert_eq!(vault.sync_revision, 0);
}

#[test]
fn unlock_with_correct_password_and_key_works() {
    let secret_key = test_secret_key();
    let (mut vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");
    vault.lock().expect("vault should lock");

    vault
        .unlock("correct horse battery staple", &secret_key)
        .expect("correct password + key should unlock");

    assert_eq!(vault.state(), VaultState::Unlocked);
}

#[test]
fn unlock_with_wrong_password_fails() {
    let secret_key = test_secret_key();
    let (mut vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");
    vault.lock().expect("vault should lock");

    let error = vault
        .unlock("not the password", &secret_key)
        .expect_err("wrong password must fail");

    assert!(matches!(error, CoreError::WrongPassword));
    assert_eq!(vault.state(), VaultState::Locked);
}

#[test]
fn unlock_with_wrong_secret_key_fails() {
    let secret_key = test_secret_key();
    let (mut vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");
    vault.lock().expect("vault should lock");

    let wrong_key = LocalSecretKey::from_hex(
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();
    let error = vault
        .unlock("correct horse battery staple", &wrong_key)
        .expect_err("wrong secret key must fail");

    assert!(matches!(error, CoreError::WrongPassword));
    assert_eq!(vault.state(), VaultState::Locked);
}

#[test]
fn password_alone_cannot_unlock() {
    let secret_key = test_secret_key();
    let (mut vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");
    vault.lock().expect("vault should lock");

    let zero_key = LocalSecretKey::from_hex(
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap();
    let error = vault
        .unlock("correct horse battery staple", &zero_key)
        .expect_err("password without correct secret key must fail");

    assert!(matches!(error, CoreError::WrongPassword));
}

#[test]
fn lock_clears_items_from_memory() {
    let secret_key = test_secret_key();
    let (mut vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");
    let id = vault
        .create_item(note_item("secret"))
        .expect("item should be created");
    assert!(vault.get_item(id).is_ok());

    vault.lock().expect("vault should lock");

    assert_eq!(vault.state(), VaultState::Locked);
    assert!(vault.items.is_empty());
    assert!(matches!(vault.get_item(id), Err(CoreError::VaultLocked)));
}

#[test]
fn operations_on_locked_vault_fail() {
    let secret_key = test_secret_key();
    let (mut vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");
    vault.lock().expect("vault should lock");

    assert!(matches!(
        vault.create_item(note_item("secret")),
        Err(CoreError::VaultLocked)
    ));
    assert!(matches!(vault.list_items(), Err(CoreError::VaultLocked)));
}
