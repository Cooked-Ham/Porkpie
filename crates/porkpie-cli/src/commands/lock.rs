use crate::commands::CommandContext;
use crate::errors::Result;

/// Lock the remembered session.
pub async fn run(context: &CommandContext) -> Result<()> {
    let mut session = context.load_session()?;
    session.lock();
    context.save_session(&session)?;
    println!("Vault locked");
    Ok(())
}
