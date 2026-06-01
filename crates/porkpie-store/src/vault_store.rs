use crate::errors::{map_sqlx_error, Result, StoreError};
use crate::models::{i64_to_u64, parse_salt, parse_vault_id, u64_to_i64, EncryptedVaultData};
use porkpie_core::Vault;
use porkpie_types::{Timestamp, VaultId};
use sqlx::SqlitePool;

/// Store encrypted vault metadata.
pub async fn store_vault(pool: &SqlitePool, vault: &Vault) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO vaults (id, name, created_at, salt, master_key_wrapped, sync_revision)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            created_at = excluded.created_at,
            salt = excluded.salt,
            master_key_wrapped = excluded.master_key_wrapped,
            sync_revision = excluded.sync_revision
        "#,
    )
    .bind(vault.id.to_string())
    .bind(&vault.name)
    .bind(vault.created_at.to_millis())
    .bind(vault.salt.as_slice())
    .bind(vault.master_key_wrapped.as_slice())
    .bind(u64_to_i64(vault.sync_revision))
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(())
}

/// Load encrypted vault metadata without decrypting it.
pub async fn load_vault(pool: &SqlitePool, vault_id: &VaultId) -> Result<EncryptedVaultData> {
    let row = sqlx::query_as::<_, (String, String, i64, Vec<u8>, Vec<u8>, i64)>(
        r#"
        SELECT id, name, created_at, salt, master_key_wrapped, sync_revision
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
    })
}

/// Load encrypted vault metadata by name.
pub async fn load_vault_by_name(pool: &SqlitePool, name: &str) -> Result<EncryptedVaultData> {
    let row = sqlx::query_as::<_, (String, String, i64, Vec<u8>, Vec<u8>, i64)>(
        r#"
        SELECT id, name, created_at, salt, master_key_wrapped, sync_revision
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
