---
task_id: 06-storage
task_name: Local Storage (SQLx & SQLite)
sequence: 6
dependencies_complete: [01-workspace, 03-types, 04-crypto, 05-vault-core]
estimated_duration: 3-4 hours
difficulty: Medium
blockers_resolved: none
can_parallelize: false
---

# Task 6: Local Storage (SQLx & SQLite)

## 🎯 Objective

Implement `porkpie-store` crate for encrypted vault persistence in SQLite. **Key principle:** Store layer never decrypts. All data encrypted before arriving.

## ✅ Acceptance Criteria

**Database Schema**
- [ ] SQLite schema created with tables:
  - [ ] `vaults` (id, created_at, master_key_wrapped, sync_revision)
  - [ ] `items` (id, vault_id, item_type, ciphertext, created_at, updated_at, sync_revision)
  - [ ] `sync_state` (vault_id, last_synced_revision, last_synced_at)
- [ ] Foreign key constraints enabled
- [ ] Indexes on vault_id, item_type for queries

**Vault Storage**
- [ ] `store_vault(db: &Pool, vault: &Vault) -> Result<()>`
  - [ ] Inserts vault metadata (never decrypts)
  - [ ] Stores master_key_wrapped as-is (encrypted blob)
- [ ] `load_vault(db: &Pool, vault_id: &VaultId) -> Result<EncryptedVaultData>`
  - [ ] Returns encrypted vault (not decrypted)
  - [ ] Returns master_key_wrapped blob
- [ ] `delete_vault(db: &Pool, vault_id: &VaultId) -> Result<()>`

**Item Storage**
- [ ] `store_item(db: &Pool, vault_id: &VaultId, item: &Item) -> Result<()>`
  - [ ] Stores encrypted ciphertext (from porkpie-crypto)
  - [ ] Never sees plaintext
- [ ] `load_item(db: &Pool, item_id: &ItemId) -> Result<Vec<u8>>`
  - [ ] Returns encrypted ciphertext (not decrypted)
- [ ] `load_items(db: &Pool, vault_id: &VaultId) -> Result<Vec<(ItemId, Vec<u8>)>>`
  - [ ] Returns all encrypted items for vault
- [ ] `update_item(db: &Pool, item_id: &ItemId, ciphertext: &[u8]) -> Result<()>`
- [ ] `delete_item(db: &Pool, item_id: &ItemId) -> Result<()>`

**Connection Pooling**
- [ ] Connection pool (sqlx::sqlite::SqlitePool)
- [ ] Pool configured for read + write
- [ ] Concurrent reads supported

**Migrations**
- [ ] Schema migrations (sqlx migrations)
- [ ] Run on database initialization
- [ ] Idempotent (safe to re-run)

**Error Handling**
- [ ] Vault not found → specific error
- [ ] Item not found → specific error
- [ ] Database constraint violations → specific error
- [ ] No panics on bad queries

**Tests**
- [ ] Store and load vault works
- [ ] Store and load item works
- [ ] Update item works
- [ ] Delete item works
- [ ] List items for vault works
- [ ] Foreign key constraints work
- [ ] Connection pooling works
- [ ] Concurrent access works

**Code Quality**
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes (zero warnings)
- [ ] All functions documented
- [ ] No SQL injection (use parameterized queries)

## 📋 Output Specification

### File Structure

```
crates/porkpie-store/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API
│   ├── db.rs                   # Connection pooling
│   ├── vault_store.rs          # Vault CRUD
│   ├── item_store.rs           # Item CRUD
│   ├── migrations.rs           # Schema setup
│   ├── errors.rs               # Store-specific errors
│   └── models.rs               # Internal types
└── tests/
    ├── store.rs                # CRUD tests
    ├── constraints.rs          # FK constraint tests
    └── concurrent.rs           # Concurrency tests
```

### Cargo.toml

```toml
[package]
name = "porkpie-store"
version = "0.1.0"
edition = "2021"

[dependencies]
porkpie-types = { path = "../porkpie-types" }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio", "uuid"] }
tokio = { version = "1", features = ["full"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
```

### SQLite Schema (`src/migrations.rs`)

