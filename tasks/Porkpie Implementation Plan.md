---
title: Porkpie Implementation Plan
purpose: Detailed technical implementation roadmap and agent instructions
document_type: Implementation Plan
status: Active
last_updated: 2026-05-31
---

# Porkpie Implementation Plan

**Role:** You are the lead implementation agent for Porkpie.

**Mission:** Build a foundational Rust vertical slice—production code, not a disposable mockup.

---

## Product Context

| Aspect | Details |
|--------|---------|
| **What** | Local-first, zero-knowledge, self-hostable password manager |
| **Who** | Developers, homelab users, small teams |
| **Domain** | porkpie.love |
| **CLI Binary** | porkpie |
| **URI Scheme** | pie:// |
| **Secret Types** | Passwords, SSH keys, API credentials, server records, DB credentials, recovery codes, notes, licenses, custom |

---

## Hard Constraints

### Tech Stack (Non-Negotiable)
- ✓ **Rust** (primary language)
- ✓ **Cargo workspace** (build system)
- ✓ **Dioxus** (Rust UI, desktop & web)
- ✓ **Axum** (HTTP sync server)
- ✓ **Tokio** (async runtime)
- ✓ **SQLx** (SQLite/Postgres)
- ✓ **clap** (CLI)
- ✓ **RustCrypto argon2** (Argon2id)
- ✓ **RustCrypto chacha20poly1305** (XChaCha20Poly1305)
- ✓ **serde** (serialization)
- ✓ **zeroize + secrecy** (secret memory)
- ✓ **tracing** (structured logs)
- ✓ **Docker Compose** (deployment)

### PROHIBITED
- ❌ Electron
- ❌ TypeScript as foundation
- ❌ Mock crypto in production code paths
- ❌ Base64-only encoding
- ❌ Plaintext secret storage anywhere
- ❌ Server-side vault decryption
- ❌ Fake sync
- ❌ UI-only features

### Repository Structure (Required)
```
porkpie/
├── Cargo.toml
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

---

## Hard Security Invariants

**Non-negotiable. Violating any is a security failure.**

1. ✓ **No plaintext secrets stored** anywhere
   - Not in local DB, server DB, sync, logs, tests, backups
2. ✓ **No master password stored** (derive on each unlock)
3. ✓ **Client-side unlock only** (server never sees master password)
4. ✓ **Server stores encrypted blobs only** (no decryption server-side)
5. ✓ **Authenticated encryption required** (AEAD for all items)
6. ✓ **Wrong password fails** (never decrypt silently)
7. ✓ **Tampered ciphertext fails** (MAC validation)
8. ✓ **Locking clears decrypted state** (memory purge)
9. ✓ **Plaintext export requires explicit dangerous flag**
10. ✓ **No crypto shortcuts** (no mock crypto, no static keys, no placeholders)
11. ✓ **Logs never include decrypted contents**
12. ✓ **No TODO in security-critical paths** (unimplemented crypto = QA fail)
13. ✓ **Unavailable features clearly marked** (no fake features in UI)

---

## Day-One Vertical Slice Requirements

### What MUST work end-to-end by Task 1 completion

**Core Functionality**
1. Rust workspace builds (`cargo build --workspace`)
2. Documentation exists and is complete
3. porkpie-crypto works:
   - Derive keys from passwords (Argon2id)
   - Encrypt items (XChaCha20Poly1305)
   - Decrypt items
   - Wrap/unwrap vault keys
   - Reject tampering
4. porkpie-core works:
   - Create encrypted vault
   - Unlock vault (verify password)
   - Lock vault (clear memory)
   - Manage items (CRUD)
5. Item types fully defined:
   - Login, API Key, SSH Key, Secure Note
   - Server, Database, Identity
   - Software License, Recovery Codes, Custom
6. porkpie-store persists encrypted vault to SQLite
7. porkpie-cli supports:
   - `init` — Create vault
   - `unlock` — Unlock
   - `list` — List items
   - `get <id>` — Read decrypted secret
   - `export/import` — Encrypted backup
8. porkpie-ui provides:
   - Onboarding (create vault)
   - Unlock dialog
   - Item list
   - Item editor
   - Password generator
   - Import/export
   - Lock behavior
9. porkpie-api stores encrypted sync revisions (no decryption)
10. Docker Compose starts the server
11. All tests pass (`cargo test --workspace`)
12. All security invariants verified by tests

---

## Day-One Non-Goals

**Explicitly NOT building in MVP:**
- ✗ OpenSSH agent integration
- ✗ Browser extension
- ✗ Mobile app
- ✗ Passkeys / WebAuthn
- ✗ Team sharing
- ✗ Emergency access
- ✗ Production security audit
- ✗ Penetration testing

---

## Code Quality Requirements

### Before Declaring Task Complete

**ALL of these must pass:**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

**No exceptions:**
- Clippy warnings = failure
- Format violations = failure
- Test failures = failure

---

## Task Completion Report Format

**Every completed task must include:**

```
## Summary
[What was built, key decisions, status]

