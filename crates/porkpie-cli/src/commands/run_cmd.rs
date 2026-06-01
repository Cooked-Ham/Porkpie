use crate::commands::{find_item_by_name, parse_pie_uri, unlock_vault_by_name, CommandContext};
use crate::errors::{map_store_error, CliError, Result};
use porkpie_store::{load_item, load_item_record};
use std::process::Command;

pub async fn run(
    context: &CommandContext,
    env_mappings: Vec<String>,
    command_args: Vec<String>,
) -> Result<()> {
    if command_args.is_empty() {
        return Err(CliError::InvalidArgument(
            "no command specified to run".to_string(),
        ));
    }

    let pool = context.pool().await?;

    let mut env_vars: Vec<(String, String)> = Vec::new();

    for mapping in &env_mappings {
        let (env_name, uri_str) = mapping.split_once('=').ok_or_else(|| {
            CliError::InvalidArgument(format!("invalid env mapping: {}", mapping))
        })?;

        let uri = parse_pie_uri(uri_str)?;
        let vault = unlock_vault_by_name(context, &uri.vault_name).await?;

        let (item_id, _item) = find_item_by_name(&vault, &uri.item_name)?;
        let record = load_item_record(&pool, &item_id)
            .await
            .map_err(map_store_error)?;

        let ciphertext = load_item(&pool, &item_id).await.map_err(map_store_error)?;
        let decrypted = vault.decrypt_item(&ciphertext, &item_id, &record.item_type)?;

        let value = decrypted
            .data
            .get_field(&uri.field_name)
            .map_err(|e| CliError::FieldError(e.to_string()))?;

        env_vars.push((env_name.to_string(), value));
    }

    let program = &command_args[0];
    let args = &command_args[1..];

    let mut cmd = Command::new(program);
    cmd.args(args);

    for (name, value) in &env_vars {
        cmd.env(name, value);
    }

    let status = cmd.status().map_err(|e| {
        CliError::Io(std::io::Error::other(format!(
            "failed to execute {}: {}",
            program, e
        )))
    })?;

    std::process::exit(status.code().unwrap_or(1));
}
