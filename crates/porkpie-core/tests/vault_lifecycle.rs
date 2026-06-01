use porkpie_core::{CoreError, Item, Vault, VaultState};
use porkpie_types::{ItemType, SecureNoteSecret};

fn note_item(content: &str) -> Item {
    Item::new(ItemType::SecureNote(SecureNoteSecret {
        title: "note".to_string(),
        content: content.to_string(),
    }))
}

#[test]
fn create_vault_returns_unlocked_vault() {
    let vault = Vault::create("correct horse battery staple").expect("vault should be created");

    assert_eq!(vault.state(), VaultState::Unlocked);
    assert!(!vault.master_key_wrapped.is_empty());
    assert_eq!(vault.sync_revision, 0);
}

#[test]
fn unlock_with_correct_password_works() {
    let mut vault = Vault::create("correct horse battery staple").expect("vault should be created");
    vault.lock().expect("vault should lock");

    vault
        .unlock("correct horse battery staple")
        .expect("correct password should unlock");

    assert_eq!(vault.state(), VaultState::Unlocked);
}

#[test]
fn unlock_with_wrong_password_fails() {
    let mut vault = Vault::create("correct horse battery staple").expect("vault should be created");
    vault.lock().expect("vault should lock");

    let error = vault
        .unlock("not the password")
        .expect_err("wrong password must fail");

    assert!(matches!(error, CoreError::WrongPassword));
    assert_eq!(vault.state(), VaultState::Locked);
}

#[test]
fn lock_clears_items_from_memory() {
    let mut vault = Vault::create("correct horse battery staple").expect("vault should be created");
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
    let mut vault = Vault::create("correct horse battery staple").expect("vault should be created");
    vault.lock().expect("vault should lock");

    assert!(matches!(
        vault.create_item(note_item("secret")),
        Err(CoreError::VaultLocked)
    ));
    assert!(matches!(vault.list_items(), Err(CoreError::VaultLocked)));
}
