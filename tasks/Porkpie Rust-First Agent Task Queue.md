---
title: Porkpie Rust-First Agent Task Queue
purpose: Sequenced task breakdown for AI agent implementation
document_type: Task Queue
status: Active
last_updated: 2026-05-31
---

# Porkpie Rust-First Agent Task Queue

This document breaks down the MVP vertical slice into discrete, sequential tasks for agent implementation.

---

## Task 1: Workspace and Documentation Scaffold

**Objective:** Create the Rust workspace structure and foundational documentation.

**Owner:** Agent

**Acceptance Criteria**
- [ ] `cargo metadata` works
- [ ] `cargo fmt --all` passes (no formatting issues)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes (placeholder tests OK)
- [ ] `cargo build --workspace` succeeds
- [ ] README.md explains Porkpie, porkpie.love, pie://

### Required Crates
```
porkpie-types/
porkpie-crypto/
porkpie-core/
porkpie-store/
porkpie-sync/
porkpie-api/
porkpie-cli/
porkpie-ui/
porkpie-agent/
porkpie-import/
```

### Required Apps
```
apps/desktop/
apps/web/
apps/server/
```

### Required Documentation Files
```
docs/PRODUCT_SPEC.md
docs/ARCHITECTURE.md
docs/SECURITY_INVARIANTS.md
docs/THREAT_MODEL.md
docs/DATA_MODEL.md
docs/SYNC_PROTOCOL.md
docs/CRYPTO_FORMAT.md
docs/AGENT_TASKS.md
docs/TEST_PLAN.md
docs/ROADMAP.md
```

**Output Format**
- Root Cargo.toml with workspace members
- Each crate with lib.rs containing module placeholders
- Each crate with Cargo.toml with dependencies (minimal)
- Root README.md (brief product overview)
- All doc files as markdown stubs (structure defined, content TBD by later tasks)

**Next Task:** Task 2 (Security & Architecture Docs)

---

## Task 2: Security and Architecture Documentation

**Objective:** Write core security and architecture specifications.

**Owner:** Agent

**Acceptance Criteria**
- [ ] SECURITY_INVARIANTS.md complete
- [ ] THREAT_MODEL.md complete
- [ ] ARCHITECTURE.md complete
- [ ] DATA_MODEL.md complete (schema diagrams OK)
- [ ] All documents link together (cross-references work)

### SECURITY_INVARIANTS.md Must Include
- ✓ Zero-knowledge model explanation
- ✓ Local-first design principles
- ✓ 13 security invariants (non-negotiable rules)
- ✓ Plaintext secret handling rules
- ✓ Master password derivation rules
- ✓ Client-side unlock rules
- ✓ Server storage constraints
- ✓ Authenticated encryption requirements
- ✓ Wrong password/tampering failure modes
- ✓ Memory clearing on lock
- ✓ Export safety rules
- ✓ Crypto implementation rules
- ✓ Logging sanitization rules

### THREAT_MODEL.md Must Include
- ✓ Attack scenarios (password guessing, server breach, local compromise)
- ✓ Mitigations for each scenario
- ✓ In-scope vs out-of-scope threats
- ✓ Assumptions (trusted OS, no backdoored libraries)

### ARCHITECTURE.md Must Include
- ✓ High-level component diagram (ASCII OK)
- ✓ Data flow (vault creation → unlock → item management → sync)
- ✓ Crate dependencies (who uses who)
- ✓ Security boundaries (client vs server)
- ✓ Encryption boundaries (where crypto happens)

### DATA_MODEL.md Must Include
- ✓ Entity types (Vault, Item, Revision, etc.)
- ✓ Item types (Login, APIKey, SSHKey, etc.)
- ✓ Field definitions
- ✓ Relationships
- ✓ Constraints

**Output Format**
- Markdown documents with clear sections
- ASCII diagrams where helpful
- Links between docs (use relative paths)

**Next Task:** Task 3 (Core Types & Errors)

---

## Task 3: Core Types and Error Definitions

**Objective:** Implement porkpie-types crate with all domain types.

**Owner:** Agent

**Acceptance Criteria**
- [ ] porkpie-types compiles without warnings
- [ ] All types implement Serialize, Deserialize
- [ ] All item type enums defined
- [ ] Custom errors defined with error codes
- [ ] Unit tests for type serialization

