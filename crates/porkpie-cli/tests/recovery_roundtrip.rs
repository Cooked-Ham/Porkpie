use std::collections::HashSet;

use porkpie_core::{EncryptedItemData, EncryptedVaultData, Item, LocalSecretKey, Vault};
use porkpie_import::{
    encrypted_backup::{import_backup, write_backup_file},
    export_backup_file, read_backup_file, BackupImportMode, BackupImportResult,
};
use porkpie_store::{
    connect_database, load_item, load_item_record, load_vault, store_item, store_vault,
};
use porkpie_types::{ItemType, SecureNoteSecret};

fn test_secret_key() -> LocalSecretKey {
    LocalSecretKey::from_hex("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        .unwrap()
}

fn note_item(content: &str) -> Item {
    Item::new(ItemType::SecureNote(SecureNoteSecret {
        title: "note".to_string(),
        content: content.to_string(),
    }))
}

#[tokio::test]
async fn recovery_restore_roundtrip_works() {
    let secret_key = test_secret_key();
    let password = "correct horse battery staple";
    let fixture_secret = "DO_NOT_LEAK_RECOVERY_42";

    // 1. Create vault and add item with fixture secret.
    let (mut vault, recovery_kit) = Vault::create("TestVault", password, &secret_key).unwrap();
    let vault_id = vault.id;
    let item_id = vault.create_item(note_item(fixture_secret)).unwrap();

    // 2. Export encrypted backup.
    let item = vault.get_item(item_id).unwrap().clone();
    let encrypted_item = EncryptedItemData::new(
        item_id,
        vault.id,
        "SecureNote",
        vault.encrypt_item(&item).unwrap(),
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
        kdf_params: porkpie_core::Argon2Params::default(),
    };
    let backup = export_backup_file(&vault, vault_data, vec![encrypted_item]).unwrap();

    // 3. Write recovery kit and backup to temp files.
    let kit_path = std::env::temp_dir().join(format!("porkpie-recovery-kit-{}.json", vault_id));
    let backup_path = std::env::temp_dir().join(format!("porkpie-backup-{}.enc", vault_id));
    std::fs::write(&kit_path, serde_json::to_string(&recovery_kit).unwrap()).unwrap();
    write_backup_file(&backup_path, &backup).unwrap();

    // 4. Create a new clean database.
    let db_path = std::env::temp_dir().join(format!("porkpie-restore-db-{}.db", vault_id));
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = connect_database(&db_url).await.unwrap();
    porkpie_store::migrations::run_migrations(&pool)
        .await
        .unwrap();

    // 5. Restore from recovery kit and backup.
    let kit_contents = std::fs::read_to_string(&kit_path).unwrap();
    let kit: serde_json::Value = serde_json::from_str(&kit_contents).unwrap();
    let kit_secret_key_hex = kit["local_secret_key"].as_str().unwrap();
    let kit_secret_key = LocalSecretKey::from_hex(kit_secret_key_hex).unwrap();

    let backup = read_backup_file(&backup_path).unwrap();
    let BackupImportResult {
        vault,
        items,
        imported: _,
        skipped: _,
    } = import_backup(
        backup,
        password,
        &kit_secret_key,
        &HashSet::new(),
        BackupImportMode::SkipDuplicates,
        Some(&vault_id.to_string()),
    )
    .unwrap();

    // 6. Store restored vault and items.
    let locked_vault = vault.into_locked_vault();
    store_vault(&pool, &locked_vault).await.unwrap();
    for item in &items {
        store_item(&pool, item).await.unwrap();
    }

    // 7. Unlock restored vault.
    let mut loaded_vault = load_vault(&pool, &vault_id)
        .await
        .unwrap()
        .into_locked_vault();
    loaded_vault
        .unlock(password, &kit_secret_key)
        .expect("unlock restored vault");

    // 8. Load encrypted item from DB and decrypt.
    let ciphertext = load_item(&pool, &vault_id, &item_id).await.unwrap();
    let record = load_item_record(&pool, &vault_id, &item_id).await.unwrap();
    let restored_item = loaded_vault
        .decrypt_item(&ciphertext, &item_id, &record.item_type)
        .unwrap();

    // 9. Assert fixture secret matches.
    if let ItemType::SecureNote(secret) = &restored_item.data {
        assert_eq!(secret.content, fixture_secret);
    } else {
        panic!("expected SecureNote");
    }

    // 10. Assert raw DB does not contain fixture secret.
    let db_bytes = std::fs::read(&db_path).unwrap();
    assert!(
        !db_bytes
            .windows(fixture_secret.len())
            .any(|w| w == fixture_secret.as_bytes()),
        "fixture secret must not appear in raw DB bytes"
    );

    // Cleanup.
    let _ = std::fs::remove_file(&kit_path);
    let _ = std::fs::remove_file(&backup_path);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn recovery_restore_wrong_password_fails() {
    let secret_key = test_secret_key();
    let password = "correct horse battery staple";
    let wrong_password = "wrong horse battery staple";

    let (mut vault, recovery_kit) = Vault::create("TestVault", password, &secret_key).unwrap();
    let vault_id = vault.id;
    let item_id = vault.create_item(note_item("secret")).unwrap();
    let item = vault.get_item(item_id).unwrap().clone();
    let encrypted_item = EncryptedItemData::new(
        item.id,
        vault.id,
        "SecureNote",
        vault.encrypt_item(&item).unwrap(),
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
        kdf_params: porkpie_core::Argon2Params::default(),
    };
    let backup = export_backup_file(&vault, vault_data, vec![encrypted_item]).unwrap();

    let kit_path =
        std::env::temp_dir().join(format!("porkpie-recovery-kit-wrong-{}.json", vault_id));
    let backup_path = std::env::temp_dir().join(format!("porkpie-backup-wrong-{}.enc", vault_id));
    std::fs::write(&kit_path, serde_json::to_string(&recovery_kit).unwrap()).unwrap();
    write_backup_file(&backup_path, &backup).unwrap();

    let kit_contents = std::fs::read_to_string(&kit_path).unwrap();
    let kit: serde_json::Value = serde_json::from_str(&kit_contents).unwrap();
    let kit_secret_key_hex = kit["local_secret_key"].as_str().unwrap();
    let kit_secret_key = LocalSecretKey::from_hex(kit_secret_key_hex).unwrap();

    let backup = read_backup_file(&backup_path).unwrap();
    let result = import_backup(
        backup,
        wrong_password,
        &kit_secret_key,
        &HashSet::new(),
        BackupImportMode::SkipDuplicates,
        Some(&vault_id.to_string()),
    );

    assert!(result.is_err(), "wrong password must fail decryption");

    let _ = std::fs::remove_file(&kit_path);
    let _ = std::fs::remove_file(&backup_path);
}

#[tokio::test]
async fn recovery_restore_vault_id_mismatch_fails() {
    let secret_key = test_secret_key();
    let password = "correct horse battery staple";

    let (mut vault_a, _recovery_kit_a) = Vault::create("VaultA", password, &secret_key).unwrap();
    let (vault_b, recovery_kit_b) = Vault::create("VaultB", password, &secret_key).unwrap();
    let item_id_a = vault_a.create_item(note_item("secret")).unwrap();
    let item = vault_a.get_item(item_id_a).unwrap().clone();
    let encrypted_item = EncryptedItemData::new(
        item.id,
        vault_a.id,
        "SecureNote",
        vault_a.encrypt_item(&item).unwrap(),
        item.created_at,
        item.updated_at,
        vault_a.sync_revision(),
    );
    let vault_data = EncryptedVaultData {
        id: vault_a.id,
        name: vault_a.name.clone(),
        created_at: vault_a.created_at,
        salt: vault_a.salt,
        master_key_wrapped: vault_a.master_key_wrapped().clone(),
        sync_revision: vault_a.sync_revision(),
        kdf_params: porkpie_core::Argon2Params::default(),
    };
    let backup = export_backup_file(&vault_a, vault_data, vec![encrypted_item]).unwrap();

    // Use recovery kit from vault B with backup from vault A.
    let kit_path =
        std::env::temp_dir().join(format!("porkpie-recovery-kit-mismatch-{}.json", vault_b.id));
    let backup_path =
        std::env::temp_dir().join(format!("porkpie-backup-mismatch-{}.enc", vault_a.id));
    std::fs::write(&kit_path, serde_json::to_string(&recovery_kit_b).unwrap()).unwrap();
    write_backup_file(&backup_path, &backup).unwrap();

    let kit_contents = std::fs::read_to_string(&kit_path).unwrap();
    let kit: serde_json::Value = serde_json::from_str(&kit_contents).unwrap();
    let kit_vault_id = kit["vault_id"].as_str().unwrap();

    let backup = read_backup_file(&backup_path).unwrap();
    let backup_vault_id = backup.vault.id.to_string();
    assert_ne!(
        backup_vault_id, kit_vault_id,
        "test setup: vault IDs must differ"
    );

    // Import with the wrong vault_id should fail.
    let result = import_backup(
        backup,
        password,
        &secret_key,
        &HashSet::new(),
        BackupImportMode::SkipDuplicates,
        Some(kit_vault_id),
    );
    assert!(result.is_err(), "import with mismatched vault_id must fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("does not match expected vault_id"),
        "error should mention vault_id mismatch: {err}"
    );

    let _ = std::fs::remove_file(&kit_path);
    let _ = std::fs::remove_file(&backup_path);
}

#[tokio::test]
async fn recovery_restore_keychain_failure_behavior() {
    let secret_key = test_secret_key();
    let password = "correct horse battery staple";
    let fixture_secret = "DO_NOT_LEAK_KEYCHAIN_FAIL_42";

    let (mut vault, recovery_kit) = Vault::create("TestVault", password, &secret_key).unwrap();
    let vault_id = vault.id;
    let item_id = vault.create_item(note_item(fixture_secret)).unwrap();
    let item = vault.get_item(item_id).unwrap().clone();
    let encrypted_item = EncryptedItemData::new(
        item_id,
        vault.id,
        "SecureNote",
        vault.encrypt_item(&item).unwrap(),
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
        kdf_params: porkpie_core::Argon2Params::default(),
    };
    let backup = export_backup_file(&vault, vault_data, vec![encrypted_item]).unwrap();

    let kit_path = std::env::temp_dir().join(format!("porkpie-recovery-kit-kc-{}.json", vault_id));
    let backup_path = std::env::temp_dir().join(format!("porkpie-backup-kc-{}.enc", vault_id));
    std::fs::write(&kit_path, serde_json::to_string(&recovery_kit).unwrap()).unwrap();
    write_backup_file(&backup_path, &backup).unwrap();

    // Create a context with a temp database.
    let db_path = std::env::temp_dir().join(format!("porkpie-restore-kc-db-{}.db", vault_id));
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    let context = porkpie_cli::commands::CommandContext::new(
        Some(db_url.clone()),
        Some(std::env::temp_dir().join(format!("porkpie-session-kc-{}.json", vault_id))),
    );

    // Use a failing keychain that always fails on store.
    struct FailingKeychain;
    impl porkpie_cli::secret_store::SecretStore for FailingKeychain {
        fn store_local_secret_key(
            &self,
            _vault_id: &porkpie_types::VaultId,
            _key: &LocalSecretKey,
        ) -> porkpie_cli::secret_store::Result<()> {
            Err(porkpie_cli::secret_store::SecretStoreError::Unavailable(
                "test keychain failure".to_string(),
            ))
        }
        fn load_local_secret_key(
            &self,
            _vault_id: &porkpie_types::VaultId,
        ) -> porkpie_cli::secret_store::Result<Option<LocalSecretKey>> {
            Ok(None)
        }
        fn delete_local_secret_key(
            &self,
            _vault_id: &porkpie_types::VaultId,
        ) -> porkpie_cli::secret_store::Result<()> {
            Ok(())
        }
    }

    // Restore with the failing keychain should still succeed (vault restored, but keychain fails).
    let result = porkpie_cli::commands::recovery_cmd::restore_with_store_and_password(
        &context,
        &kit_path,
        &backup_path,
        Some(&FailingKeychain),
        password,
    )
    .await;
    assert!(
        result.is_ok(),
        "restore should succeed even with failing keychain"
    );

    // Verify the vault is restored correctly.
    let pool = porkpie_store::connect_database(&db_url).await.unwrap();
    let mut loaded_vault = load_vault(&pool, &vault_id)
        .await
        .unwrap()
        .into_locked_vault();
    loaded_vault
        .unlock(password, &secret_key)
        .expect("unlock restored vault");

    let ciphertext = load_item(&pool, &vault_id, &item_id).await.unwrap();
    let record = load_item_record(&pool, &vault_id, &item_id).await.unwrap();
    let restored_item = loaded_vault
        .decrypt_item(&ciphertext, &item_id, &record.item_type)
        .unwrap();

    if let ItemType::SecureNote(secret) = &restored_item.data {
        assert_eq!(secret.content, fixture_secret);
    } else {
        panic!("expected SecureNote");
    }

    let _ = std::fs::remove_file(&kit_path);
    let _ = std::fs::remove_file(&backup_path);
    let _ = std::fs::remove_file(&db_path);
}
