use crate::commands::{unlock_current_vault, CommandContext};
use crate::errors::{CliError, Result};
use crate::interactive::{confirm_action, prompt_master_password, prompt_secret};
use porkpie_types::LocalSecretKey;

/// Change the master password. Keeps the vault key and item ciphertext unchanged.
pub async fn change_password(context: &CommandContext) -> Result<()> {
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let vault_id = vault.id;

    let current_password = prompt_master_password()?;
    let secret_key = {
        let session = context.load_session()?;
        session.require_secret_key()?
    };

    // Verify current password works.
    let mut test_vault = porkpie_core::Vault::from_encrypted_metadata(
        vault.id,
        vault.name.clone(),
        vault.created_at,
        vault.salt,
        vault.master_key_wrapped().to_vec(),
        vault.sync_revision(),
        *vault.kdf_params(),
    );
    if test_vault.unlock(&current_password, &secret_key).is_err() {
        return Err(CliError::InvalidArgument(
            "current password is incorrect".to_string(),
        ));
    }

    println!("Enter new master password:");
    let new_password = prompt_secret("New password", true)?;
    println!("Confirm new master password:");
    let confirm = prompt_secret("Confirm", true)?;
    if new_password != confirm {
        return Err(CliError::InvalidArgument(
            "passwords do not match".to_string(),
        ));
    }

    // Re-derive master key and re-wrap vault key.
    let password = secrecy::Secret::new(current_password);
    let master_key = zeroize::Zeroizing::new(
        porkpie_crypto::derive_key(
            &password,
            secret_key.as_bytes(),
            &vault.salt,
            vault.kdf_params(),
        )
        .map_err(|e| CliError::Core(porkpie_core::CoreError::CryptoError(e)))?,
    );
    let new_password_secret = secrecy::Secret::new(new_password);
    let new_master_key = zeroize::Zeroizing::new(
        porkpie_crypto::derive_key(
            &new_password_secret,
            secret_key.as_bytes(),
            &vault.salt,
            vault.kdf_params(),
        )
        .map_err(|e| CliError::Core(porkpie_core::CoreError::CryptoError(e)))?,
    );

    let vault_key = test_vault
        .decrypt_vault_key(&master_key)
        .map_err(CliError::Core)?;
    let new_wrapped = porkpie_crypto::wrap_vault_key(&new_master_key, &vault_key)
        .map_err(|e| CliError::Core(porkpie_core::CoreError::CryptoError(e)))?;

    // Update vault metadata.
    porkpie_store::update_vault_wrapped_key(
        &pool,
        &vault_id,
        &new_wrapped,
        Some(vault.kdf_params()),
    )
    .await
    .map_err(CliError::Store)?;

    // Update session if needed.
    let mut session = context.load_session()?;
    session.lock();
    context.save_session(&session)?;

    println!("Master password changed. Unlock again with the new password.");
    Ok(())
}