### Must Define Types
- **IDs:** ItemId, VaultId, RevisionId, UserId (UUID-based)
- **Timestamps:** (i64 unix millis)
- **Item Types Enum:** Login, APIKey, SSHKey, SecureNote, Server, Database, Identity, SoftwareLicense, RecoveryCodes, Custom
- **Secret Types:** By item (LoginSecret, APIKeySecret, etc.)
- **Vault Metadata:** creation time, version, key derivation params
- **Sync Metadata:** revision number, timestamp, item hash
- **Errors:** VaultError, CryptoError, SyncError, StorageError (with error codes)

### Item Schemas
```
Login:
  - username: String
  - password: String (encrypted)
  - url: Option<String>
  - notes: Option<String>

APIKey:
  - name: String
  - key: String (encrypted)
  - provider: String

SSHKey:
  - name: String
  - public_key: String
  - private_key: String (encrypted)
  - passphrase: Option<String> (encrypted)

SecureNote:
  - title: String
  - content: String (encrypted)

Server:
  - hostname: String
  - port: u16
  - username: Option<String>
  - password: Option<String> (encrypted)
  - notes: Option<String>

Database:
  - engine: String (postgres, mysql, sqlite)
  - host: String
  - port: u16
  - username: String
  - password: String (encrypted)
  - database: String

Identity:
  - full_name: String
  - email: String
  - phone: Option<String>
  - address: Option<String>

SoftwareLicense:
  - product: String
  - license_key: String (encrypted)
  - version: String
  - expiry: Option<i64>

RecoveryCodes:
  - codes: Vec<String> (encrypted)

Custom:
  - fields: Map<String, String> (values encrypted if marked)
```

**Output Format**
- lib.rs with all type definitions
- src/types.rs (items, ids, metadata)
- src/errors.rs (error types with codes)
- src/secrets.rs (encrypted field types)
- tests/serialization.rs (ser/de tests)

**Next Task:** Task 4 (Cryptographic Functions)

---

## Task 4: Cryptographic Operations

**Objective:** Implement porkpie-crypto crate with all encryption/decryption.

**Owner:** Agent

**Dependencies:** porkpie-types (Task 3 complete)

**Acceptance Criteria**
- [ ] porkpie-crypto compiles without warnings
- [ ] Argon2id key derivation works
- [ ] XChaCha20Poly1305 encryption works
- [ ] XChaCha20Poly1305 decryption works
- [ ] Tamper detection rejects modified ciphertext
- [ ] Wrong password produces error (never decrypts)
- [ ] All crypto tests pass
- [ ] No clippy warnings

### Must Implement

**Key Derivation**
```rust
pub fn derive_key(password: &str, salt: &[u8; 32], params: &Argon2Params) -> Result<[u8; 32]>
```

**Vault Key Wrapping**
```rust
pub fn wrap_vault_key(master_key: &[u8; 32], vault_key: &[u8; 32]) -> Result<EncryptedVaultKey>
pub fn unwrap_vault_key(master_key: &[u8; 32], wrapped: &EncryptedVaultKey) -> Result<[u8; 32]>
```

**Item Encryption**
```rust
pub fn encrypt_item<T: Serialize>(item: &T, key: &[u8; 32]) -> Result<Vec<u8>>
pub fn decrypt_item<T: DeserializeOwned>(ciphertext: &[u8], key: &[u8; 32]) -> Result<T>
```

**Nonce Management**
```rust
pub fn generate_nonce() -> [u8; 24]  // XChaCha20 nonce
```

### Security Requirements

✓ Use `chacha20poly1305` for AEAD (XChaCha20Poly1305)
✓ Use `argon2` for key derivation (Argon2id)
✓ Use `rand` for secure random (no predictable nonces)
✓ Use `zeroize` for sensitive data
✓ Validate MAC on decrypt (fail if tampered)
✓ No logging of secrets
✓ Constant-time comparisons where applicable

### Test Cases

**Encryption/Decryption**
- ✓ Encrypt plaintext → ciphertext
- ✓ Decrypt ciphertext → plaintext matches original
- ✓ Different inputs produce different ciphertexts (nonce randomness)

**Tamper Detection**
- ✓ Modify ciphertext byte → decrypt fails
- ✓ Modify authentication tag → decrypt fails
- ✓ Truncate ciphertext → decrypt fails

**Key Derivation**
- ✓ Same password + salt → same key
- ✓ Different salt → different key
- ✓ Argon2id parameters configurable

**Vault Key**
- ✓ Wrap key → unwrap → same key
- ✓ Wrong master key → unwrap fails
- ✓ Modified wrapped key → unwrap fails

