---
task_id: 10-final
task_name: Import/Export and Final Validation
sequence: 10
dependencies_complete: [01-workspace, 02-docs, 03-types, 04-crypto, 05-vault-core, 06-storage, 07-cli, 08-ui, 09-sync]
estimated_duration: 3-4 hours
difficulty: Medium
blockers_resolved: none
can_parallelize: false
---

# Task 10: Import/Export and Final Validation

## 🎯 Objective

Complete import/export functionality and perform comprehensive validation. Ensure MVP is production-ready.

## ✅ Acceptance Criteria

**Import/Export Implementation**
- [ ] `porkpie-import` crate created
- [ ] CSV import functionality
  - [ ] Parse CSV (columns: item_type, title, username, password, notes)
  - [ ] Validate each row
  - [ ] Create items via vault API
  - [ ] Encrypt before storing
- [ ] Encrypted backup import
  - [ ] Load JSON backup file (encrypted with vault key)
  - [ ] Decrypt with password
  - [ ] Merge items into vault
  - [ ] Handle duplicates (skip or overwrite)
- [ ] Encrypted backup export
  - [ ] Serialize all vault items to JSON
  - [ ] Encrypt with master key
  - [ ] Save to file with timestamp
  - [ ] Format: `porkpie-backup-{timestamp}.json.enc`

**CLI Export/Import**
- [ ] `porkpie export` creates backup file
- [ ] `porkpie import <file>` loads backup
- [ ] Error handling for missing files, bad format

**UI Export/Import**
- [ ] Export button in settings
- [ ] Import button opens file picker
- [ ] Success notification after import
- [ ] Progress indicator for large imports

**Comprehensive Testing**
- [ ] `cargo test --workspace` passes (all crates)
- [ ] All unit tests pass
- [ ] Integration tests pass (end-to-end CLI)
- [ ] Security invariants verified by tests

**Code Quality**
- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes (zero warnings)
- [ ] All clippy warnings resolved
- [ ] All documentation complete
- [ ] No `todo!()`, `unwrap()`, `panic!()` in production code

**Documentation Completeness**
- [ ] README.md complete and accurate
- [ ] All docs/ files written (SECURITY_INVARIANTS, THREAT_MODEL, ARCHITECTURE, etc.)
- [ ] Code comments explain non-obvious decisions
- [ ] API docs for all public functions

**Security Verification**
- [ ] All 13 security invariants verified
- [ ] Plaintext secrets never stored (verified by tests)
- [ ] Master password never stored (verified by tests)
- [ ] Encryption/decryption working (verified by tests)
- [ ] Wrong password produces error (verified by tests)
- [ ] Tampered ciphertext detected (verified by tests)
- [ ] Memory zeroization working (code review)
- [ ] No hardcoded secrets in code
- [ ] No credentials in git

**Build & Deployment**
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo build --workspace --release` succeeds
- [ ] Binary runs: `./target/debug/porkpie --version`
- [ ] Docker image builds: `docker build -f Dockerfile .`
- [ ] Docker Compose starts: `docker-compose up`

**Final Checklist**
- [ ] All tasks 1-10 completed
- [ ] All acceptance criteria met
- [ ] All tests passing
- [ ] No warnings from clippy
- [ ] No formatting issues
- [ ] Documentation complete
- [ ] Security verified
- [ ] MVP is production-ready

## 📋 Output Specification

### File Structure

```
crates/porkpie-import/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── csv.rs                  # CSV import
│   ├── encrypted_backup.rs     # Backup format
│   ├── errors.rs
│   └── validators.rs           # Field validation
└── tests/
    ├── csv.rs                  # CSV tests
    └── backup.rs               # Backup tests

Final Repository Structure:
porkpie/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── Dockerfile
├── docker-compose.yml
├── crates/
│   ├── porkpie-types/
│   ├── porkpie-crypto/
│   ├── porkpie-core/
│   ├── porkpie-store/
│   ├── porkpie-sync/
│   ├── porkpie-api/
│   ├── porkpie-cli/
│   ├── porkpie-ui/
│   ├── porkpie-agent/
│   └── porkpie-import/
├── apps/
│   ├── desktop/
│   ├── web/
│   └── server/
├── infra/
│   ├── docker/
│   ├── compose/
│   └── caddy/
└── docs/
    ├── PRODUCT_SPEC.md
    ├── ARCHITECTURE.md
    ├── SECURITY_INVARIANTS.md
    ├── THREAT_MODEL.md
    ├── DATA_MODEL.md
    ├── SYNC_PROTOCOL.md
    ├── CRYPTO_FORMAT.md
    ├── AGENT_TASKS.md
    ├── TEST_PLAN.md
    └── ROADMAP.md
```

### Cargo.toml (porkpie-import)

```toml
[package]
name = "porkpie-import"
version = "0.1.0"
edition = "2021"

[dependencies]
porkpie-types = { path = "../porkpie-types" }
porkpie-core = { path = "../porkpie-core" }
porkpie-crypto = { path = "../porkpie-crypto" }
csv = "1.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4"] }

[dev-dependencies]
```

### Example: CSV Import (`src/csv.rs`)

```rust
use csv::ReaderBuilder;
use porkpie_core::Vault;
use porkpie_types::*;

