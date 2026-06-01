use crate::commands::CommandContext;
use crate::errors::{map_store_error, CliError, Result};
use porkpie_store::{load_item_record, load_items, load_vault};
use porkpie_sync::{EncryptedSyncItem, SyncRequest};

const DEFAULT_SYNC_URL: &str = "http://127.0.0.1:8000";

/// Push local encrypted item rows and fetch remote changes.
pub async fn run(
    context: &CommandContext,
    server: Option<String>,
    api_key: Option<String>,
) -> Result<()> {
    let session = context.load_session()?;
    let vault_id = session.require_unlocked_vault()?;
    let pool = context.pool().await?;
    let vault = load_vault(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;
    let item_refs = load_items(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;

    let mut items = Vec::with_capacity(item_refs.len());
    for (item_id, _) in item_refs {
        let item = load_item_record(&pool, &item_id)
            .await
            .map_err(map_store_error)?;
        items.push(EncryptedSyncItem {
            item_id: item.id.to_string(),
            item_type: item.item_type,
            ciphertext: item.ciphertext,
            created_at: item.created_at.to_millis(),
            updated_at: item.updated_at.to_millis(),
            sync_revision: item.sync_revision,
        });
    }

    let server = server
        .or_else(|| std::env::var("PORKPIE_SYNC_URL").ok())
        .unwrap_or_else(|| DEFAULT_SYNC_URL.to_string());
    let api_key = api_key
        .or_else(|| std::env::var("PORKPIE_API_KEY").ok())
        .ok_or(CliError::SyncHttp {
            status: reqwest::StatusCode::UNAUTHORIZED,
            message: "missing API key; pass --api-key or set PORKPIE_API_KEY".to_string(),
        })?;

    let client = reqwest::Client::new();
    let begin_url = format!("{}/api/v1/sync/begin", server.trim_end_matches('/'));
    let begin = client
        .post(begin_url)
        .bearer_auth(&api_key)
        .json(&SyncRequest {
            vault_id: vault_id.to_string(),
            last_revision: vault.sync_revision,
        })
        .send()
        .await?;
    ensure_success(begin).await?;

    let push_url = format!("{}/api/v1/sync/push", server.trim_end_matches('/'));
    let push = client
        .post(push_url)
        .bearer_auth(api_key)
        .json(&porkpie_api_compat::SyncPushRequest {
            vault_id: vault_id.to_string(),
            base_revision: vault.sync_revision,
            items,
            merge_strategy: None,
        })
        .send()
        .await?;
    ensure_success(push).await?;

    println!("Sync completed for vault {vault_id}");
    Ok(())
}

async fn ensure_success(response: reqwest::Response) -> Result<()> {
    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let message = match response.text().await {
        Ok(message) => message,
        Err(_) => "response body unavailable".to_string(),
    };
    Err(CliError::SyncHttp { status, message })
}

mod porkpie_api_compat {
    use porkpie_sync::{EncryptedSyncItem, MergeStrategy};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct SyncPushRequest {
        pub vault_id: String,
        pub base_revision: u64,
        pub items: Vec<EncryptedSyncItem>,
        pub merge_strategy: Option<MergeStrategy>,
    }
}