**Error Handling**
- ✓ Wrong password detected
- ✓ Corrupted ciphertext rejected
- ✓ Invalid parameters caught

**Output Format**
- lib.rs with module structure
- src/key_derivation.rs (Argon2id)
- src/encryption.rs (AEAD operations)
- src/nonce.rs (nonce generation)
- src/errors.rs (crypto-specific errors)
- tests/crypto.rs (comprehensive crypto tests)
- tests/vectors.rs (NIST test vectors)

**Next Task:** Task 5 (Vault Core)

---

## Task 5: Vault Core Logic

**Objective:** Implement porkpie-core crate with vault lifecycle and item management.

**Owner:** Agent

**Dependencies:** porkpie-types (Task 3), porkpie-crypto (Task 4)

**Acceptance Criteria**
- [ ] Vault creation works
- [ ] Vault unlock works (derive key, verify)
- [ ] Vault lock works (clear memory)
- [ ] Item CRUD works (in-memory while unlocked)
- [ ] All tests pass
- [ ] No clippy warnings

### Core Types & Functions

**Vault Struct**
```rust
pub struct Vault {
    id: VaultId,
    created_at: Timestamp,
    master_key_wrapped: EncryptedVaultKey,  // encrypted with derived key
    items: HashMap<ItemId, Item>,           // in-memory, only when unlocked
    is_locked: bool,
}
```

**Vault Operations**
```rust
pub fn create_vault(password: &str) -> Result<Vault>
pub fn unlock(&mut self, password: &str, master_key_wrapped: &EncryptedVaultKey) -> Result<()>
pub fn lock(&mut self) -> Result<()>
```

**Item Operations**
```rust
pub fn create_item(&mut self, item: Item) -> Result<ItemId>
pub fn get_item(&self, id: ItemId) -> Result<&Item>
pub fn update_item(&mut self, id: ItemId, item: Item) -> Result<()>
pub fn delete_item(&mut self, id: ItemId) -> Result<()>
pub fn list_items(&self) -> Result<Vec<&Item>>
```

**Password Generation**
```rust
pub fn generate_password(length: usize, patterns: &Patterns) -> String
```

### Item Type Support

All 10 item types supported (Login, APIKey, SSHKey, Note, Server, Database, Identity, License, Recovery, Custom)

### Test Cases

- ✓ Create vault with password
- ✓ Unlock vault with correct password
- ✓ Fail unlock with wrong password
- ✓ Lock vault (verify items cleared)
- ✓ Create/read/update/delete items
- ✓ List items (empty when locked)
- ✓ Generate passwords
- ✓ Error cases (create item while locked, etc.)

**Output Format**
- lib.rs with module structure
- src/vault.rs (Vault struct & operations)
- src/item.rs (Item definitions)
- src/password_gen.rs (password generation)
- tests/vault_lifecycle.rs (vault operations)
- tests/item_crud.rs (item operations)

**Next Task:** Task 6 (SQLite Storage)

---

## Task 6: Local Storage (SQLx & SQLite)

**Objective:** Implement porkpie-store crate for encrypted vault persistence.

**Owner:** Agent

**Dependencies:** porkpie-types, porkpie-crypto (Tasks 3-4)

**Acceptance Criteria**
- [ ] SQLite schema created
- [ ] CRUD operations work (store/retrieve encrypted blobs)
- [ ] Migrations work
- [ ] Connection pooling works
- [ ] All tests pass
- [ ] No clippy warnings

### Schema (SQLite)

```sql
CREATE TABLE vaults (
  id TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL,
  master_key_wrapped BLOB NOT NULL,  -- encrypted, never decrypted by store
  sync_revision INTEGER DEFAULT 0
);

CREATE TABLE items (
  id TEXT PRIMARY KEY,
  vault_id TEXT NOT NULL REFERENCES vaults(id),
  item_type TEXT NOT NULL,           -- Login, APIKey, etc.
  ciphertext BLOB NOT NULL,          -- encrypted item (never decrypted by store)
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  sync_revision INTEGER DEFAULT 0,
  FOREIGN KEY (vault_id) REFERENCES vaults(id)
);

CREATE TABLE sync_state (
  vault_id TEXT PRIMARY KEY REFERENCES vaults(id),
  last_synced_revision INTEGER,
  last_synced_at INTEGER
);
```

### CRUD Operations

**Vault Storage**
```rust
pub async fn store_vault(db: &Pool, vault: &Vault) -> Result<()>
pub async fn load_vault(db: &Pool, vault_id: &VaultId) -> Result<EncryptedVaultData>
pub async fn delete_vault(db: &Pool, vault_id: &VaultId) -> Result<()>
```

