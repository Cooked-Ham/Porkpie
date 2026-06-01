---
task_id: 03-types
task_name: Core Types and Error Definitions
sequence: 3
dependencies_complete: [01-workspace, 02-docs]
estimated_duration: 3-4 hours
difficulty: Medium
blockers_resolved: none
can_parallelize: false
---

# Task 3: Core Types and Error Definitions

## 🎯 Objective

Implement `porkpie-types` crate with all shared domain types. This is the single source of truth for types used across all crates.

## ✅ Acceptance Criteria

**Core Types**
- [ ] VaultId type (UUID-based)
- [ ] ItemId type (UUID-based)
- [ ] RevisionId type (u64)
- [ ] UserId type (UUID-based)
- [ ] Timestamp type (i64 unix millis)

**Item Types**
- [ ] ItemType enum with all 10 variants:
  - [ ] Login
  - [ ] APIKey
  - [ ] SSHKey
  - [ ] SecureNote
  - [ ] Server
  - [ ] Database
  - [ ] Identity
  - [ ] SoftwareLicense
  - [ ] RecoveryCodes
  - [ ] Custom
- [ ] Vault struct with metadata

**Secret Field Types**
- [ ] LoginSecret (username, password, url, notes)
- [ ] APIKeySecret (name, key, provider)
- [ ] SSHKeySecret (name, public_key, private_key, passphrase)
- [ ] SecureNoteSecret (title, content)
- [ ] ServerSecret (hostname, port, username, password, notes)
- [ ] DatabaseSecret (engine, host, port, username, password, database)
- [ ] IdentitySecret (name, email, phone, address)
- [ ] SoftwareLicenseSecret (product, key, version, expiry)
- [ ] RecoveryCodesSecret (codes list)
- [ ] CustomSecret (key-value map)

**Serialization**
- [ ] All types implement Serialize
- [ ] All types implement Deserialize
- [ ] All types implement Clone
- [ ] All types implement Debug

**Error Types**
- [ ] VaultError enum with variants
- [ ] CryptoError enum with variants
- [ ] StorageError enum with variants
- [ ] SyncError enum with variants
- [ ] Error codes assigned (e.g., 1001, 1002, etc.)

**Tests**
- [ ] Serialization round-trip tests
- [ ] Type validation tests
- [ ] Error code tests
- [ ] All tests pass

**Code Quality**
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes (zero warnings)
- [ ] No `todo!()` or `panic!()` in production code
- [ ] Comprehensive documentation comments

## 📋 Output Specification

### File Structure

```
crates/porkpie-types/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Module declarations
│   ├── ids.rs              # VaultId, ItemId, etc.
│   ├── timestamp.rs        # Timestamp type
│   ├── item_type.rs        # ItemType enum + all Secret types
│   ├── vault.rs            # Vault struct
│   ├── errors.rs           # Error types + codes
│   └── constants.rs        # Max lengths, constraints
└── tests/
    ├── serialization.rs    # Serde tests
    ├── types.rs            # Type validation
    └── errors.rs           # Error handling
```

### Cargo.toml

```toml
[package]
name = "porkpie-types"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"

[dev-dependencies]
```

### Example: IDs Module (`src/ids.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VaultId(Uuid);

impl VaultId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Uuid::parse_str(s).map(Self)
    }
}

impl fmt::Display for VaultId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// [Similar for ItemId, UserId, RevisionId]
```

### Example: Item Types Module (`src/item_type.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Login(LoginSecret),
    APIKey(APIKeySecret),
    SSHKey(SSHKeySecret),
    SecureNote(SecureNoteSecret),
    Server(ServerSecret),
    Database(DatabaseSecret),
    Identity(IdentitySecret),
    SoftwareLicense(SoftwareLicenseSecret),
    RecoveryCodes(RecoveryCodesSecret),
    Custom(CustomSecret),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginSecret {
    pub username: String,
    pub password: String,    // Will be encrypted by porkpie-crypto
    pub url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIKeySecret {
    pub name: String,
    pub key: String,         // Will be encrypted
    pub provider: String,
}

// [Similar for other 8 types]
```

### Example: Error Types (`src/errors.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault not found: {0}")]
    NotFound(String),
    
    #[error("Vault already locked")]
    AlreadyLocked,
    
    #[error("Vault not unlocked")]
    NotUnlocked,
    
    #[error("Invalid item ID")]
    InvalidItemId,
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,
    
    #[error("Decryption failed: ciphertext tampered")]
    DecryptionFailed,
    
    #[error("Wrong password")]
    WrongPassword,
    
    #[error("Invalid nonce")]
    InvalidNonce,
}

// [Similar for StorageError, SyncError]
```

## 🔗 References

- **Item Schemas:** Porkpie Architecture and Coding Plan — Section 5 (Crate Responsibilities)
- **Data Model:** See Task 2 output (DATA_MODEL.md)
- **Error Handling:** Porkpie Implementation Plan — Section "Task Completion Report Format"

## ✔️ Success Verification

```bash
# Format check
cargo fmt --all --check

# Lint (zero warnings)
cargo clippy --workspace -- -D warnings

# Tests pass
cargo test --workspace

# Build succeeds
cargo build --workspace

# Verify serialization works
cargo test --package porkpie-types -- --nocapture
```

**Expected:** All pass, zero warnings.

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

## 🚨 If Blocked...

| Problem | Solution |
|---------|----------|
| "Not sure what fields each item type needs" | Think like a password manager user. What info would you want to store for each? (e.g., Login = username, password, URL) |
| "UUID complexity" | Use the `uuid` crate with `v4` feature. Just call `Uuid::new_v4()` for random IDs. |
| "Error handling is complex" | Use `thiserror` crate. Define error enum, `#[error(...)]` macro for messages. Done. |
| "Serialization failing" | Ensure all types derive `Serialize` + `Deserialize`. Check `serde_json::to_string()` works. |

## 📌 What Comes Next

**Task 4: Cryptographic Operations**

Next agent will implement porkpie-crypto. They'll use these types as input/output for encryption operations.

---

**Status:** Ready for agent assignment