pub async fn import_csv(
    path: &str,
    vault: &mut Vault,
) -> Result<usize, ImportError> {
    let file = std::fs::File::open(path)?;
    let mut reader = ReaderBuilder::new().from_reader(file);

    let mut count = 0;
    for result in reader.records() {
        let record = result?;
        
        // Parse CSV row
        let item_type = record.get(0).ok_or(ImportError::MissingField)?;
        let title = record.get(1).ok_or(ImportError::MissingField)?;
        
        // Create item (details omitted for brevity)
        // vault.create_item(item)?;
        
        count += 1;
    }

    Ok(count)
}
```

### Example: Backup Format (`src/encrypted_backup.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupFile {
    pub version: u32,
    pub vault_id: String,
    pub timestamp: i64,
    pub items: Vec<BackupItem>,  // encrypted
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupItem {
    pub id: String,
    pub item_type: String,
    pub ciphertext: Vec<u8>,     // encrypted with vault key
    pub created_at: i64,
}

pub async fn export_backup(vault: &Vault) -> Result<BackupFile> {
    let items = vault
        .list_items()?
        .into_iter()
        .map(|item| BackupItem {
            id: item.id.to_string(),
            item_type: item.item_type.clone(),
            ciphertext: porkpie_crypto::encrypt_item(item)?,
            created_at: item.created_at,
        })
        .collect();

    Ok(BackupFile {
        version: 1,
        vault_id: vault.id.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64,
        items,
    })
}

pub async fn import_backup(
    path: &str,
    password: &str,
    vault: &mut Vault,
) -> Result<usize> {
    let file = std::fs::File::open(path)?;
    let backup: BackupFile = serde_json::from_reader(file)?;

    let mut count = 0;
    for backup_item in backup.items {
        // Decrypt item
        let item = porkpie_crypto::decrypt_item(&backup_item.ciphertext)?;
        
        // Add to vault
        vault.create_item(item)?;
        
        count += 1;
    }

    Ok(count)
}
```

## 🔗 References

- **All Previous Tasks:** See TASK-01 through TASK-09
- **Security Invariants:** Task 2
- **Import/Export Details:** Porkpie Implementation Plan

## ✔️ Final Verification Commands

**Run in order. All must succeed.**

```bash
# 1. Format check
cargo fmt --all --check

# 2. Lint (zero warnings)
cargo clippy --workspace --all-targets -- -D warnings

# 3. All tests
cargo test --workspace -- --nocapture

# 4. Build (debug)
cargo build --workspace

# 5. Build (release)
cargo build --workspace --release

# 6. Verify binary
./target/debug/porkpie --version

# 7. Docker build
docker build -f Dockerfile -t porkpie:latest .

# 8. Docker Compose
docker-compose up --build
# Then: curl http://localhost:8000/api/v1/health
```

**Expected:** All commands succeed. No warnings. No errors.

## 📋 MVP Completion Checklist

**Functionality**
- [ ] Vault creation works end-to-end
- [ ] Vault unlock/lock works
- [ ] All 10 item types work
- [ ] CLI commands all work
- [ ] UI screens all work
- [ ] Sync protocol works
- [ ] Import/export works
- [ ] Docker deployment works

**Code Quality**
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] All code formatted
- [ ] No unimplemented! or todo! in prod
- [ ] All public APIs documented

**Security**
- [ ] All 13 invariants verified
- [ ] Encryption working (encrypt → decrypt)
- [ ] Tampering detected (MAC validation)
- [ ] Wrong password fails (doesn't decrypt)
- [ ] Memory zeroized (no plaintext leaks)
- [ ] Logs sanitized (no secrets)
- [ ] No hardcoded credentials
- [ ] No secrets in git

**Documentation**
- [ ] README complete
- [ ] All docs/ files written
- [ ] API docs for all public functions
- [ ] Architecture documented
- [ ] Security model documented

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
| "Tests are failing" | Run `cargo test --workspace -- --nocapture` to see output. Fix each failure. |
| "Clippy warnings persist" | Run `cargo clippy --workspace`. Follow suggestions. No warnings tolerated. |
| "Import/export is complex" | Start with CSV. Parse rows, create items, persist. Backup is just serialized + encrypted JSON. |
| "Docker build fails" | Check Dockerfile syntax. Ensure Cargo.toml versions match. Build locally first. |

## 📌 MISSION COMPLETE ✅

**When all items checked:**

1. ✅ Porkpie MVP built
2. ✅ All security invariants verified
3. ✅ All tests passing
4. ✅ Production code ready
5. ✅ Documentation complete
6. ✅ Ready to use / deploy

**Next steps:** User testing, real-world deployment, community feedback.

---

**Status:** Ready for agent assignment

**Recommended completion:** Tag repository with `v0.1.0-mvp` when done.

---

# Final Report Template

When Task 10 completes, generate this report:

```
# Porkpie MVP Completion Report

## Summary
[What was built, total effort, status]

## Files & Crates
[List all crates created, lines of code, test coverage]

## Commands Run
[Final build/test/lint output]

## Test Results
[Total tests, passing, coverage % if available]

## Security Verification
[All 13 invariants verified, specific tests]

## Known Limitations
[Deferred features, known issues]

## Deployment
[Docker image built, docker-compose working, server health check OK]

## Success Metrics
- ✅ Builds cleanly
- ✅ All tests pass
- ✅ Zero warnings
- ✅ Documented
- ✅ Secure
- ✅ Deployable

**Porkpie MVP: PRODUCTION READY**
```

---

**🎉 Congratulations! Porkpie MVP is complete.**