/// Rotate the local secret key. Generates a new key, re-wraps vault key, updates recovery kit.
pub async fn rotate_local_secret(context: &CommandContext) -> Result<()> {
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let vault_id = vault.id;

    let password = prompt_master_password()?;
    let _current_secret_key = {
        let session = context.load_session()?;
        session.require_secret_key()?
    };

    let new_secret_key = LocalSecretKey::generate();
    let recovery_kit = porkpie_types::RecoveryKit::new(
        &vault_id.to_string(),
        &new_secret_key,
        porkpie_types::Timestamp::now().to_millis(),
    );

    // Re-wrap vault key with new local secret key using CURRENT stored KDF params.
    let password_secret = secrecy::Secret::new(password);
    let new_master_key = zeroize::Zeroizing::new(
        porkpie_crypto::derive_key(
            &password_secret,
            new_secret_key.as_bytes(),
            &vault.salt,
            vault.kdf_params(),
        )
        .map_err(|e| CliError::Core(porkpie_core::CoreError::CryptoError(e)))?,
    );
    let vault_key = vault
        .vault_key()
        .ok_or_else(|| CliError::InvalidArgument("vault is locked".to_string()))?;
    let new_wrapped = porkpie_crypto::wrap_vault_key(&new_master_key, vault_key)
        .map_err(|e| CliError::Core(porkpie_core::CoreError::CryptoError(e)))?;

    // Show recovery kit and require confirmation before proceeding.
    println!("NEW RECOVERY KIT (save this securely):");
    println!(
        "{}",
        serde_json::to_string_pretty(&recovery_kit)
            .map_err(|e| CliError::Io(std::io::Error::other(e)))?
    );
    if !crate::interactive::confirm_action(
        "Have you saved the new recovery kit? This is the only copy.",
    )? {
        println!("Cancelled. No changes were made.");
        return Ok(());
    }

    // Store new secret key in keychain.
    let store = crate::secret_store::default_secret_store();
    if let Some(ref s) = store {
        s.store_local_secret_key(&vault_id, &new_secret_key)
            .map_err(|e| {
                CliError::InvalidArgument(format!(
                    "failed to store new secret key in keychain: {e}. Aborting."
                ))
            })?;

        // Verify the keychain can load the new key and it matches.
        let loaded = s
            .load_local_secret_key(&vault_id)
            .map_err(|e| {
                CliError::InvalidArgument(format!(
                    "failed to verify new secret key in keychain: {e}. Aborting."
                ))
            })?
            .ok_or_else(|| {
                CliError::InvalidArgument(
                    "new secret key missing from keychain after store. Aborting.".to_string(),
                )
            })?;
        if loaded.to_hex() != new_secret_key.to_hex() {
            return Err(CliError::InvalidArgument(
                "keychain verification failed: stored key does not match. Aborting.".to_string(),
            ));
        }
    }

    // Update DB wrapped key only after keychain store + verify succeeds.
    porkpie_store::update_vault_wrapped_key(
        &pool,
        &vault_id,
        &new_wrapped,
        Some(vault.kdf_params()),
    )
    .await
    .map_err(CliError::Store)?;

    // Save session without any secret key material.
    let session = crate::session::SessionState::unlocked(vault_id);
    context.save_session(&session)?;

    println!("Local secret key rotated.");
    println!("WARNING: The old recovery kit is now invalid.");

    Ok(())
}

/// Rotate the vault key. Re-encrypts all items. Creates encrypted backup unless --skip-backup.
pub async fn rotate_key(context: &CommandContext, skip_backup: bool) -> Result<()> {
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let vault_id = vault.id;

    let password = prompt_master_password()?;
    let secret_key = {
        let session = context.load_session()?;
        session.require_secret_key()?
    };

    if !skip_backup {
        println!("Creating encrypted backup before key rotation...");
        let items = porkpie_store::load_items(&pool, &vault_id)
            .await
            .map_err(CliError::Store)?;
        let encrypted_items: Vec<porkpie_core::EncryptedItemData> = items
            .into_iter()
            .map(|(id, ciphertext)| {
                porkpie_core::EncryptedItemData::new(
                    id,
                    vault_id,
                    "", // item type unknown at store layer; acceptable for backup
                    ciphertext,
                    porkpie_types::Timestamp::now(),
                    porkpie_types::Timestamp::now(),
                    0,
                )
            })
            .collect();
        let vault_data = porkpie_core::EncryptedVaultData {
            id: vault_id,
            name: vault.name.clone(),
            created_at: vault.created_at,
            salt: vault.salt,
            master_key_wrapped: vault.master_key_wrapped().to_vec(),
            sync_revision: vault.sync_revision(),
            kdf_params: *vault.kdf_params(),
        };
        let backup = porkpie_import::export_backup_file(&vault, vault_data, encrypted_items)
            .map_err(|e| CliError::Io(std::io::Error::other(e)))?;
        let backup_path =
            std::path::PathBuf::from(porkpie_import::backup_file_name(backup.timestamp));
        porkpie_import::write_backup_file(&backup_path, &backup)
            .map_err(|e| CliError::Io(std::io::Error::other(e)))?;
        println!("Backup written to: {}", backup_path.display());
    }

    if !confirm_action("This will re-encrypt all items. Continue?")? {
        println!("Cancelled.");
        return Ok(());
    }

    let mut vault = unlock_current_vault(context).await?;
    let re_encrypted = vault
        .rotate_vault_key(&password, &secret_key)
        .map_err(CliError::Core)?;

    // Persist atomically in a single transaction.
    let new_wrapped = vault.master_key_wrapped().to_vec();
    porkpie_store::rotate_vault_key_transactional(
        &pool,
        &vault_id,
        &new_wrapped,
        &re_encrypted,
        Some(vault.kdf_params()),
    )
    .await
    .map_err(CliError::Store)?;

    println!("Vault key rotated. All items re-encrypted.");
    Ok(())
}

