---
title: Porkpie Architecture and Coding Plan
purpose: Foundational architecture specification and coding standards for Porkpie development
document_type: Architecture Specification
status: Active
last_updated: 2026-05-31
---

# Porkpie Architecture and Coding Plan

## 1. Product Identity

| Property | Value |
|----------|-------|
| **Name** | Porkpie |
| **Domain** | porkpie.love |
| **CLI Binary** | porkpie |
| **URI Scheme** | pie:// |
| **Hosted App** | app.porkpie.love |
| **Documentation** | docs.porkpie.love |
| **Sync Server** | sync.porkpie.love |

## 2. Product Positioning

**Definition:** Porkpie is a **local-first, zero-knowledge, self-hostable password manager** for developers, homelab users, and small teams.

### Secret Types Managed
- ✓ Passwords
- ✓ SSH keys
- ✓ API credentials
- ✓ Server records
- ✓ Database credentials
- ✓ Recovery codes
- ✓ Identity records
- ✓ Secure notes
- ✓ Software licenses
- ✓ Custom typed secrets

### Tagline Options
- "Secrets, safely served."
- "Put your secrets somewhere safer than a note file."
- "Local-first credential management for developers and homelabs."
- "A self-hostable vault for passwords, SSH keys, and developer secrets."

## 3. Foundation Stack

**CRITICAL:** Use production stack from day one. No disposable mockups.

### Required Stack

**Language & Ecosystem**
- Language: **Rust** (primary)
- Async Runtime: **Tokio**
- Build System: **Cargo** (workspace)
- Format/Lint: **rustfmt**, **clippy**

**UI & Interface**
- Desktop/Web UI: **Dioxus** (Rust, not Electron or TypeScript)
- CLI: **clap**

**Server & API**
- HTTP Framework: **Axum**
- Deployment: **Docker Compose**

**Storage & Data**
- Database: **SQLx** (SQLite, Postgres support)
- Serialization: **serde**

**Cryptography (RustCrypto crates)**
- Key Derivation: **argon2** (Argon2id parameters)
- Authenticated Encryption: **chacha20poly1305** (XChaCha20Poly1305 + AEAD)
- Secret Memory: **zeroize**, **secrecy** (volatile memory management)

**Observability**
- Logging: **tracing** (structured logs with secret redaction)

### PROHIBITED

- ❌ Electron
- ❌ TypeScript as foundation
- ❌ Mock/test-only crypto in real code
- ❌ Base64-only encoding (no real encryption)
- ❌ Plaintext secret persistence
- ❌ Server-side vault decryption
- ❌ Fake sync implementations
- ❌ UI-only features (features must have backend)

## 4. Repository Structure

```
porkpie/
├── Cargo.toml                    # Workspace root
├── README.md
├── crates/                       # Core libraries
│   ├── porkpie-types/            # Shared domain types & error types
│   ├── porkpie-crypto/           # Security-critical crypto operations
│   ├── porkpie-core/             # Vault domain logic & state
│   ├── porkpie-store/            # Local SQLite persistence
│   ├── porkpie-sync/             # Sync protocol & conflict resolution
│   ├── porkpie-api/              # HTTP server (Axum)
│   ├── porkpie-cli/              # Command-line interface
│   ├── porkpie-ui/               # Dioxus UI (desktop & web)
│   ├── porkpie-agent/            # AI agent integration points
│   └── porkpie-import/           # Data migration & import
├── apps/
│   ├── desktop/                  # Desktop app (Dioxus)
│   ├── web/                      # Web app (Dioxus)
│   └── server/                   # Sync server (Axum + SQLx)
├── infra/
│   ├── docker/
│   │   ├── Dockerfile.server
│   │   └── Dockerfile.cli
│   ├── compose/
│   │   └── docker-compose.yml
│   └── caddy/
│       └── Caddyfile
└── docs/
    ├── PRODUCT_SPEC.md           # Product requirements
    ├── ARCHITECTURE.md           # Expanded architecture
    ├── SECURITY_INVARIANTS.md    # Non-negotiable security rules
    ├── THREAT_MODEL.md           # Attack surface & mitigations
    ├── DATA_MODEL.md             # Entity relationships & schemas
    ├── SYNC_PROTOCOL.md          # Synchronization specification
    ├── CRYPTO_FORMAT.md          # Encryption format details
    ├── AGENT_TASKS.md            # Agentic instruction set
    ├── TEST_PLAN.md              # Testing strategy & coverage
    └── ROADMAP.md                # Feature roadmap & phases
```

## 5. Crate Responsibilities

### porkpie-types
**Purpose:** Single source of truth for shared domain types.

**Owns (MUST HAVE)**
- ✓ Unique identifiers (ItemId, VaultId, RevisionId)
- ✓ Temporal types (Timestamp, Duration)
- ✓ Item type enums (Login, APIKey, SSHKey, Note, etc.)
- ✓ Vault metadata structures
- ✓ Sync revision metadata
- ✓ Error types (custom errors, error codes)
- ✓ Import/export data structures
- ✓ Constants (max lengths, constraints)

