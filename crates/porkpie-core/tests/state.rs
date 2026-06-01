use porkpie_core::{Vault, VaultState};

#[test]
fn revision_tracks_item_mutations() {
    let mut vault = Vault::create("correct horse battery staple").expect("vault should be created");

    assert_eq!(vault.sync_revision, 0);
    assert_eq!(vault.state(), VaultState::Unlocked);

    vault.lock().expect("vault should lock");
    assert_eq!(vault.sync_revision, 0);
    assert_eq!(vault.state(), VaultState::Locked);
}

#[test]
fn encrypted_metadata_constructs_locked_vault() {
    let vault = Vault::create("correct horse battery staple").expect("vault should be created");
    let loaded = Vault::from_encrypted_metadata(
        vault.id,
        vault.created_at,
        vault.salt,
        vault.master_key_wrapped.clone(),
        vault.sync_revision,
    );

    assert_eq!(loaded.state(), VaultState::Locked);
    assert_eq!(loaded.id, vault.id);
}
