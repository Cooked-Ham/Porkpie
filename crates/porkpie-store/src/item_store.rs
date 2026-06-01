use crate::errors::{map_sqlx_error, Result, StoreError};
use crate::models::{i64_to_u64, parse_item_id, parse_vault_id, u64_to_i64, EncryptedItemData};
use porkpie_types::{ItemId, Timestamp, VaultId};
use sqlx::SqlitePool;

/// Store an encrypted item row. The ciphertext is stored as-is and is never decrypted here.
pub async fn store_item(pool: &SqlitePool, item: &EncryptedItemData) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO items (
            id, vault_id, item_type, ciphertext, created_at, updated_at, sync_revision
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(item.id.to_string())
    .bind(item.vault_id.to_string())
    .bind(item.item_type.as_str())
    .bind(item.ciphertext.as_slice())
    .bind(item.created_at.to_millis())
    .bind(item.updated_at.to_millis())
    .bind(u64_to_i64(item.sync_revision))
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(())
}

/// Load encrypted item ciphertext by item identifier.
pub async fn load_item(pool: &SqlitePool, item_id: &ItemId) -> Result<Vec<u8>> {
    let row = sqlx::query_as::<_, (Vec<u8>,)>("SELECT ciphertext FROM items WHERE id = ?")
        .bind(item_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(StoreError::ItemNotFound(*item_id))?;

    Ok(row.0)
}

/// Load a full encrypted item row by item identifier.
pub async fn load_item_record(pool: &SqlitePool, item_id: &ItemId) -> Result<EncryptedItemData> {
    let row = sqlx::query_as::<_, (String, String, String, Vec<u8>, i64, i64, i64)>(
        r#"
        SELECT id, vault_id, item_type, ciphertext, created_at, updated_at, sync_revision
        FROM items
        WHERE id = ?
        "#,
    )
    .bind(item_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or(StoreError::ItemNotFound(*item_id))?;

    encrypted_item_from_row(row)
}

/// Load all encrypted item ciphertexts for a vault.
pub async fn load_items(pool: &SqlitePool, vault_id: &VaultId) -> Result<Vec<(ItemId, Vec<u8>)>> {
    let rows = sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT id, ciphertext FROM items WHERE vault_id = ? ORDER BY created_at, id",
    )
    .bind(vault_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    rows.into_iter()
        .map(|(id, ciphertext)| Ok((parse_item_id(id)?, ciphertext)))
        .collect()
}

/// Update an encrypted item ciphertext and revision metadata.
pub async fn update_item(pool: &SqlitePool, item_id: &ItemId, ciphertext: &[u8]) -> Result<()> {
    let now = Timestamp::now();
    let result = sqlx::query(
        r#"
        UPDATE items
        SET ciphertext = ?, updated_at = ?, sync_revision = sync_revision + 1
        WHERE id = ?
        "#,
    )
    .bind(ciphertext)
    .bind(now.to_millis())
    .bind(item_id.to_string())
    .execute(pool)
    .await
    .map_err(map_sqlx_error)?;

    if result.rows_affected() == 0 {
        return Err(StoreError::ItemNotFound(*item_id));
    }

    Ok(())
}

/// Delete an encrypted item by identifier.
pub async fn delete_item(pool: &SqlitePool, item_id: &ItemId) -> Result<()> {
    let result = sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(item_id.to_string())
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?;

    if result.rows_affected() == 0 {
        return Err(StoreError::ItemNotFound(*item_id));
    }

    Ok(())
}

fn encrypted_item_from_row(
    row: (String, String, String, Vec<u8>, i64, i64, i64),
) -> Result<EncryptedItemData> {
    Ok(EncryptedItemData {
        id: parse_item_id(row.0)?,
        vault_id: parse_vault_id(row.1)?,
        item_type: row.2,
        ciphertext: row.3,
        created_at: Timestamp(row.4),
        updated_at: Timestamp(row.5),
        sync_revision: i64_to_u64(row.6),
    })
}