**MUST NOT OWN**
- ✗ Encryption/decryption logic
- ✗ Database queries
- ✗ UI components
- ✗ HTTP routes

### porkpie-crypto
**Purpose:** Security-critical cryptographic operations. **ONLY place crypto happens.**

**Owns**
- ✓ Secret key generation (cryptographically secure random)
- ✓ Argon2id key derivation (configurable params)
- ✓ Vault key wrapping (encrypt master key)
- ✓ Item encryption (XChaCha20Poly1305)
- ✓ Item decryption with tamper detection
- ✓ Nonce generation (secure, non-reusing)
- ✓ Encrypted export format (binary spec)
- ✓ MAC validation (AEAD authentication)
- ✓ Test vectors (NIST, RFC compliance)

**RULES (NON-NEGOTIABLE)**
- ✓ No mock crypto in production paths
- ✓ No static keys or constants in encryption
- ✓ No nonce reuse (fail if duplicate)
- ✓ No plaintext output except decrypt() result
- ✓ No secret logging
- ✓ Wrong password → error (never decrypt)
- ✓ Tampered ciphertext → error (MAC fail)
- ✓ All operations constant-time where applicable

### porkpie-core
**Purpose:** Domain logic for vault lifecycle and item management.

**Owns**
- ✓ Vault creation (master password → wrapped key)
- ✓ Vault unlock (password → derived key → decrypt)
- ✓ Vault lock (clear decrypted items from memory)
- ✓ Item CRUD (create, read, update, delete)
- ✓ In-memory item list (while unlocked)
- ✓ Password generation (entropy, patterns)
- ✓ Revision tracking for sync

**Uses (delegates to)**
- porkpie-crypto for all encryption
- porkpie-store for persistence (via traits)

### porkpie-store
**Purpose:** Local encrypted blob persistence.

**Owns**
- ✓ SQLite schema design
- ✓ Connection pooling & management
- ✓ CRUD on encrypted blobs
- ✓ Query builders & filters
- ✓ Schema migrations
- ✓ Transaction handling

**CONTRACT:** Stores encrypted bytes only. Never decrypts.

### porkpie-sync
**Purpose:** Synchronization protocol and conflict resolution.

**Owns**
- ✓ Sync request/response types
- ✓ Revision-based merge strategy
- ✓ Conflict detection (same item modified locally & remotely)
- ✓ Merge logic (last-write-wins, custom rules)
- ✓ Sync state tracking (last synced revision)
- ✓ Delta calculation

### porkpie-api
**Purpose:** HTTP API for vault synchronization (server-side).

**Owns**
- ✓ Axum HTTP routes
- ✓ API authentication (keys, not passwords)
- ✓ Encrypted blob storage (no decryption)
- ✓ Request validation & sanitization
- ✓ Response serialization

**CONTRACT:** Stores encrypted data only. Server NEVER sees plaintext.

### porkpie-cli
**Purpose:** Command-line interface for vault management.

**Commands**
```
porkpie init              # Create new vault
porkpie unlock            # Unlock with master password
porkpie lock              # Lock vault
porkpie list              # List item summaries
porkpie get <id>          # Show decrypted secret
porkpie add <type>        # Create item (interactive)
porkpie edit <id>         # Edit item
porkpie delete <id>       # Delete item
porkpie export            # Export encrypted backup
porkpie import <file>     # Import encrypted backup
porkpie sync              # Sync with server
```

### porkpie-ui
**Purpose:** Desktop/Web user interface (Dioxus-based).

**Features**
- Onboarding (create vault, set password)
- Unlock dialog with password entry
- Item list view with search & filtering
- Item detail view & editor
- Password generator (entropy slider, patterns)
- Import/export UI
- Lock on timeout behavior
- Settings & preferences

### porkpie-agent
**Purpose:** Integration points for AI agent automation.

**Enables**
- Structured task input/output
- Agentic instruction following
- Progress reporting
- Error handling & recovery

### porkpie-import
**Purpose:** Data migration from other vaults.

**Supports**
- CSV (generic format)
- LastPass XML export
- 1Password JSON export
- Bitwarden JSON export
- Encrypted backup import

## 6. Security Invariants (Non-Negotiable)

These are fundamental requirements. Violating any is a security failure.

