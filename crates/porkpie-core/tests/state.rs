use porkpie_core::{LocalSecretKey, Vault, VaultState};

fn test_secret_key() -> LocalSecretKey {
    LocalSecretKey::from_hex("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        .unwrap()
}

#[test]
fn revision_tracks_item_mutations() {
    let (mut vault, _) = Vault::create(
        "TestVault",
        "correct horse battery staple",
        &test_secret_key(),
    )
    .expect("vault should be created");

    assert_eq!(vault.sync_revision(), 0);
    assert_eq!(vault.state(), VaultState::Unlocked);

    vault.lock().expect("vault should lock");
    assert_eq!(vault.sync_revision(), 0);
    assert_eq!(vault.state(), VaultState::Locked);
}

#[test]
fn encrypted_metadata_constructs_locked_vault() {
    let (vault, _) = Vault::create(
        "TestVault",
        "correct horse battery staple",
        &test_secret_key(),
    )
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

    assert_eq!(loaded.state(), VaultState::Locked);
    assert_eq!(loaded.id, vault.id);
}