## Files Changed
- [new/modified files]
- [directory structures]

## Commands Run
[cargo build output]
[cargo test results]
[cargo clippy results]
[cargo fmt results]

## Test Results
[Tests passed count]
[Coverage notes]
[Edge cases handled]

## Security Notes
[Crypto decisions made]
[Invariants verified]
[Edge cases in security paths]

## Known Limitations
[What's NOT done]
[Why deferred]

## Next Recommended Task
[Which task to start next]
```

---

## Agent Instructions (Critical)

**When implementing tasks:**

### DO ✓
- Make reasonable decisions aligned with architecture
- Document decisions in commit messages
- Keep implementation progressing toward MVP
- Prioritize security and correctness over speed
- Report progress in standard format
- Flag architectural conflicts immediately
- Test security invariants continuously

### DON'T ✗
- Ask broad clarification questions (make decisions)
- Skip security invariant verification
- Ship mock or test-only code to main paths
- Mix encrypted and plaintext storage
- Add features outside MVP scope
- Commit with TODO in security-critical code
- Ignore compiler/clippy warnings
- Ship failing tests

### Decision-Making Guidelines

When faced with ambiguity:

1. **Choose conservative security** over convenience
2. **Prefer explicit over implicit** (fail loud, not silent)
3. **Encrypt by default** (plaintext requires justification)
4. **Test edge cases** (wrong password, tampering, lock/unlock cycles)
5. **Document non-obvious decisions** in code comments
6. **Align with existing crate patterns** (don't invent new conventions)
7. **Keep crates focused** (single responsibility)

---

## Recommended Task Sequencing

### Phase 1: Foundation (Days 1-2)
1. Workspace & docs scaffold
2. Security & architecture docs
3. Core types & errors

### Phase 2: Cryptography (Days 3-4)
4. Crypto operations & tests
5. Key derivation & encryption

### Phase 3: Vault Logic (Days 5-6)
6. Vault core (create, unlock, lock)
7. Item management (CRUD)

### Phase 4: Storage (Days 7-8)
8. SQLite schema & persistence
9. Store crate implementation

### Phase 5: Interfaces (Days 9-10)
10. CLI implementation
11. UI scaffold (Dioxus)

### Phase 6: Sync & Integration (Days 11-12)
12. Sync protocol
13. HTTP API server
14. Docker Compose

### Phase 7: Polish & Completion (Days 13-15)
15. Import/export
16. Full test coverage
17. Documentation polish
18. Final validation

---

## Progress Tracking

**Keep implementation moving:**
- One task per day minimum
- Daily commit with progress report
- No blocking issues without escalation
- Security review after each phase
- Full test suite run after each major feature

**Success criteria:** MVP shipped with all security invariants verified.