```sql
-- Create tables
CREATE TABLE IF NOT EXISTS vaults (
    id TEXT PRIMARY KEY NOT NULL,
    created_at INTEGER NOT NULL,
    master_key_wrapped BLOB NOT NULL,
    sync_revision INTEGER NOT NULL DEFAULT 0,
    created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY NOT NULL,
    vault_id TEXT NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    ciphertext BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    sync_revision INTEGER NOT NULL DEFAULT 0,
    created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sync_state (
    vault_id TEXT PRIMARY KEY NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    last_synced_revision INTEGER,
    last_synced_at INTEGER
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_items_vault_id ON items(vault_id);
CREATE INDEX IF NOT EXISTS idx_items_type ON items(item_type);
```

### Example: Vault Storage (`src/vault_store.rs`)

```rust
use sqlx::SqlitePool;
use porkpie_types::VaultId;

pub async fn store_vault(
    pool: &SqlitePool,
    id: &VaultId,
    created_at: i64,
    master_key_wrapped: &[u8],
) -> Result<(), StoreError> {
    sqlx::query(
        r#"
        INSERT INTO vaults (id, created_at, master_key_wrapped, sync_revision)
        VALUES (?, ?, ?, 0)
        "#
    )
    .bind(id.to_string())
    .bind(created_at)
    .bind(master_key_wrapped)
    .execute(pool)
    .await
    .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub async fn load_vault(
    pool: &SqlitePool,
    id: &VaultId,
) -> Result<(Vec<u8>, i64), StoreError> {
    let row = sqlx::query_as::<_, (Vec<u8>, i64)>(
        "SELECT master_key_wrapped, sync_revision FROM vaults WHERE id = ?"
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| StoreError::DatabaseError(e.to_string()))?
    .ok_or(StoreError::VaultNotFound)?;

    Ok(row)
}
```

### Example: Item Storage (`src/item_store.rs`)

```rust
pub async fn store_item(
    pool: &SqlitePool,
    item_id: &ItemId,
    vault_id: &VaultId,
    item_type: &str,
    ciphertext: &[u8],
    created_at: i64,
) -> Result<(), StoreError> {
    sqlx::query(
        r#"
        INSERT INTO items (id, vault_id, item_type, ciphertext, created_at, updated_at, sync_revision)
        VALUES (?, ?, ?, ?, ?, ?, 0)
        "#
    )
    .bind(item_id.to_string())
    .bind(vault_id.to_string())
    .bind(item_type)
    .bind(ciphertext)
    .bind(created_at)
    .bind(created_at)
    .execute(pool)
    .await
    .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub async fn load_items(
    pool: &SqlitePool,
    vault_id: &VaultId,
) -> Result<Vec<(String, String, Vec<u8>)>, StoreError> {
    let rows = sqlx::query_as::<_, (String, String, Vec<u8>)>(
        "SELECT id, item_type, ciphertext FROM items WHERE vault_id = ?"
    )
    .bind(vault_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| StoreError::DatabaseError(e.to_string()))?;

    Ok(rows)
}
```

## 🔗 References

- **Data Model:** Task 2 (DATA_MODEL.md)
- **Vault Structure:** Task 5 (porkpie-core)

## ✔️ Success Verification

```bash
# Build
cargo build --package porkpie-store

# Tests (requires sqlite)
cargo test --package porkpie-store -- --nocapture

# Lint
cargo clippy --package porkpie-store -- -D warnings

# Format
cargo fmt --package porkpie-store --check
```

## 🚨 If Blocked...

| Problem | Solution |
|---------|----------|
| "SQLx is complex" | Use query builders. `sqlx::query(SQL_STRING).bind(value).execute(pool).await` |
| "How do migrations work?" | Put SQL in `.sql` files in `sqlx-data/` folder. sqlx reads at compile-time. |
| "Connection pooling confusing" | Use `SqlitePool::connect(url).await`. Pool handles lifecycle. |
| "Foreign key constraints fail" | Ensure `PRAGMA foreign_keys = ON;` is set. Delete parent → deletes children. |

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

**Task 7: Command-Line Interface**

Next agent will implement porkpie-cli. They'll use these storage APIs to persist/load vaults.

---

**Status:** Ready for agent assignment
