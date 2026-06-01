use crate::commands::CommandContext;
use crate::errors::{map_core_error, map_store_error, CliError, Result};
use porkpie_store::{
    load_item_records, load_sync_state, load_vault, save_sync_state, EncryptedItemData,
};
use porkpie_sync::{ConflictItem, EncryptedSyncItem, MergeStrategy, SyncRequest};
use porkpie_types::{ItemId, Timestamp};
use serde::{Deserialize, Serialize};

const DEFAULT_SYNC_URL: &str = "http://127.0.0.1:8000";

/// Bidirectional sync: register vault, push local encrypted changes,
/// pull remote encrypted changes, merge them locally, and preserve
/// conflicts instead of silently overwriting.
pub async fn run(
    context: &CommandContext,
    server: Option<String>,
    api_key: Option<String>,
    strategy: MergeStrategy,
) -> Result<()> {
    let session = context.load_session()?;
    let vault_id = session.require_unlocked_vault()?;
    let secret_key = session.require_secret_key()?;
    let pool = context.pool().await?;

    let server_url = server
        .or_else(|| std::env::var("PORKPIE_SYNC_URL").ok())
        .unwrap_or_else(|| DEFAULT_SYNC_URL.to_string())
        .trim_end_matches('/')
        .to_string();
    let api_key_str = api_key
        .or_else(|| std::env::var("PORKPIE_API_KEY").ok())
        .ok_or(CliError::SyncHttp {
            status: reqwest::StatusCode::UNAUTHORIZED,
            message: "missing API key; pass --api-key or set PORKPIE_API_KEY".to_string(),
        })?;

    let client = reqwest::Client::new();

    // 1. Load vault metadata from local DB (encrypted blobs only).
    let vault_data = load_vault(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;

    // 2. Register vault on server if this is the first sync.
    let register_url = format!("{server_url}/api/v1/sync/register");
    let register_resp = client
        .post(&register_url)
        .bearer_auth(&api_key_str)
        .json(&SyncRegisterRequest {
            vault_id: vault_id.to_string(),
            name: vault_data.name.clone(),
            salt: vault_data.salt.to_vec(),
            master_key_wrapped: vault_data.master_key_wrapped.clone(),
            created_at: vault_data.created_at.to_millis(),
            kdf_time_cost: vault_data.kdf_params.time_cost,
            kdf_mem_cost: vault_data.kdf_params.mem_cost,
            kdf_parallelism: vault_data.kdf_params.parallelism,
        })
        .send()
        .await?;
    ensure_success(register_resp).await?;

    // 3. Load local sync cursor.
    let sync_state = load_sync_state(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;
    let last_synced_revision = sync_state.and_then(|s| s.last_synced_revision).unwrap_or(0);

    // 4. Load all local items.
    let local_items = load_item_records(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;
    let changed_local: Vec<&EncryptedItemData> = local_items
        .iter()
        .filter(|i| i.sync_revision > last_synced_revision)
        .collect();

    // 5. Pull encrypted items from server.
    let begin_url = format!("{server_url}/api/v1/sync/begin");
    let begin_resp = client
        .post(&begin_url)
        .bearer_auth(&api_key_str)
        .json(&SyncRequest {
            vault_id: vault_id.to_string(),
            last_revision: last_synced_revision,
        })
        .send()
        .await?;
    let server_sync: SyncResponse = response_json(ensure_success(begin_resp).await?).await?;

    // 6. Unlock vault in memory.
    let mut vault = vault_data.into_locked_vault();
    let password = crate::interactive::prompt_master_password()?;
    vault
        .unlock(&password, &secret_key)
        .map_err(map_core_error)?;

    let local_encrypted: std::collections::HashMap<String, &EncryptedItemData> =
        local_items.iter().map(|i| (i.id.to_string(), i)).collect();

    for item_data in &local_items {
        if let Ok(decrypted) =
            vault.decrypt_item(&item_data.ciphertext, &item_data.id, &item_data.item_type)
        {
            vault.items_mut().insert(decrypted.id, decrypted);
        }
    }

    // 7. Decrypt and merge each server item.
    let mut conflicts: Vec<ConflictItem> = Vec::new();
    let mut merged_count = 0usize;

    for server_item in &server_sync.items {
        let item_id = match ItemId::from_string(&server_item.item_id) {
            Ok(id) => id,
            Err(_) => {
                eprintln!(
                    "WARN: skipping item with invalid ID: {}",
                    server_item.item_id
                );
                continue;
            }
        };
        let local_changed = local_encrypted
            .get(&server_item.item_id)
            .is_some_and(|e| e.sync_revision > last_synced_revision);

        if local_changed && !server_item.ciphertext.is_empty() {
            let local_revision = local_encrypted
                .get(&server_item.item_id)
                .map(|e| e.sync_revision)
                .unwrap_or(0);
            conflicts.push(ConflictItem {
                item_id: server_item.item_id.clone(),
                local_revision,
                server_revision: server_item.sync_revision,
                server_data: server_item.ciphertext.clone(),
            });
            merged_count += 1;
            continue;
        }

        match vault.decrypt_item(&server_item.ciphertext, &item_id, &server_item.item_type) {
            Ok(decrypted) => {
                vault.items_mut().insert(decrypted.id, decrypted);
                merged_count += 1;
            }
            Err(error) => {
                eprintln!(
                    "WARN: could not decrypt server item {}: {error}",
                    server_item.item_id
                );
            }
        }
    }

    if !conflicts.is_empty() {
        eprintln!(
            "WARNING: {} item(s) changed on both sides. Conflicts preserved.",
            conflicts.len()
        );
    }

    // 8. Re-encrypt all vault items and persist with bumped revisions.
    let mut max_revision = server_sync.new_revision;
    for (id, item) in vault.items() {
        let ciphertext = vault.encrypt_item(item).map_err(map_core_error)?;
        let local = local_encrypted.get(&id.to_string());
        let existing_revision = local.map(|e| e.sync_revision).unwrap_or(0);
        let new_revision = existing_revision
            .max(server_sync.new_revision)
            .saturating_add(1);
        max_revision = max_revision.max(new_revision);

        let record = EncryptedItemData::new(
            *id,
            vault_id,
            item.data.type_label(),
            ciphertext,
            item.created_at,
            item.updated_at,
            new_revision,
        );
        porkpie_store::upsert_item_revision(&pool, &record, new_revision)
            .await
            .map_err(map_store_error)?;
    }

    // 9. Push local encrypted changes to server.
    if !changed_local.is_empty() {
        let push_items: Vec<EncryptedSyncItem> = changed_local
            .iter()
            .map(|item| EncryptedSyncItem {
                item_id: item.id.to_string(),
                item_type: item.item_type.clone(),
                ciphertext: item.ciphertext.clone(),
                created_at: item.created_at.to_millis(),
                updated_at: item.updated_at.to_millis(),
                sync_revision: item.sync_revision,
            })
            .collect();

        let push_url = format!("{server_url}/api/v1/sync/push");
        let push_resp = client
            .post(&push_url)
            .bearer_auth(&api_key_str)
            .json(&SyncPushRequest {
                vault_id: vault_id.to_string(),
                base_revision: last_synced_revision,
                items: push_items,
                merge_strategy: Some(strategy),
            })
            .send()
            .await?;

        if push_resp.status() == reqwest::StatusCode::CONFLICT {
            let error_body: ErrorResponse = response_json(push_resp).await?;
            let server_conflicts = error_body.conflicts.unwrap_or_default();
            eprintln!(
                "WARNING: server rejected push with {} conflict(s). Conflicts preserved.",
                server_conflicts.len()
            );
        } else {
            ensure_success(push_resp).await?;
        }
    }

    // 10. Save updated sync cursor.
    let updated = porkpie_store::SyncState {
        vault_id,
        last_synced_revision: Some(max_revision),
        last_synced_at: Some(Timestamp::now()),
    };
    save_sync_state(&pool, &updated)
        .await
        .map_err(map_store_error)?;

    if conflicts.is_empty() {
        println!(
            "Sync completed for vault {vault_id} ({merged_count} items merged, revision {max_revision})"
        );
    } else {
        println!(
            "Sync completed for vault {vault_id} with {} conflict(s) (revision {max_revision})",
            conflicts.len()
        );
    }

    Ok(())
}

// ---- request types ----

#[derive(Serialize)]
struct SyncRegisterRequest {
    vault_id: String,
    name: String,
    salt: Vec<u8>,
    master_key_wrapped: Vec<u8>,
    created_at: i64,
    kdf_time_cost: u32,
    kdf_mem_cost: u32,
    kdf_parallelism: u32,
}

#[derive(Deserialize)]
struct SyncResponse {
    items: Vec<EncryptedSyncItem>,
    new_revision: u64,
}

#[derive(Serialize)]
struct SyncPushRequest {
    vault_id: String,
    base_revision: u64,
    items: Vec<EncryptedSyncItem>,
    merge_strategy: Option<MergeStrategy>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    #[allow(dead_code)]
    error: String,
    #[allow(dead_code)]
    message: String,
    #[allow(dead_code)]
    conflicts: Option<Vec<ConflictItem>>,
}

// ---- helpers ----

async fn ensure_success(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let message = match response.text().await {
        Ok(msg) => msg,
        Err(_) => "response body unavailable".to_string(),
    };
    Err(CliError::SyncHttp { status, message })
}

async fn response_json<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    response.json().await.map_err(|error| CliError::SyncHttp {
        status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("response parse error: {error}"),
    })
}
