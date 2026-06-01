use porkpie_core::{Item, LocalSecretKey, Vault};
use porkpie_import::{
    encrypted_backup::{import_backup, write_backup_file},
    export_backup_file, read_backup_file, BackupImportMode,
};
use porkpie_store::{EncryptedItemData, EncryptedVaultData};
use porkpie_types::{ItemType, LoginSecret};
use std::collections::HashSet;

fn test_secret_key() -> LocalSecretKey {
    LocalSecretKey::from_hex("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        .unwrap()
}

#[test]
fn backup_roundtrip_keeps_secret_data_encrypted() {
    let password = "sixteen-character-password";
    let secret_key = test_secret_key();
    let (vault, vault_data, item) = encrypted_login(password, &secret_key, "secret-password");
    let backup = export_backup_file(&vault, vault_data, vec![item.clone()]).expect("export backup");
    let serialized = serde_json::to_vec(&backup).expect("serialize backup");

    let imported = import_backup(
        backup,
        password,
        &secret_key,
        &HashSet::new(),
        BackupImportMode::SkipDuplicates,
    )
    .expect("import backup");

    assert_eq!(imported.imported, 1);
    assert_eq!(imported.items[0], item);
    assert!(!imported.items[0]
        .ciphertext
        .windows(b"secret-password".len())
        .any(|window| window == b"secret-password"));
    assert!(!serialized
        .windows(b"secret-password".len())
        .any(|window| window == b"secret-password"));
}

#[test]
fn backup_import_rejects_wrong_password() {
    let secret_key = test_secret_key();
    let (vault, vault_data, item) =
        encrypted_login("sixteen-character-password", &secret_key, "secret-password");
    let backup = export_backup_file(&vault, vault_data, vec![item]).expect("export backup");

    let error = import_backup(
        backup,
        "wrong-password",
        &secret_key,
        &HashSet::new(),
        BackupImportMode::SkipDuplicates,
    )
    .expect_err("wrong password should fail");

    assert!(error.to_string().contains("Wrong password"));
}

#[test]
fn backup_import_skips_duplicates() {
    let password = "sixteen-character-password";
    let secret_key = test_secret_key();
    let (vault, vault_data, item) = encrypted_login(password, &secret_key, "secret-password");
    let backup = export_backup_file(&vault, vault_data, vec![item.clone()]).expect("export backup");
    let existing = HashSet::from([item.id.to_string()]);

    let imported = import_backup(
        backup,
        password,
        &secret_key,
        &existing,
        BackupImportMode::SkipDuplicates,
    )
    .expect("import backup");

    assert_eq!(imported.imported, 0);
    assert_eq!(imported.skipped, 1);
}

#[test]
fn backup_file_serializes_with_enc_extension_payload() {
    let password = "sixteen-character-password";
    let secret_key = test_secret_key();
    let (vault, vault_data, item) = encrypted_login(password, &secret_key, "secret-password");
    let backup = export_backup_file(&vault, vault_data, vec![item]).expect("export backup");
    let mut path = std::env::temp_dir();
    path.push(format!(
        "porkpie-backup-test-{}.json.enc",
        porkpie_types::VaultId::new()
    ));

    write_backup_file(&path, &backup).expect("write backup");
    let read_back = read_backup_file(&path).expect("read backup");
    std::fs::remove_file(path).expect("remove backup");

    assert_eq!(read_back.version, 1);
}

fn encrypted_login(
    password: &str,
    secret_key: &LocalSecretKey,
    secret: &str,
) -> (Vault, EncryptedVaultData, EncryptedItemData) {
    let (mut vault, _) = Vault::create("TestVault", password, secret_key).expect("create vault");
    let item_id = vault
        .create_item(Item::new(ItemType::Login(LoginSecret {
            username: "me@example.com".to_string(),
            password: secret.to_string(),
            url: None,
            notes: None,
        })))
        .expect("create item");
    let item = vault.get_item(item_id).expect("get item").clone();
    let encrypted = EncryptedItemData::new(
        item_id,
        vault.id,
        "Login",
        vault.encrypt_item(&item).expect("encrypt item"),
        item.created_at,
        item.updated_at,
        vault.sync_revision(),
    );
    let vault_data = EncryptedVaultData {
        id: vault.id,
        name: vault.name.clone(),
        created_at: vault.created_at,
        salt: vault.salt,
        master_key_wrapped: vault.master_key_wrapped().clone(),
        sync_revision: vault.sync_revision(),
    };

    (vault, vault_data, encrypted)
}
