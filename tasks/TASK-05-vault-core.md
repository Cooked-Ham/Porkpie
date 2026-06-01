---
task_id: 05-vault-core
task_name: Vault Core Logic
sequence: 5
dependencies_complete: [01-workspace, 02-docs, 03-types, 04-crypto]
estimated_duration: 3-4 hours
difficulty: Medium
blockers_resolved: none
can_parallelize: false
---

# Task 5: Vault Core Logic

## 🎯 Objective

Implement `porkpie-core` crate with vault lifecycle management (create, unlock, lock) and in-memory item management.

## ✅ Acceptance Criteria

**Vault Lifecycle**
- [ ] `Vault::create(password: &str) -> Result<Vault>`
  - [ ] Derives key from password using porkpie-crypto
  - [ ] Generates random salt
  - [ ] Wraps vault key
  - [ ] Returns unlocked vault
- [ ] `vault.unlock(password: &str) -> Result<()>`
  - [ ] Derives key from password
  - [ ] Unwraps vault key
  - [ ] Sets `is_locked = false`
  - [ ] Returns error on wrong password
- [ ] `vault.lock() -> Result<()>`
  - [ ] Clears all items from memory
  - [ ] Sets `is_locked = true`
  - [ ] Zeroizes decrypted state

**Item Management (In-Memory)**
- [ ] `vault.create_item(item: Item) -> Result<ItemId>`
  - [ ] Only works when unlocked
  - [ ] Generates new ItemId
  - [ ] Stores in memory
- [ ] `vault.get_item(id: ItemId) -> Result<&Item>`
  - [ ] Returns reference to item
  - [ ] Only works when unlocked
- [ ] `vault.list_items() -> Result<Vec<&Item>>`
  - [ ] Returns all items
  - [ ] Only works when unlocked
- [ ] `vault.update_item(id: ItemId, item: Item) -> Result<()>`
  - [ ] Updates item in memory
  - [ ] Only works when unlocked
- [ ] `vault.delete_item(id: ItemId) -> Result<()>`
  - [ ] Removes item from memory
  - [ ] Only works when unlocked

**State Management**
- [ ] Vault tracking revision number (for sync)
- [ ] Items track creation_at, updated_at timestamps
- [ ] Vault struct is Send + Sync (thread-safe)

**Password Generation**
- [ ] `generate_password(length: usize, options: &PasswordOptions) -> String`
  - [ ] Length configurable (min 8, max 128)
  - [ ] Character sets: uppercase, lowercase, numbers, symbols
  - [ ] Cryptographically secure random
- [ ] Options for pattern (custom chars, exclude ambiguous)

**Error Handling**
- [ ] Error when operating on locked vault
- [ ] Error when item not found
- [ ] Error when password wrong
- [ ] Specific error messages (not generic)

**Tests**
- [ ] Create vault works
- [ ] Unlock with correct password works
- [ ] Unlock with wrong password fails
- [ ] Lock clears items from memory
- [ ] Create/list/update/delete items work
- [ ] Operations on locked vault fail
- [ ] Password generation has sufficient entropy
- [ ] Revision tracking works

**Code Quality**
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes (zero warnings)
- [ ] All functions documented
- [ ] No panics on bad input

## 📋 Output Specification

### File Structure

```
crates/porkpie-core/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API
│   ├── vault.rs                # Vault struct + lifecycle
│   ├── item.rs                 # Item management
│   ├── password_gen.rs         # Password generation
│   ├── errors.rs               # Core-specific errors
│   └── state.rs                # Vault state tracking
└── tests/
    ├── vault_lifecycle.rs      # Create/unlock/lock tests
    ├── item_crud.rs            # Item management tests
    ├── state.rs                # State management tests
    └── password_gen.rs         # Password generation tests
```

### Cargo.toml

```toml
[package]
name = "porkpie-core"
version = "0.1.0"
edition = "2021"

[dependencies]
porkpie-types = { path = "../porkpie-types" }
porkpie-crypto = { path = "../porkpie-crypto" }
uuid = { version = "1.0", features = ["v4"] }
rand = "0.8"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
```

### Example: Vault Struct (`src/vault.rs`)

