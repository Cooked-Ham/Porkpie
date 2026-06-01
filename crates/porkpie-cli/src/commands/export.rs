use crate::commands::{unlock_current_vault, CommandContext};
use crate::errors::{map_store_error, CliError, Result};
use porkpie_import::{backup_file_name, encrypted_backup::write_backup_file, export_backup_file};
use porkpie_store::{load_item_record, load_items, load_vault};
use std::path::PathBuf;

/// Export the current vault as an encrypted backup.
pub async fn run_encrypted(context: &CommandContext, output: Option<PathBuf>) -> Result<()> {
    let unlocked_vault = unlock_current_vault(context).await?;
    let vault_id = unlocked_vault.id;
    let pool = context.pool().await?;
    let vault = load_vault(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;
    let item_refs = load_items(&pool, &vault_id)
        .await
        .map_err(map_store_error)?;
    let mut items = Vec::with_capacity(item_refs.len());

    for (item_id, _) in item_refs {
        items.push(
            load_item_record(&pool, &vault_id, &item_id)
                .await
                .map_err(map_store_error)?,
        );
    }

    let backup = export_backup_file(&unlocked_vault, vault, items)?;
    let output_path = output.unwrap_or_else(default_backup_path);
    write_backup_file(&output_path, &backup)?;
    println!("Backup saved to {}", output_path.display());
    Ok(())
}

/// Export vault data, either encrypted or plaintext.
pub async fn run(
    context: &CommandContext,
    format: &str,
    dangerous: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    match format.to_ascii_lowercase().as_str() {
        "plaintext" => run_plaintext(context, dangerous, output).await,
        _ => run_encrypted(context, output).await,
    }
}

/// Export all items in plaintext JSON. Requires the `--dangerous` flag.
async fn run_plaintext(
    context: &CommandContext,
    dangerous: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    if !dangerous {
        return Err(CliError::InvalidArgument(
            "plaintext export requires the --dangerous flag. This will expose ALL secrets in plaintext.".to_string(),
        ));
    }

    let confirmed = crate::interactive::confirm_dangerous_plaintext_export()?;
    if !confirmed {
        println!("Plaintext export cancelled.");
        return Ok(());
    }

    let vault = unlock_current_vault(context).await?;
    let items = vault.list_items().map_err(crate::errors::map_core_error)?;

    let mut plaintext_items = Vec::new();
    for item in items {
        // Use serde_json::Value to avoid leaking in Debug output
        let json_value = serde_json::to_value(&item.data).map_err(CliError::Json)?;
        plaintext_items.push(json_value);
    }

    let output_path = output.unwrap_or_else(|| PathBuf::from("porkpie-export-plaintext.json"));
    let file = std::fs::File::create(&output_path)?;
    serde_json::to_writer_pretty(file, &plaintext_items).map_err(CliError::Json)?;

    println!(
        "DANGER: Plaintext export written to {}. Delete this file immediately after use.",
        output_path.display()
    );
    Ok(())
}

fn default_backup_path() -> PathBuf {
    PathBuf::from(backup_file_name(
        porkpie_types::Timestamp::now().to_millis(),
    ))
}
