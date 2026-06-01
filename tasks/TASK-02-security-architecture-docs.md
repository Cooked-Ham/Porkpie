---
task_id: 02-docs
task_name: Security and Architecture Documentation
sequence: 2
dependencies_complete: [01-workspace]
estimated_duration: 4-5 hours
difficulty: Medium
blockers_resolved: none
can_parallelize: false
---

# Task 2: Security and Architecture Documentation

## 🎯 Objective

Write comprehensive security and architecture specifications. These documents define what you're building and how to verify it's secure. Later tasks use these as reference material.

## ✅ Acceptance Criteria

**SECURITY_INVARIANTS.md**
- [ ] 13 security invariants documented
- [ ] Each invariant includes:
  - [ ] Rule statement
  - [ ] Why it matters
  - [ ] How it's enforced (code/tests)
- [ ] Covers: plaintext storage, password handling, encryption, tampering, logging, memory
- [ ] Examples of GOOD vs BAD approaches

**THREAT_MODEL.md**
- [ ] Attack scenarios listed (≥5):
  - [ ] Password guessing (Argon2id mitigation)
  - [ ] Server breach (encrypted data only)
  - [ ] Local machine compromise (client-side decryption)
  - [ ] Network eavesdropping (HTTPS/TLS)
  - [ ] Supply chain (dependency trust)
- [ ] Each scenario includes mitigation
- [ ] In-scope vs out-of-scope threats listed
- [ ] Assumptions documented (OS is trusted, no rootkits, etc.)

**ARCHITECTURE.md**
- [ ] Component diagram (ASCII or Mermaid OK)
- [ ] Data flow diagram (vault → items → sync)
- [ ] Crate dependency graph
- [ ] Security boundaries marked (client vs server)
- [ ] Encryption boundaries marked (where crypto happens)
- [ ] Each major component described (1-2 paragraphs)

**DATA_MODEL.md**
- [ ] Vault entity defined (fields, metadata)
- [ ] Item entity defined (base + type-specific)
- [ ] 10 item types defined:
  - [ ] Login (username, password, url, notes)
  - [ ] APIKey (name, key, provider)
  - [ ] SSHKey (name, public_key, private_key, passphrase)
  - [ ] SecureNote (title, content)
  - [ ] Server (hostname, port, username, password)
  - [ ] Database (engine, host, port, username, password, database)
  - [ ] Identity (name, email, phone, address)
  - [ ] SoftwareLicense (product, key, version, expiry)
  - [ ] RecoveryCodes (list of codes)
  - [ ] Custom (key-value pairs)
- [ ] Relationships documented (Vault → Items)
- [ ] Constraints documented (max lengths, required fields)
- [ ] SQLite schema outline (table structures)

**Code Quality**
- [ ] All docs valid markdown
- [ ] No broken links (internal references work)
- [ ] Cross-references between docs exist
- [ ] Consistent formatting
- [ ] No placeholder text (e.g., "[TODO]")

## 📋 Output Specification

### File Locations

```
docs/
├── SECURITY_INVARIANTS.md    # 13 invariants + enforcement
├── THREAT_MODEL.md            # Attacks + mitigations
├── ARCHITECTURE.md            # Diagrams + component descriptions
└── DATA_MODEL.md              # Entities + schemas
```

### SECURITY_INVARIANTS.md Structure

```markdown
# Security Invariants

## 1. No Plaintext Secrets Stored

**The Rule:** Secrets must NEVER be stored in plaintext anywhere.

**Scope:** local DB, server DB, sync payloads, logs, test fixtures, backups

**Enforcement:**
- Use XChaCha20Poly1305 for all encryption
- Zeroize plaintext after encryption
- Never log secrets
- Export plaintext requires explicit --dangerous-export-plaintext flag

**Test:** Decrypt ciphertext, verify plaintext not stored on disk

---

[Repeat for all 13 invariants]
```

### ARCHITECTURE.md Structure

```markdown
# Architecture

## Overview

[1 paragraph: what you're building]

## Component Diagram

\`\`\`
[ASCII diagram showing: UI ← Core ← Crypto]
                        ↓
                      Store
\`\`\`

## Data Flow

\`\`\`
User creates vault → Master password → Argon2id
                  ↓
            Vault creation
                  ↓
         Encrypted in SQLite
\`\`\`

## Crate Responsibilities

### porkpie-types
- Domain types (IDs, Items, Timestamps)
- Error types
- Constants

### porkpie-crypto
- Key derivation
- Encryption/decryption
- MAC validation

[... etc for all 10 crates ...]

## Security Boundaries

- **Client boundary:** Everything user-facing (CLI, UI, decrypt)
- **Server boundary:** HTTP API stores encrypted blobs only
- **Never cross:** Server must never see plaintext

## Encryption Boundaries

- **IN:** porkpie-crypto (only place crypto happens)
- **OUT:** All other crates use crypto services
```

### DATA_MODEL.md Structure

```markdown
# Data Model

## Vault

| Field | Type | Notes |
|-------|------|-------|
| id | VaultId (UUID) | Unique identifier |
| created_at | Timestamp | Unix millis |
| master_key_wrapped | Vec<u8> | Encrypted, never decrypted by store |
| sync_revision | u64 | For sync protocol |

## Item (Base)

| Field | Type | Notes |
|-------|------|-------|
| id | ItemId (UUID) | Unique |
| vault_id | VaultId | FK to Vault |
| item_type | ItemType | Login, APIKey, etc. |
| encrypted_data | Vec<u8> | Full item encrypted |
| created_at | Timestamp | |
| updated_at | Timestamp | |

## Item Types

### Login

\`\`\`json
{
  "username": "user@example.com",
  "password": "secret123",      // encrypted
  "url": "https://example.com",
  "notes": "work account"
}
\`\`\`

[... repeat for all 10 types ...]
```

## 🔗 References

- **Security Invariants:** Porkpie Architecture and Coding Plan — Section 6
- **Threat Model:** Porkpie Architecture and Coding Plan — Section 6
- **Data Model:** Porkpie Architecture and Coding Plan — Sections 4-5
- **Crate Details:** Porkpie Architecture and Coding Plan — Section 5

## ✔️ Success Verification

1. **Markdown validity:**
   ```bash
   # Check for broken markdown
   find docs -name "*.md" -type f
   # Open each in markdown viewer, verify readable
   ```

2. **Internal links work:**
   ```bash
   grep -r "\[.*\](" docs/
   # Verify all referenced files exist
   ```

3. **No TODO placeholders:**
   ```bash
   grep -r "TODO\|FIXME\|\[.*\]" docs/
   # Should return empty (no placeholders)
   ```

4. **Consistent with architecture:**
   - Security Invariants match Implementation Plan
   - Data Model matches crate responsibilities
   - Threat Model covers all major scenarios

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
| "Don't know enough about cryptography" | Read Porkpie Architecture and Coding Plan Section 3 (Foundation Stack) + Section 6 (Security Invariants). Copy invariants + explain why each matters. |
| "Can't design the data model" | Use the 10 item types from Task Queue (Task 3). For each type, create fields that a real password manager would need. |
| "Too vague about threat model" | Think: what could go wrong? (password guessing, server breach, etc.). For each, write 1-2 sentence mitigation. |
| "Architecture is complex" | Start simple: draw 3 boxes (UI, Core, Crypto). Add arrows (calls). Add Store below. Add Server to the right. Done. |

## 📌 What Comes Next

**Task 3: Core Types and Error Definitions**

Next agent will implement porkpie-types crate. They'll use these docs as reference for entity definitions, constraints, and error handling patterns.

---

**Status:** Ready for agent assignment