**Item Storage**
```rust
pub async fn store_item(db: &Pool, vault_id: &VaultId, item: &Item) -> Result<()>
pub async fn load_item(db: &Pool, item_id: &ItemId) -> Result<Vec<u8>>  // encrypted
pub async fn list_items(db: &Pool, vault_id: &VaultId) -> Result<Vec<ItemId>>
pub async fn delete_item(db: &Pool, item_id: &ItemId) -> Result<()>
```

### Key Rules

✓ All data stored encrypted (store layer never decrypts)
✓ Store layer knows nothing about plaintext
✓ Connection pooling for performance
✓ Migrations for schema evolution
✓ Error handling for DB constraints

### Test Cases

- ✓ Store & load vault
- ✓ Store & load items
- ✓ List items for vault
- ✓ Update item
- ✓ Delete item
- ✓ Foreign key constraints
- ✓ Error cases (vault not found, etc.)

**Output Format**
- lib.rs
- src/db.rs (connection pooling)
- src/vault_store.rs (vault operations)
- src/item_store.rs (item operations)
- src/migrations.rs (schema migrations)
- tests/store.rs (storage tests)

**Next Task:** Task 7 (CLI Implementation)

---

## Task 7: Command-Line Interface

**Objective:** Implement porkpie-cli crate with all CLI commands.

**Owner:** Agent

**Dependencies:** All previous tasks (Vault, Crypto, Store)

**Acceptance Criteria**
- [ ] All commands compile
- [ ] All commands work end-to-end
- [ ] Help text is clear
- [ ] Error messages are helpful
- [ ] All tests pass
- [ ] No clippy warnings

### Required Commands

```
porkpie init                 # Create new vault (interactive)
porkpie unlock               # Unlock vault with password
porkpie lock                 # Lock vault
porkpie list                 # List items (requires unlocked vault)
porkpie get <id>             # Show decrypted secret
porkpie add <type>           # Create item (interactive)
porkpie edit <id>            # Edit item (interactive)
porkpie delete <id>          # Delete item
porkpie export               # Export encrypted backup
porkpie import <file>        # Import encrypted backup
porkpie sync                 # Sync with server (basic stub)
porkpie help [command]       # Help text
```

### Implementation Details

**init**
- Prompt for master password (min 16 chars)
- Create vault in local SQLite
- Output vault ID

**unlock**
- Prompt for vault ID
- Prompt for master password
- Load vault, unlock with password
- Set session state (unlocked)

**lock**
- Clear session state
- Clear decrypted items from memory

**list**
- Require unlocked vault
- Show item summaries (type, title, created_at)

**get**
- Require unlocked vault
- Show decrypted secret (full content)

**add/edit**
- Prompt for item fields by type
- Validate required fields
- Store encrypted

**export**
- Create encrypted backup file
- Format: JSON encrypted with temporary key

**import**
- Load encrypted backup
- Decrypt with provided password
- Merge into vault

### Output Format

- lib.rs with command structure
- src/commands/ (each command in separate file)
- src/interactive.rs (prompt utilities)
- tests/cli.rs (integration tests)

**Next Task:** Task 8 (UI with Dioxus)

---

## Task 8: User Interface (Dioxus)

**Objective:** Implement porkpie-ui crate with desktop/web interface.

**Owner:** Agent

**Dependencies:** All vault/crypto/store crates

**Acceptance Criteria**
- [ ] UI compiles for desktop
- [ ] UI compiles for web
- [ ] All screens render
- [ ] Interactions work (create, unlock, list, edit)
- [ ] Password generator works
- [ ] Lock on timeout works
- [ ] Tests pass (UI component tests)

### Required Screens

1. **Onboarding** — Create new vault
2. **Unlock** — Password entry, unlock button
3. **List** — Item list with search
4. **Item Detail** — View/edit item
5. **Password Generator** — Entropy slider, patterns
6. **Import/Export** — File dialogs
7. **Settings** — Timeout, appearance

### Core Components

**VaultManager** — Session state (locked/unlocked)
**ItemList** — Filterable item display
**ItemEditor** — Form for create/edit
**PasswordGen** — Interactive password generator
**UnlockDialog** — Password input

### Test Cases

- ✓ Render all screens
- ✓ Navigation between screens
- ✓ Form submission
- ✓ Validation (password strength, etc.)
- ✓ Lock on timeout
- ✓ Error dialogs

