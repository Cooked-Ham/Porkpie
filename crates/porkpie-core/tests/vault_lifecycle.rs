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
    assert!(!vault.master_key_wrapped().is_empty());
    assert_eq!(vault.sync_revision(), 0);
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
    assert!(vault.items().is_empty());
    assert!(matches!(vault.get_item(id), Err(CoreError::VaultLocked)));
}

#[test]
fn lock_zeroizes_vault_key() {
    let secret_key = test_secret_key();
    let (mut vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");

    // Vault is unlocked after creation, so vault_key should be Some
    assert!(!vault.is_locked());

    vault.lock().expect("vault should lock");

    // After lock, vault is locked and vault_key is None
    assert!(vault.is_locked());
    // The Zeroizing<[u8; 32]> was dropped during lock(), which overwrites
    // the 32 bytes with zeros before deallocation.
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

#[test]
fn rotate_vault_key_changes_wrapped_key_and_re_encrypts_items() {
    let secret_key = test_secret_key();
    let password = "correct horse battery staple";
    let (mut vault, _recovery) =
        Vault::create("TestVault", password, &secret_key).expect("vault should be created");
    let id = vault
        .create_item(note_item("secret"))
        .expect("item should be created");

    let old_wrapped = vault.master_key_wrapped().clone();
    let old_ciphertext = vault
        .encrypt_item(vault.get_item(id).expect("item exists"))
        .expect("encrypt");

    let re_encrypted = vault
        .rotate_vault_key(password, &secret_key)
        .expect("rotate should succeed");

    // Wrapped key changed
    assert_ne!(vault.master_key_wrapped(), &old_wrapped);
    // Re-encrypted list contains the item
    assert_eq!(re_encrypted.len(), 1);
    assert_eq!(re_encrypted[0].0, id);
    // New ciphertext is different from old ciphertext
    assert_ne!(re_encrypted[0].1, old_ciphertext);
    // Vault is still unlocked
    assert_eq!(vault.state(), VaultState::Unlocked);
    // Vault key changed, so old ciphertext cannot decrypt
    let result = vault.decrypt_item(&old_ciphertext, &id, "SecureNote");
    assert!(result.is_err());
    // But new ciphertext can decrypt
    let decrypted = vault
        .decrypt_item(&re_encrypted[0].1, &id, "SecureNote")
        .expect("new ciphertext should decrypt");
    if let ItemType::SecureNote(secret) = &decrypted.data {
        assert_eq!(secret.content, "secret");
    } else {
        panic!("expected SecureNote");
    }
}

#[test]
fn kdf_params_persisted_after_creation() {
    let secret_key = test_secret_key();
    let (vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");

    // Default params should be stored
    let params = vault.kdf_params();
    assert_eq!(params.time_cost, 2);
    assert_eq!(params.mem_cost, 19456);
    assert_eq!(params.parallelism, 1);
}

#[test]
fn kdf_params_used_during_unlock() {
    let secret_key = test_secret_key();
    let (mut vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");
    vault.lock().expect("vault should lock");

    // Unlock should succeed using stored params
    vault
        .unlock("correct horse battery staple", &secret_key)
        .expect("unlock with stored params should work");
    assert_eq!(vault.state(), VaultState::Unlocked);
}

#[test]
fn kdf_params_persisted_in_encrypted_metadata() {
    let secret_key = test_secret_key();
    let (vault, _recovery) =
        Vault::create("TestVault", "correct horse battery staple", &secret_key)
            .expect("vault should be created");

    let loaded = Vault::from_encrypted_metadata(
        vault.id,
        vault.name.clone(),
        vault.created_at,
        vault.salt,
        vault.master_key_wrapped().clone(),
        vault.sync_revision(),
        *vault.kdf_params(),
    );

    assert_eq!(loaded.kdf_params(), vault.kdf_params());
}

#[test]
fn vault_unlocks_after_kdf_params_change() {
    let secret_key = test_secret_key();
    let password = "correct horse battery staple";
    let (vault, _recovery) =
        Vault::create("TestVault", password, &secret_key).expect("vault should be created");

    // Change KDF params to hardened and re-wrap vault key
    let hardened = porkpie_crypto::Argon2Params {
        time_cost: 4,
        mem_cost: 65536,
        parallelism: 1,
    };

    // Derive new master key with hardened params
    let password_secret = secrecy::Secret::new(password.to_owned());
    let new_master_key = zeroize::Zeroizing::new(
        porkpie_crypto::derive_key(
            &password_secret,
            secret_key.as_bytes(),
            &vault.salt,
            &hardened,
        )
        .expect("derive with hardened params"),
    );

    // Get current vault key and re-wrap with new master key
    let vault_key = vault.vault_key().expect("vault should be unlocked");
    let new_wrapped = porkpie_crypto::wrap_vault_key(&new_master_key, vault_key)
        .expect("wrap with new master key");

    // Create a new vault from metadata with hardened params and new wrapped key
    let mut loaded = Vault::from_encrypted_metadata(
        vault.id,
        vault.name.clone(),
        vault.created_at,
        vault.salt,
        new_wrapped,
        vault.sync_revision(),
        hardened,
    );

    // Unlock should now work with hardened params
    loaded
        .unlock(password, &secret_key)
        .expect("unlock after kdf change should work");
    assert_eq!(loaded.state(), VaultState::Unlocked);
    assert_eq!(loaded.kdf_params().time_cost, 4);
}