/// Benchmark Argon2id and suggest parameters near the target time.
pub async fn calibrate_kdf(_context: &CommandContext, target_ms: u64) -> Result<()> {
    println!("Calibrating Argon2id parameters to target ~{target_ms}ms...");

    let mut best_time = u128::MAX;
    let mut best_params = porkpie_crypto::Argon2Params::default();

    for time_cost in [2, 3, 4, 5] {
        for mem_cost in [8192, 16384, 32768, 65536] {
            let params = porkpie_crypto::Argon2Params {
                time_cost,
                mem_cost,
                parallelism: 1,
            };
            let start = std::time::Instant::now();
            let _ = porkpie_crypto::derive_key(
                &secrecy::Secret::new("benchmark".to_string()),
                &[0u8; 32],
                &[0u8; 32],
                &params,
            );
            let elapsed = start.elapsed().as_millis();
            if elapsed < best_time && elapsed >= u128::from(target_ms) / 2 {
                best_time = elapsed;
                best_params = params;
            }
        }
    }

    println!(
        "Suggested parameters: time_cost={}, mem_cost={}",
        best_params.time_cost, best_params.mem_cost
    );
    println!("Actual benchmark time: {best_time}ms");
    Ok(())
}

/// Upgrade the KDF profile for the current vault.
pub async fn upgrade_kdf(context: &CommandContext, profile: &str) -> Result<()> {
    let vault = unlock_current_vault(context).await?;
    let pool = context.pool().await?;
    let vault_id = vault.id;

    let password = prompt_master_password()?;
    let secret_key = {
        let session = context.load_session()?;
        session.require_secret_key()?
    };

    let new_params = match profile.to_ascii_lowercase().as_str() {
        "low-memory" => porkpie_crypto::Argon2Params {
            time_cost: 2,
            mem_cost: 4096,
            parallelism: 1,
        },
        "standard" => porkpie_crypto::Argon2Params::default(),
        "hardened" => porkpie_crypto::Argon2Params {
            time_cost: 4,
            mem_cost: 65536,
            parallelism: 1,
        },
        "paranoid" => porkpie_crypto::Argon2Params {
            time_cost: 8,
            mem_cost: 262144,
            parallelism: 2,
        },
        _ => {
            return Err(CliError::InvalidArgument(format!(
                "unknown profile: {profile}"
            )))
        }
    };

    let password_secret = secrecy::Secret::new(password);
    let new_master_key = zeroize::Zeroizing::new(
        porkpie_crypto::derive_key(
            &password_secret,
            secret_key.as_bytes(),
            &vault.salt,
            &new_params,
        )
        .map_err(|e| CliError::Core(porkpie_core::CoreError::CryptoError(e)))?,
    );

    let vault_key = vault
        .vault_key()
        .ok_or_else(|| CliError::InvalidArgument("vault is locked".to_string()))?;
    let new_wrapped = porkpie_crypto::wrap_vault_key(&new_master_key, vault_key)
        .map_err(|e| CliError::Core(porkpie_core::CoreError::CryptoError(e)))?;

    porkpie_store::update_vault_wrapped_key(&pool, &vault_id, &new_wrapped, Some(&new_params))
        .await
        .map_err(CliError::Store)?;

    println!("KDF profile upgraded to '{profile}'.");
    Ok(())
}