```rust
use crate::Item;
use porkpie_types::ItemId;
use std::collections::HashMap;

pub struct Vault {
    pub id: VaultId,
    pub created_at: Timestamp,
    pub master_key_wrapped: Vec<u8>,     // from porkpie-crypto::wrap_vault_key
    pub items: HashMap<ItemId, Item>,     // in-memory, cleared on lock
    pub is_locked: bool,
    pub sync_revision: u64,
}

impl Vault {
    /// Create new vault with master password
    pub fn create(password: &str) -> Result<Self, CoreError> {
        let salt = porkpie_crypto::generate_salt()?;
        let master_key = porkpie_crypto::derive_key(password, &salt)?;
        
        // Generate vault key
        let vault_key = porkpie_crypto::generate_vault_key()?;
        
        // Wrap vault key with master key
        let master_key_wrapped = porkpie_crypto::wrap_vault_key(&master_key, &vault_key)?;
        
        Ok(Self {
            id: VaultId::new(),
            created_at: Timestamp::now(),
            master_key_wrapped,
            items: HashMap::new(),
            is_locked: false,  // Initially unlocked after creation
            sync_revision: 0,
        })
    }

    /// Unlock vault with password
    pub fn unlock(&mut self, password: &str) -> Result<(), CoreError> {
        if !self.is_locked {
            return Err(CoreError::AlreadyUnlocked);
        }

        let master_key = porkpie_crypto::derive_key(password, &self.salt)?;
        let vault_key = porkpie_crypto::unwrap_vault_key(&master_key, &self.master_key_wrapped)?;
        
        self.is_locked = false;
        self.vault_key = Some(vault_key);  // Store for decrypt operations
        
        Ok(())
    }

    /// Lock vault (clear all decrypted state)
    pub fn lock(&mut self) -> Result<(), CoreError> {
        self.items.clear();
        self.is_locked = true;
        self.vault_key = None;
        Ok(())
    }

    /// Create item in vault
    pub fn create_item(&mut self, item: Item) -> Result<ItemId, CoreError> {
        if self.is_locked {
            return Err(CoreError::VaultLocked);
        }
        
        let id = ItemId::new();
        self.items.insert(id, item);
        self.sync_revision += 1;
        
        Ok(id)
    }

    /// Get item by ID
    pub fn get_item(&self, id: ItemId) -> Result<&Item, CoreError> {
        if self.is_locked {
            return Err(CoreError::VaultLocked);
        }
        
        self.items.get(&id).ok_or(CoreError::ItemNotFound)
    }

    /// List all items
    pub fn list_items(&self) -> Result<Vec<&Item>, CoreError> {
        if self.is_locked {
            return Err(CoreError::VaultLocked);
        }
        
        Ok(self.items.values().collect())
    }

    /// Update item
    pub fn update_item(&mut self, id: ItemId, item: Item) -> Result<(), CoreError> {
        if self.is_locked {
            return Err(CoreError::VaultLocked);
        }
        
        self.items.insert(id, item);
        self.sync_revision += 1;
        
        Ok(())
    }

    /// Delete item
    pub fn delete_item(&mut self, id: ItemId) -> Result<(), CoreError> {
        if self.is_locked {
            return Err(CoreError::VaultLocked);
        }
        
        self.items.remove(&id).ok_or(CoreError::ItemNotFound)?;
        self.sync_revision += 1;
        
        Ok(())
    }
}
```

### Example: Password Generation (`src/password_gen.rs`)

```rust
use rand::seq::SliceRandom;

pub struct PasswordOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub numbers: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool,
}

pub fn generate_password(options: &PasswordOptions) -> Result<String, CoreError> {
    if options.length < 8 || options.length > 128 {
        return Err(CoreError::InvalidPasswordLength);
    }

    let mut chars = Vec::new();
    
    if options.uppercase { chars.extend_from_slice(&UPPERCASE); }
    if options.lowercase { chars.extend_from_slice(&LOWERCASE); }
    if options.numbers { chars.extend_from_slice(&NUMBERS); }
    if options.symbols { chars.extend_from_slice(&SYMBOLS); }
    
    if chars.is_empty() {
        return Err(CoreError::NoCharacterSetsSelected);
    }

    let mut rng = rand::thread_rng();
    let password: String = (0..options.length)
        .map(|_| {
            chars.choose(&mut rng)
                .map(|&c| c as char)
                .unwrap_or('?')
        })
        .collect();

    Ok(password)
}

const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
```

## 🔗 References

- **Vault Design:** Task 2 (DATA_MODEL.md)
- **Crypto API:** Task 4 (porkpie-crypto)
- **Item Types:** Task 3 (porkpie-types)

## ✔️ Success Verification

```bash
# Build
cargo build --package porkpie-core

# Tests
cargo test --package porkpie-core -- --nocapture

# Lint
cargo clippy --package porkpie-core -- -D warnings

# Format
cargo fmt --package porkpie-core --check
```

## 🚨 If Blocked...

| Problem | Solution |
|---------|----------|
| "Locking vault is complicated" | On lock: clear `items` HashMap, set `is_locked = true`, drop vault_key. |
| "How do I track state?" | Use fields: `is_locked: bool`, `vault_key: Option<[u8; 32]>`, `items: HashMap`. |
| "Password generation entropy?" | Use `rand::thread_rng()` and `.choose()`. It's cryptographically secure. |
| "Memory safety issues" | Use `zeroize::Zeroizing` for sensitive data. Auto-zeros on drop. |

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

**Task 6: Local Storage (SQLx & SQLite)**

Next agent will persist encrypted vault to SQLite. They'll use vault.serialize() to get encrypted blob.

---

**Status:** Ready for agent assignment
