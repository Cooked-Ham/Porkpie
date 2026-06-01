---
task_id: 07-cli
task_name: Command-Line Interface
sequence: 7
dependencies_complete: [01-workspace, 03-types, 04-crypto, 05-vault-core, 06-storage]
estimated_duration: 3-4 hours
difficulty: Medium
blockers_resolved: none
can_parallelize: false
---

# Task 7: Command-Line Interface

## 🎯 Objective

Implement `porkpie-cli` crate with all CLI commands for vault management. Users interact with Porkpie through this CLI.

## ✅ Acceptance Criteria

**Commands Implemented**
- [ ] `porkpie init` — Create new vault (interactive)
- [ ] `porkpie unlock` — Unlock vault with password
- [ ] `porkpie lock` — Lock vault
- [ ] `porkpie list` — List vault items
- [ ] `porkpie get <id>` — Show decrypted secret
- [ ] `porkpie add <type>` — Create new item (interactive)
- [ ] `porkpie edit <id>` — Edit existing item
- [ ] `porkpie delete <id>` — Delete item
- [ ] `porkpie export` — Export encrypted backup
- [ ] `porkpie import <file>` — Import encrypted backup
- [ ] `porkpie sync` — Sync with server (basic stub)
- [ ] `porkpie help [command]` — Show help

**Command Details**

**init** (Interactive)
- Prompt: "Create new vault"
- Prompt for master password (min 16 chars, confirm)
- Create vault via porkpie-core
- Save to SQLite via porkpie-store
- Output: "Vault created: {vault_id}"

**unlock** (Interactive)
- Prompt: "Enter vault ID"
- Prompt: "Enter master password"
- Load vault from SQLite
- Call vault.unlock(password)
- Store session state (vault unlocked)
- Output: "Vault unlocked"

**lock**
- Clear session state
- Output: "Vault locked"

**list**
- Require: Vault unlocked
- Show: Item ID, Type, Title, Created date
- Format: Nice table or list

**get <id>**
- Require: Vault unlocked
- Load item from vault
- Display: Full decrypted content
- Format: Pretty-printed JSON or YAML

**add <type>** (Interactive)
- Require: Vault unlocked
- Prompt for fields based on item_type
- Create item via vault.create_item()
- Persist to SQLite
- Output: "Item created: {item_id}"

**edit <id>** (Interactive)
- Require: Vault unlocked
- Load item
- Prompt to modify each field
- Update via vault.update_item()
- Persist to SQLite

**delete <id>**
- Require: Vault unlocked
- Confirm: "Delete item {id}? (y/N)"
- Delete via vault.delete_item()
- Output: "Item deleted"

**export**
- Creates encrypted backup file (JSON encrypted with vault key)
- Output: "Backup saved to porkpie-backup-{timestamp}.json"

**import <file>**
- Load encrypted backup
- Prompt for backup password
- Merge into current vault
- Output: "Imported {count} items"

**Error Handling**
- [ ] Vault not found → clear error message
- [ ] Item not found → clear error message
- [ ] Wrong password → "Invalid password"
- [ ] Validation errors → show which field failed

**Session Management**
- [ ] Track current vault ID (in memory or config file)
- [ ] Track unlock state
- [ ] Timeout on inactivity (30 min default)

**Help & Usability**
- [ ] All commands have help text
- [ ] Help shown on --help, -h
- [ ] Errors suggest next command
- [ ] Progress messages informative

**Tests**
- [ ] Each command tested (mock storage)
- [ ] Error cases tested
- [ ] Help text renders
- [ ] Invalid args caught

**Code Quality**
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes (zero warnings)
- [ ] All functions documented

## 📋 Output Specification

### File Structure

```
crates/porkpie-cli/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── init.rs             # vault init
│   │   ├── unlock.rs           # vault unlock
│   │   ├── lock.rs             # vault lock
│   │   ├── list.rs             # list items
│   │   ├── get.rs              # get secret
│   │   ├── add.rs              # create item
│   │   ├── edit.rs             # edit item
│   │   ├── delete.rs           # delete item
│   │   ├── export.rs           # export backup
│   │   ├── import.rs           # import backup
│   │   └── sync.rs             # sync
│   ├── session.rs              # Session state tracking
│   ├── interactive.rs          # Prompt utilities
│   └── errors.rs               # CLI errors
└── tests/
    └── cli.rs                  # Integration tests
```

### Cargo.toml