1. **No plaintext secret storage** — Anywhere (local DB, server DB, sync, logs, tests, backups)
2. **No master password storage** — Derived from user input on each unlock
3. **Client-side unlock only** — Server never sees master password or decrypted vault
4. **Server stores encrypted blobs only** — No server-side decryption
5. **Authenticated encryption required** — Every item encrypted with AEAD (XChaCha20Poly1305)
6. **Wrong password fails** — Never silently decrypt with wrong password
7. **Tampered ciphertext fails** — MAC must validate (authentication tag check)
8. **Locking clears memory** — All decrypted data purged on lock
9. **Plaintext export requires flag** — Must be explicit: `--dangerous-export-plaintext`
10. **No crypto shortcuts** — No base64-only, no mock crypto, no static keys, no placeholder crypto
11. **Logs sanitized** — Never include decrypted item contents
12. **No TODO in security paths** — Unimplemented crypto = immediate QA failure
13. **Unavailable features clearly marked** — UI shows "Not implemented" not fake data

## 7. Day-One Vertical Slice Acceptance Criteria

All of these must work end-to-end before declaring MVP complete:

### Build & Documentation
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo fmt --all` passes (no formatting issues)
- [ ] `cargo clippy --workspace -- -D warnings` passes (zero warnings)
- [ ] Core docs exist: PRODUCT_SPEC, ARCHITECTURE, SECURITY_INVARIANTS

### Cryptography (porkpie-crypto)
- [ ] Derive keys from passwords (Argon2id)
- [ ] Encrypt items (XChaCha20Poly1305)
- [ ] Decrypt items correctly
- [ ] Reject tampered ciphertext (MAC validation fails)
- [ ] Wrap/unwrap vault keys
- [ ] All crypto tests pass

### Vault Core (porkpie-core)
- [ ] Create new encrypted vault
- [ ] Unlock vault with correct password
- [ ] Reject unlock with wrong password
- [ ] Lock vault (clear decrypted state)
- [ ] Create/edit/delete items (in-memory)
- [ ] List items (decrypted)

### Item Types (all supported)
- [ ] Login (username + password)
- [ ] API Key (name + key)
- [ ] SSH Key (public + private key)
- [ ] Secure Note (plaintext)
- [ ] Server (hostname, port, credentials)
- [ ] Database (connection string, user, pass)
- [ ] Identity (personal info)
- [ ] Software License (product + key)
- [ ] Recovery Codes (list of codes)
- [ ] Custom (key-value pairs)

### Persistence (porkpie-store)
- [ ] Store encrypted vault to SQLite
- [ ] Retrieve encrypted vault from SQLite
- [ ] Query items by ID

### CLI (porkpie-cli)
- [ ] `porkpie init` — Create vault
- [ ] `porkpie unlock` — Unlock vault
- [ ] `porkpie lock` — Lock vault
- [ ] `porkpie list` — Show items
- [ ] `porkpie get <id>` — Decrypt & display secret
- [ ] `porkpie export` — Encrypted backup
- [ ] `porkpie import` — Import encrypted backup

### UI (porkpie-ui)
- [ ] Onboarding flow (create vault, set password)
- [ ] Unlock dialog
- [ ] Item list view with search
- [ ] Item detail/edit view
- [ ] Password generator
- [ ] Import/export dialogs
- [ ] Lock on timeout
- [ ] Settings panel

### Sync (porkpie-api)
- [ ] HTTP server starts with Docker Compose
- [ ] Stores encrypted sync revisions
- [ ] Rejects non-encrypted data
- [ ] Handles sync requests

### Testing
- [ ] `cargo test --workspace` passes
- [ ] Security invariants verified by tests
- [ ] Crypto operations tested (encryption, decryption, tampering)

## 8. Day-One Non-Goals

NOT building in MVP (explicitly deferred):
- ✗ OpenSSH agent integration
- ✗ Browser extension
- ✗ Mobile app
- ✗ Passkeys / WebAuthn
- ✗ Team sharing / access control
- ✗ Emergency access procedures
- ✗ Production security audit / certification
- ✗ Penetration testing

## 9. Code Quality Standards

### Before Declaring Task Complete

**ALL commands must pass:**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

### No Exceptions
- Clippy warnings = build failure
- Format violations = build failure
- Unused imports = build failure
- Test failures = build failure

## 10. Task Completion Report Format

Every completed task must include:

1. **Summary** — What was built, key decisions
2. **Files Created/Modified** — List of new/changed files
3. **Commands & Output** — Build, test, lint results
4. **Test Results** — Tests passed, coverage, edge cases
5. **Security Notes** — Crypto decisions, invariant compliance
6. **Known Limitations** — What's NOT done yet
7. **Next Recommended Task** — Which task to do next

## 11. Agentic Implementation Guidelines

**For AI agents working on this project:**

✓ **DO:**
- Make reasonable architectural decisions aligned with this spec
- Document decisions in commit messages
- Keep implementation progressing toward MVP
- Prioritize security and correctness over speed
- Report progress in standard format
- Flag any architectural conflicts immediately

✗ **DON'T:**
- Ask broad clarification questions (make decisions)
- Skip security invariant verification
- Ship mock crypto or test-only code to main
- Mix encrypted and plaintext storage
- Add features not in the MVP scope
