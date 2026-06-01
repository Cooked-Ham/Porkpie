//! SSH command implementations for the Porkpie CLI.

use crate::commands::{find_item_by_name, unlock_current_vault, CommandContext};
use crate::errors::{CliError, Result};

/// Display the public key for an SSH key item.
///
/// `target` may be a `pie://` URI pointing to the `public_key` field, or an
/// item name.  Only the public key is printed; the private key is never
/// revealed by this command.
pub async fn run_public_key(context: &CommandContext, target: &str) -> Result<()> {
    let pie_uri = if target.starts_with("pie://") {
        Some(crate::commands::parse_pie_uri(target)?)
    } else {
        None
    };

    let vault = unlock_current_vault(context).await?;

    let public_key = if let Some(uri) = pie_uri {
        // pie://vault/item/public_key
        let vault_name = vault.name.clone();
        if uri.vault_name != vault_name {
            return Err(CliError::InvalidArgument(format!(
                "URI vault '{}' does not match unlocked vault '{}'",
                uri.vault_name, vault_name
            )));
        }
        let item = find_item_by_name(&vault, &uri.item_name)?;
        item.1.data.get_field(&uri.field_name).map_err(|e| {
            CliError::FieldError(format!(
                "field '{}' on item '{}': {}",
                uri.field_name, uri.item_name, e
            ))
        })?
    } else {
        // Treat target as an item name and extract the public_key field.
        let item = find_item_by_name(&vault, target)?;
        item.1
            .data
            .get_field("public_key")
            .map_err(|e| CliError::FieldError(format!("item '{target}': {e}")))?
    };

    println!("{public_key}");
    Ok(())
}

/// Print the honest status of the SSH agent integration.
pub async fn run_agent() -> Result<()> {
    println!("OpenSSH agent socket/named-pipe integration is not implemented yet.");
    Ok(())
}