```toml
[package]
name = "porkpie-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "porkpie"
path = "src/main.rs"

[dependencies]
porkpie-types = { path = "../porkpie-types" }
porkpie-core = { path = "../porkpie-core" }
porkpie-store = { path = "../porkpie-store" }
clap = { version = "4.4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
dialoguer = "0.11"  # Interactive prompts
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
```

### Example: Main Entry Point (`src/main.rs`)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "porkpie")]
#[command(version = "0.1.0")]
#[command(about = "Local-first password manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize new vault
    Init,
    
    /// Unlock vault
    Unlock,
    
    /// Lock vault
    Lock,
    
    /// List items
    List,
    
    /// Get secret value
    Get { id: String },
    
    /// Add new item
    Add { item_type: String },
    
    /// Edit item
    Edit { id: String },
    
    /// Delete item
    Delete { id: String },
    
    /// Export encrypted backup
    Export,
    
    /// Import encrypted backup
    Import { file: String },
    
    /// Sync with server
    Sync,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init().await?,
        Commands::Unlock => commands::unlock().await?,
        Commands::Lock => commands::lock().await?,
        Commands::List => commands::list().await?,
        Commands::Get { id } => commands::get(&id).await?,
        Commands::Add { item_type } => commands::add(&item_type).await?,
        Commands::Edit { id } => commands::edit(&id).await?,
        Commands::Delete { id } => commands::delete(&id).await?,
        Commands::Export => commands::export().await?,
        Commands::Import { file } => commands::import(&file).await?,
        Commands::Sync => commands::sync().await?,
    }

    Ok(())
}
```

### Example: Init Command (`src/commands/init.rs`)

```rust
use dialoguer::Password;

pub async fn init() -> Result<()> {
    println!("📦 Creating new vault...");

    let password = Password::new()
        .with_prompt("Master password")
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()?;

    if password.len() < 16 {
        return Err("Password must be at least 16 characters".into());
    }

    let vault = porkpie_core::Vault::create(&password)?;
    let vault_id = vault.id.to_string();

    // Persist to SQLite
    let db = porkpie_store::connect().await?;
    porkpie_store::store_vault(&db, &vault).await?;

    println!("✅ Vault created!");
    println!("   ID: {}", vault_id);
    println!("   Next: porkpie list");

    Ok(())
}
```

## 🔗 References

- **Vault Core:** Task 5 (porkpie-core)
- **Storage:** Task 6 (porkpie-store)
- **Command Design:** Porkpie Implementation Plan

## ✔️ Success Verification

```bash
# Build
cargo build --package porkpie-cli

# Help works
cargo run --package porkpie-cli -- --help

# Tests
cargo test --package porkpie-cli

# Lint
cargo clippy --package porkpie-cli -- -D warnings
```

## 🚨 If Blocked...

| Problem | Solution |
|---------|----------|
| "clap is confusing" | Use derive macro. `#[derive(Parser)] struct Cli { #[command(subcommand)] command: Commands }` |
| "How do I get user input?" | Use `dialoguer` crate. `Password::new().interact()?` gets password. |
| "Session state complex" | Store in memory during program run. Or load from `.porkpie` config file on startup. |
| "Interactive prompts hard" | Use `dialoguer` for prompts. It handles input validation + confirmation. |

## 🔒 STRICT TYPECHECK REQUIREMENTS

**Type safety is non-negotiable.** Rust's type system is your first line of defense.

- ✓ **All type errors must compile** — `cargo build` must succeed with zero type errors
- ✓ **No `unsafe` blocks without justification** — Document why in code comment
- ✓ **No unchecked casts** — Use `as` only where necessary (document reasoning)
- ✓ **No `unwrap()` on external input** — Use `.map_err()` or `?` operator
- ✓ **No `todo!()` or `unimplemented!()` in production code** — Only in stubs
- ✓ **Compiler warnings are failures** — `cargo clippy` must have zero warnings
- ✓ **Type inference must be clear** — Add explicit types where ambiguous
- ✓ **Trait bounds must be explicit** — Don't hide requirements

**Verification command:**
```bash
cargo check --workspace
cargo build --workspace
```

**If ANY type error appears, stop and fix it. Type errors = broken code.**

## 📌 What Comes Next

**Task 8: User Interface (Dioxus)**

Next agent will build GUI. CLI and UI will share same underlying vault/crypto/store libraries.

---

**Status:** Ready for agent assignment
