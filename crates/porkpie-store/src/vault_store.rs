use crate::errors::{map_sqlx_error, Result, StoreError};
use crate::models::{i64_to_u64, parse_salt, parse_vault_id, u64_to_i64, EncryptedVaultData};
use porkpie_core::Vault;
use porkpie_types::{ItemId, Timestamp, VaultId};
use sqlx::SqlitePool;

/// Store encrypted vault metadata.
pub async fn store_vault(pool: &SqlitePool, vault: &Vault) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO vaults (id, name, created_at, salt, master_key_wrapped, sync_revision, kdf_time_cost, kdf_mem_cost, kdf_parallelism)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            created_at = excluded.created_at,
            salt = excluded.salt,
            master_key_wrapped = excluded.master_key_wrapped,
            sync_revision = excluded.sync_revision,
            kdf_time_cost = excluded.kdf_time_cost,
            kdf_mem_cost = excluded.kdf_mem_cost,
            kdf_parallelism = excluded.kdf_parallelism
        "#,
    )
    .bind(vault.id.to_string())
    .bind(&vault.name)
    .bind(vault.created_at.to_millis())
    .bind(vault.salt.as_slice())
    .bind(vault.master_key_wrapped().as_slice())
    .bind(u64_to_i64(vault.sync_revision()))
    .bind(vault.kdf_params().time_cost as i64)
    .bind(vault.kdf_params().mem_cost as i64)
    .bind(vault.kdf_params().parallelism as i64)
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(())
}

/// Load encrypted vault metadata without decrypting it.
pub async fn load_vault(pool: &SqlitePool, vault_id: &VaultId) -> Result<EncryptedVaultData> {
    let row = sqlx::query_as::<_, (String, String, i64, Vec<u8>, Vec<u8>, i64, i64, i64, i64)>(
        r#"
        SELECT id, name, created_at, salt, master_key_wrapped, sync_revision, kdf_time_cost, kdf_mem_cost, kdf_parallelism
        FROM vaults
        WHERE id = ?
        "#,
    )
    .bind(vault_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or(StoreError::VaultNotFound(*vault_id))?;

    Ok(EncryptedVaultData {
        id: parse_vault_id(row.0)?,
        name: row.1,
        created_at: Timestamp(row.2),
        salt: parse_salt(row.3)?,
        master_key_wrapped: row.4,
        sync_revision: i64_to_u64(row.5),
        kdf_params: porkpie_crypto::Argon2Params {
            time_cost: u32::try_from(row.6).unwrap_or(2),
            mem_cost: u32::try_from(row.7).unwrap_or(19456),
            parallelism: u32::try_from(row.8).unwrap_or(1),
        },
    })
}

/// Load encrypted vault metadata by name.
pub async fn load_vault_by_name(pool: &SqlitePool, name: &str) -> Result<EncryptedVaultData> {
    let row = sqlx::query_as::<_, (String, String, i64, Vec<u8>, Vec<u8>, i64, i64, i64, i64)>(
        r#"
        SELECT id, name, created_at, salt, master_key_wrapped, sync_revision, kdf_time_cost, kdf_mem_cost, kdf_parallelism
        FROM vaults
        WHERE name = ?
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or_else(|| StoreError::VaultNotFoundByName(name.to_string()))?;

    Ok(EncryptedVaultData {
        id: parse_vault_id(row.0)?,
        name: row.1,
        created_at: Timestamp(row.2),
        salt: parse_salt(row.3)?,
        master_key_wrapped: row.4,
        sync_revision: i64_to_u64(row.5),
        kdf_params: porkpie_crypto::Argon2Params {
            time_cost: u32::try_from(row.6).unwrap_or(2),
            mem_cost: u32::try_from(row.7).unwrap_or(19456),
            parallelism: u32::try_from(row.8).unwrap_or(1),
        },
    })
}

/// Delete a vault and all of its encrypted items.
pub async fn delete_vault(pool: &SqlitePool, vault_id: &VaultId) -> Result<()> {
    let result = sqlx::query("DELETE FROM vaults WHERE id = ?")
        .bind(vault_id.to_string())
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?;

    if result.rows_affected() == 0 {
        return Err(StoreError::VaultNotFound(*vault_id));
    }

    Ok(())
}

/// Atomically rotate the vault key and re-encrypt all items.
///
/// This function updates all item ciphertexts and the vault wrapped key in a
/// single SQLite transaction. If any step fails, the entire operation is rolled
/// back, preventing partial corruption.
pub async fn rotate_vault_key_transactional(
    pool: &SqlitePool,
    vault_id: &VaultId,
    new_wrapped_key: &[u8],
    reencrypted_items: &[(ItemId, Vec<u8>)],
    kdf_params: Option<&porkpie_crypto::Argon2Params>,
) -> Result<()> {
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;

    let now = Timestamp::now().to_millis();
    for (item_id, ciphertext) in reencrypted_items {
        let result = sqlx::query(
            "UPDATE items SET ciphertext = ?, updated_at = ?, sync_revision = sync_revision + 1 WHERE vault_id = ? AND id = ?",
        )
        .bind(ciphertext.as_slice())
        .bind(now)
        .bind(vault_id.to_string())
        .bind(item_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(StoreError::ItemNotFound(*item_id));
        }
    }

    let vault_result = if let Some(params) = kdf_params {
        sqlx::query(
            "UPDATE vaults SET master_key_wrapped = ?, sync_revision = sync_revision + 1, kdf_time_cost = ?, kdf_mem_cost = ?, kdf_parallelism = ? WHERE id = ?",
        )
        .bind(new_wrapped_key)
        .bind(params.time_cost as i64)
        .bind(params.mem_cost as i64)
        .bind(params.parallelism as i64)
        .bind(vault_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
    } else {
        sqlx::query(
            "UPDATE vaults SET master_key_wrapped = ?, sync_revision = sync_revision + 1 WHERE id = ?",
        )
        .bind(new_wrapped_key)
        .bind(vault_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
    };

    if vault_result.rows_affected() == 0 {
        return Err(StoreError::VaultNotFound(*vault_id));
    }

    tx.commit().await.map_err(map_sqlx_error)?;
    Ok(())
}

/// Update the wrapped master key for an existing vault.
/// Used by password change, local secret rotation, KDF upgrade, and vault key rotation.
pub async fn update_vault_wrapped_key(
    pool: &SqlitePool,
    vault_id: &VaultId,
    master_key_wrapped: &[u8],
    kdf_params: Option<&porkpie_crypto::Argon2Params>,
) -> Result<()> {
    let result = if let Some(params) = kdf_params {
        sqlx::query(
            "UPDATE vaults SET master_key_wrapped = ?, sync_revision = sync_revision + 1, kdf_time_cost = ?, kdf_mem_cost = ?, kdf_parallelism = ? WHERE id = ?",
        )
        .bind(master_key_wrapped)
        .bind(params.time_cost as i64)
        .bind(params.mem_cost as i64)
        .bind(params.parallelism as i64)
        .bind(vault_id.to_string())
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?
    } else {
        sqlx::query(
            "UPDATE vaults SET master_key_wrapped = ?, sync_revision = sync_revision + 1 WHERE id = ?",
        )
        .bind(master_key_wrapped)
        .bind(vault_id.to_string())
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?
    };

    if result.rows_affected() == 0 {
        return Err(StoreError::VaultNotFound(*vault_id));
    }

    Ok(())
}