**Output Format**
- lib.rs with component structure
- src/components/ (reusable UI components)
- src/pages/ (full page screens)
- src/state.rs (vault session state)
- tests/ui.rs (component tests)

**Next Task:** Task 9 (Sync Protocol & API)

---

## Task 9: Sync Protocol and HTTP API

**Objective:** Implement porkpie-sync and porkpie-api crates.

**Owner:** Agent

**Dependencies:** All previous crates

**Acceptance Criteria**
- [ ] Sync protocol defined
- [ ] HTTP API routes defined
- [ ] Conflict resolution logic works
- [ ] Docker Compose starts server
- [ ] All tests pass

### Sync Protocol

**Revision-Based Merge**
- Server stores encrypted items by revision number
- Client sends local revision → receives server changes
- Merge strategy: last-write-wins (client can override)

**Conflict Detection**
- Same item modified locally + remotely → manual merge

**Sync Flow**
1. Client sends: vault_id, last_sync_revision
2. Server returns: items changed since that revision
3. Client merges (conflict resolution if needed)
4. Client sends: updated items (encrypted)
5. Server stores with new revision number

### HTTP API Routes (porkpie-api)

```
POST /api/v1/sync/begin
  Request: {vault_id, last_revision}
  Response: {items, new_revision}

POST /api/v1/sync/push
  Request: {vault_id, items}
  Response: {success, revision}

GET /api/v1/status
  Response: {server_version, timestamp}
```

### Security

✓ API key authentication (not passwords)
✓ Stores encrypted blobs only (no decryption)
✓ Never sees plaintext
✓ Rate limiting
✓ HTTPS (TLS)

**Output Format**
- porkpie-sync/lib.rs (sync logic)
- porkpie-api/lib.rs (Axum routes)
- src/handlers.rs (route handlers)
- src/models.rs (request/response types)
- tests/sync.rs (sync tests)
- docker-compose.yml (server deployment)

**Next Task:** Task 10 (Import/Export & Final Polish)

---

## Task 10: Import/Export and Final Validation

**Objective:** Complete import/export and ensure MVP is complete.

**Owner:** Agent

**Dependencies:** All previous tasks

**Acceptance Criteria**
- [ ] Import from CSV works
- [ ] Import from encrypted backup works
- [ ] Export to encrypted backup works
- [ ] Full test suite passes
- [ ] All clippy warnings resolved
- [ ] All code formatted
- [ ] Security invariants verified
- [ ] Documentation complete

### Import Formats

**CSV**
- Columns: item_type, title, username, password, notes
- Encrypted on import

**Encrypted Backup**
- Format: JSON encrypted with backup password
- Can be re-imported later

### Export Formats

**Encrypted Backup** (default)
- Creates JSON backup encrypted with master key
- Can be imported back

**Plaintext** (dangerous)
- Flag: `--dangerous-export-plaintext`
- Requires confirmation
- Exports JSON plaintext (DANGEROUS)

### Final Checks

```bash
✓ cargo fmt --all        # All formatted
✓ cargo clippy --all     # Zero warnings
✓ cargo test --all       # All tests pass
✓ cargo build --release  # Release build succeeds
✓ Documentation complete # All docs written
✓ Security verified      # Invariants checked
✓ MVP functional         # All features work
```

### Output Format

- porkpie-import/lib.rs
- src/csv.rs (CSV import)
- src/encrypted_backup.rs (backup format)
- tests/import.rs (import tests)
- tests/export.rs (export tests)
- tests/e2e.rs (end-to-end integration tests)
- Comprehensive README.md
- Updated docs/

---

## Task Completion Checklist

For each task, verify:

- [ ] Code compiles without warnings
- [ ] `cargo clippy` passes
- [ ] `cargo fmt` passes
- [ ] `cargo test` passes (all tests)
- [ ] No `TODO` in security-critical code
- [ ] Commit message documents decisions
- [ ] Security invariants maintained
- [ ] Next task identified

---

## Success Criteria for MVP Completion

When all tasks complete:

- ✓ Rust workspace builds cleanly
- ✓ Vault creation, unlock, lock work
- ✓ All item types supported
- ✓ CLI & UI both functional
- ✓ Encryption/decryption verified
- ✓ Sync protocol working
- ✓ Server running via Docker Compose
- ✓ All tests passing
- ✓ Security invariants validated
- ✓ Full documentation present
- ✓ Zero security shortcuts taken

**Porkpie MVP is ready for real-world use.**
