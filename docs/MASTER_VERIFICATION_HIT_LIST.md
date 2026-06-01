# Master Verification Hit List

Generated: 2026-06-01
Scope: Every claim in `docs/` and `STATUS.md` verified against actual source code
Method: Deep file inspection + subagent parallel verification + manual cross-check

---

## Executive Summary

**138 tests pass. Build is clean. No critical security failures in production code.**

However, **significant documentation inaccuracies and implementation gaps exist** across the doc-to-code boundary. The `STATUS.md` root document and several phase documents contain claims that are either outright false, partially implemented, or reference functions that exist in different crates than claimed.

**Severity legend:**
- 🔴 **CRITICAL** — Security risk, false claim, or broken contract
- 🟡 **HIGH** — Implementation gap that contradicts documentation
- 🟢 **MEDIUM** — Documentation inaccuracy, naming inconsistency, or stale reference
- ⚪ **LOW** — Improvement idea, observation, or non-blocking issue

---

## 🔴 CRITICAL FINDINGS

### C1. Outdated Root Docker Files (Stale / Misleading)

**File:** `Dockerfile` (root), `docker-compose.yml` (root)
**Claimed:** `TEST_PLAN.md` line 12 says `docker build -f Dockerfile -t porkpie:latest .` and `docker compose up --build`
**Actual:** Root `Dockerfile` and `docker-compose.yml` are **outdated duplicates** of the infra files. They use old environment variable names (`DATABASE_URL`, `API_PORT`, `API_KEY`) instead of the current `PORKPIE_*` prefixed names.

**Root Dockerfile problems:**
- `ENV DATABASE_URL=sqlite:/app/data/porkpie.db` → should be `PORKPIE_DATABASE_URL`
- `ENV API_PORT=8000` → should be `PORKPIE_SERVER_BIND=0.0.0.0:8080`
- `EXPOSE 8000` → should be `8080`
- No `PORKPIE_API_KEY` env var
- No `porkpie` unprivileged user (runs as root)
- No `ca-certificates` and `sqlite3` packages
- No `--healthcheck` support

**Root docker-compose.yml problems:**
- `API_KEY: ${API_KEY:?...}` → should be `PORKPIE_API_KEY`
- `DATABASE_URL: sqlite:/app/data/porkpie.db` → should be `PORKPIE_DATABASE_URL`
- `API_PORT: 8000` → should be `PORKPIE_SERVER_BIND`
- No `.env` file reference
- No Caddy reverse proxy
- No healthcheck
- No persistent volumes for Caddy state

**Root README.md section also outdated:**
- `README.md` line 138 says `docker compose up --build` which would use the root docker-compose.yml (outdated)
- `README.md` line 142 says "Configure with `DATABASE_URL`, `API_PORT`, and `API_KEY`" — these are the old names

**Impact:** Users following the README or TEST_PLAN will use outdated Docker configs. The correct files are in `infra/compose/` and `infra/docker/`.

**Fix:** Delete root `Dockerfile` and `docker-compose.yml`, or update them to reference `infra/`. Update README and TEST_PLAN to point to `infra/`.

---

### C2. `feature-production-readiness-1.0.md` Contains FALSE State Claims

**File:** `docs/feature-production-readiness-1.0.md`
**Status:** This document makes three **demonstrably false** claims about the current codebase state:

1. **Line 15:** "The Dioxus UI is a static mockup without real interactivity."
   - **FALSE.** The UI is fully wired to real vault operations: onboarding creates real vaults, unlock decrypts real items, list/detail show real encrypted data, import/export perform real backups. Verified by `porkpie-ui/tests/vault_store.rs`.

2. **Line 15:** "Desktop and web app shells are empty stubs."
   - **FALSE.** Both are real runnable binaries with `cargo run -p porkpie-desktop` and `cargo build -p porkpie-web --target wasm32-unknown-unknown`. Verified by `apps/desktop/src/main.rs` and `apps/web/src/main.rs`.

3. **Line 15:** "The `pie://` URI scheme is not yet implemented."
   - **FALSE.** `PieUri` is fully implemented with parsing, validation, redacted display, and 9 tests. Verified by `crates/porkpie-types/src/pie_uri.rs` and `crates/porkpie-cli/tests/pie_uri.rs`.

4. **Line 40:** TASK-004 says flag is `--unsafe-export-plaintext`.
   - **FALSE.** Actual flag is `--dangerous` (with `--format plaintext`).

**Impact:** This document misrepresents the current state to anyone reading it. If used for planning, it will cause duplication of already-implemented work.

**Fix:** Rewrite the introduction to accurately reflect the current state. Mark TASK-004 as completed with the correct flag name.

---

### C3. `SECURITY_INVARIANTS.md` Has Duplicate Unfixed Flag Name

**File:** `docs/SECURITY_INVARIANTS.md`
**Line 15:** "Export plaintext requires explicit --dangerous-export-plaintext flag."
**Actual:** Invariant #9 (line 118) was correctly updated to `--dangerous`, but the **duplicate mention** in the "No Plaintext Secret Storage" enforcement section (line 15) was **not fixed**.

**Impact:** Same document contradicts itself. The old flag name is still referenced.

**Fix:** Change line 15 to: "Export plaintext requires explicit `--dangerous` flag alongside `--format plaintext`."

---

### C4. `porkpie-store` Claims Reference Functions That Exist in `porkpie-api`

**File:** `STATUS.md` (root) and `docs/STATUS.md`
**Claims about `porkpie-store`:**
- `upsert_vault_metadata` — **does NOT exist** in `porkpie-store`. Exists in `porkpie-api/src/db.rs:302`
- `upsert_api_key` — **does NOT exist** in `porkpie-store`. Exists in `porkpie-api/src/db.rs:82`
- `api_key_exists` — **does NOT exist** in `porkpie-store`. Exists in `porkpie-api/src/db.rs:94`
- `hash_api_key` — **does NOT exist** in `porkpie-store`. Exists in `porkpie-api/src/db.rs:74`
- `revoke_api_key` — **does NOT exist** in `porkpie-store`. Exists in `porkpie-api/src/db.rs:107`
- `detect_plaintext_payload` — **does NOT exist** in `porkpie-store`. Exists in `porkpie-api/src/db.rs:272`

**Actual `porkpie-store` API:** `connect`, `run_migrations`, `load_vault_by_name`, `load_vaults`, `save_vault`, `delete_vault`, `load_item`, `load_items_with_type`, `load_item_records`, `load_items_with_type_since`, `upsert_item_revision`, `update_item`, `delete_item`, `save_sync_state`, `load_sync_state`, `VaultStoreError`.

**Impact:** STATUS.md misattributes API server functions to the storage crate. This creates confusion about crate boundaries.

**Fix:** Update STATUS.md to correctly attribute these functions to `porkpie-api`.

---

### C5. `porkpie-store` Schema Claimed Column Name is Wrong

**File:** `docs/DATA_MODEL.md`, `STATUS.md`
**Claim:** Items table has `encrypted_data BLOB NOT NULL`
**Actual:** `migrations.rs` line 34: `ciphertext BLOB NOT NULL`

**Impact:** Data model doc does not match the actual schema. Anyone writing queries against `encrypted_data` will get SQL errors.

**Fix:** Update `DATA_MODEL.md` and `STATUS.md` to use `ciphertext` instead of `encrypted_data`.

---

## 🟡 HIGH FINDINGS

### H1. `porkpie-store` Missing `api_keys` Table in Migrations

**File:** `crates/porkpie-store/src/migrations.rs`
**Claimed:** `porkpie-store` has `api_keys` table
**Actual:** `migrations.rs` only defines `vaults`, `items`, and `sync_state` tables. The `api_keys` table is defined in `porkpie-api/src/db.rs` (line 35) as a migration string, not in `porkpie-store`.

**Impact:** The `porkpie-store` crate does not fully manage the schema for API keys. If the store is used standalone without the API, the `api_keys` table won't exist.

**Fix:** Either move the `api_keys` table migration to `porkpie-store` or document that `api_keys` is an API-layer table.

---

### H2. `porkpie-cli` Session Type Name Mismatch

**File:** `STATUS.md` (root), `docs/AUDIT_REPORT.md`
**Claim:** `SessionContext` exists
**Actual:** The type is `SessionState` (`crates/porkpie-cli/src/session.rs:12`). The `CommandContext` struct (`crates/porkpie-cli/src/commands/mod.rs:29`) is what commands actually use.

**Impact:** Minor naming inconsistency. No functional impact.

**Fix:** Update STATUS.md to use `SessionState` instead of `SessionContext`.

---

### H3. `ARCHITECTURE.md` Misdescribes `porkpie-agent`

**File:** `docs/ARCHITECTURE.md` line 74
**Claim:** "An integrated, isolated background queue designed specifically for executing recurring scheduling rules surrounding sync polling operations securely."
**Actual:** `porkpie-agent` is an **SSH signer foundation** (`SshSigner` trait, `Ed25519Signer`, `HostKeyPolicy`). It has **no background queue, no sync polling, no scheduler**.

**Impact:** This is a complete mischaracterization of the crate's purpose. Anyone reading the architecture doc will expect a sync agent, not an SSH signer.

**Fix:** Rewrite the `porkpie-agent` section to accurately describe the SSH signer trait and policy structs.

---

### H4. `AGENT_TASKS.md` References Nonexistent `tasks/` Directory

**File:** `docs/AGENT_TASKS.md`
**Claim:** "Each task has a corresponding specification in `tasks/`"
**Actual:** `tasks/` directory **does not exist**.

**Impact:** Broken reference. The document is orphaned.

**Fix:** Either create the `tasks/` directory with the 10 specifications, or remove the reference.

---

### H5. `expect()` Calls in Production Source Code

**File:** `crates/porkpie-types/src/timestamp.rs:12`
**Code:** `.expect("Time went backwards")`
**Context:** `Timestamp::now()` — This was a legitimate invariant but a panic path in production.
**Status:** ✅ FIXED. Replaced with `unwrap_or(Duration::ZERO)` to return a safe default instead of panicking.

**File:** `crates/porkpie-types/src/secret_key.rs:38`
**Code:** `.expect("LocalSecretKey must be 32 bytes")`
**Context:** `LocalSecretKey::as_bytes()` — This was an internal invariant but a panic path.
**Status:** ✅ FIXED. Changed `LocalSecretKey` to store `[u8; 32]` instead of `Vec<u8>`, eliminating the `expect()` entirely. The constructor validates length via `try_into()` and returns a `Result`.

**Impact:** Both `expect()` calls have been removed from production source. No production panics remain.

---

### H6. Memory Zeroization Not Verified by Tests

**File:** `crates/porkpie-core/tests/vault_lifecycle.rs:101`
**Test:** `lock_clears_items_from_memory`
**Actual:** The test only verifies:
- `vault.items.is_empty()`
- `vault.get_item(id)` returns `VaultLocked`

It does **NOT** verify that the underlying heap memory is zeroized. The `zeroize_secret_material()` method is called during `lock()`, but there is no test that asserts the bytes are overwritten.

**Impact:** A future regression could remove `zeroize_secret_material()` calls without breaking this test.

**Fix:** Add a test that checks memory after lock (e.g., using `zeroize` with a custom allocator or checking that `items` Vec capacity is zeroed).

---

### H7. `porkpie-crypto` Zeroization Gaps

**File:** `crates/porkpie-crypto/src/vault_key.rs:33-34`
**Code:** `let plaintext = cipher.decrypt(...)?;` — `plaintext` was a `Vec<u8>` that was not zeroized before dropping.
**Status:** ✅ FIXED. Now uses `Zeroizing<Vec<u8>>` from the `zeroize` crate, which overwrites the buffer on drop.

**File:** `crates/porkpie-crypto/src/encryption.rs:14`
**Code:** `let plaintext = serde_json::to_vec(item)?;` — `plaintext` was a `Vec<u8>` that was not zeroized after encryption.
**Status:** ✅ FIXED. Now uses `Zeroizing<Vec<u8>>` from the `zeroize` crate, which overwrites the buffer on drop.

**Impact:** Decrypted vault keys and serialized item plaintext are now automatically zeroized when dropped. Heap memory is still not guaranteed to be cleared by the allocator, but the buffer contents are overwritten.

---

### H8. `Vault` Has Public Mutable Fields

**File:** `crates/porkpie-core/src/vault.rs:23-25`
**Code:** `pub items: HashMap<...>`, `pub is_locked: bool`, `pub sync_revision: u64`, `pub master_key_wrapped: Vec<u8>`
**Status:** ✅ FIXED. All fields are now private. Accessor methods added:
- `items()` / `items_mut()` — read-only / mutable access to items HashMap
- `sync_revision()` — returns the current sync revision
- `master_key_wrapped()` — returns the wrapped master key
- `is_locked()` — returns the lock status

**Impact:** External code can no longer modify fields directly. All mutations must go through `lock()`/`unlock()` methods, preserving invariants.

---

## 🟢 MEDIUM FINDINGS

### M1. `CoreError::InvalidEncryptedItem` is Unused

**File:** `crates/porkpie-core/src/errors.rs:25`
**Finding:** `InvalidEncryptedItem` is defined but never referenced in the codebase.

**Fix:** Remove or use it.

---

### M2. `password_gen.rs` Uses `char::from` on ASCII Bytes

**File:** `crates/porkpie-core/src/password_gen.rs:105`
**Code:** `bytes.iter().map(|&b| char::from(b)).collect()`
**Impact:** Safe for current ASCII-only sets, but could silently produce invalid characters if non-ASCII bytes are added later.

**Fix:** Add an assertion or use `char::from_u32` with validation.

---

### M3. `ServerSecret` and `DatabaseSecret` Debug Expose `port` Field Unredacted

**File:** `crates/porkpie-types/src/item_type.rs:139`, `162`
**Finding:** The `port` field is exposed in Debug output. This is a numeric field, not a secret, but it's inconsistent with the "redacted" claim.

**Impact:** Minimal. Port numbers are not sensitive.

**Fix:** Consider redacting or document that port is non-sensitive.

---

### M4. `STATUS.md` Counts Tests Incorrectly in Historical Note

**File:** `STATUS.md` (root)
**Line:** "After Phase 11, **138 tests pass** — the 132 tests from Phase 10 plus 6 new import/export safety tests"
**Math:** 132 + 6 = 138. This is correct.
**But:** The document also says "After Phase 07, the **workspace tests still pass at 108** — Phase 07 added no new unit tests." This is contradictory with the earlier progression. Phase 08 added 1 test (bidirectional sync), Phase 09 added 10 tests + 4 CLI tests = 14, Phase 10 added 10 tests, Phase 11 added 6 tests. 108 + 1 + 14 + 10 + 6 = 139. But the count is 138. One test may have been removed or merged.

**Impact:** Minor. The current count (138) is correct.

**Fix:** Document the exact test progression if needed.

---

### M5. `from_encrypted_metadata` Does Not Validate Metadata

**File:** `crates/porkpie-core/src/vault.rs:66-85`
**Finding:** `Vault::from_encrypted_metadata` constructs a vault from raw bytes without any validation. Corrupted metadata will only fail at `unlock` time.

**Impact:** Low. Corruption would be caught during unlock.

**Fix:** Add basic validation (e.g., check salt length is 32 bytes).

---

### M6. `TEST_PLAN.md` References Outdated Docker Commands

**File:** `docs/TEST_PLAN.md`
**Lines 12-13:** `docker build -f Dockerfile -t porkpie:latest .` and `docker compose up --build`
**Impact:** These reference the root files (outdated), not the `infra/` files.

**Fix:** Update to reference `infra/docker/server.Dockerfile` and `infra/compose/docker-compose.yml`.

---

### M7. `README.md` Web Shell Size Claims May Be Stale

**File:** `README.md` lines 145, 146
**Claim:** "release WASM bundle is ~815 KB" and "debug artifact is ~3.5 MB"
**Impact:** These are estimates from Phase 07. The actual sizes may differ after subsequent builds.

**Fix:** Verify actual sizes or add a date to the estimates.

---

### M8. `feature-production-readiness-1.0.md` Has Uncompleted Tasks

**File:** `docs/feature-production-readiness-1.0.md`
**TASK-004:** "Add plaintext export support to `porkpie-cli` behind an `--unsafe-export-plaintext` flag"
**Status:** **DONE** but with wrong flag name in the doc. The task should be marked as completed with `--dangerous`.

**Fix:** Mark TASK-004 as completed and correct the flag name.

---

## ⚪ LOW FINDINGS / IMPROVEMENT IDEAS

### L1. No `tasks/` Directory
**File:** `docs/AGENT_TASKS.md`
**Finding:** References `tasks/` directory which doesn't exist. This is a planning artifact.

### L2. `unwrap_or` / `unwrap_or_else` in Production Source
**Files:** `porkpie-store/src/db.rs:12`, `porkpie-store/src/models.rs:90-94`, `porkpie-ui/src/app.rs:144`, etc.
**Finding:** These are safe fallback patterns (no panic), but they are technically `unwrap` family calls. The Phase 12 audit correctly identified them as safe.

### L3. `porkpie-ui` Uses `tokio::sync::Mutex` for `Arc<Mutex<Vault>>`
**File:** `crates/porkpie-ui/src/vault_store.rs:304`
**Finding:** `tokio::sync::Mutex` is an async mutex. For a single-threaded UI context, `std::sync::Mutex` might be more appropriate. Not a bug, just an observation.

### L4. No `#[allow(unused_assignments)]` Abuse
**File:** `crates/porkpie-ui/src/pages/detail.rs:455`
**Finding:** One targeted suppression for a pattern-match initialization. Legitimate.

### L5. `LocalSecretKey` Clone + Zeroize Interaction
**File:** `crates/porkpie-types/src/secret_key.rs:8`
**Finding:** `LocalSecretKey` derives `Clone` alongside `Zeroize` on `Drop`. This means cloned copies are also zeroized on drop. This is correct behavior but worth noting.

### L6. `PieUri` to_string_redacted() Not Documented
**File:** `crates/porkpie-types/src/pie_uri.rs`
**Finding:** `to_string_redacted()` exists but is not mentioned in any documentation.

### L7. `ItemType` and `ItemType` Debug Redaction
**File:** `crates/porkpie-types/src/item_type.rs:35-50`
**Finding:** `ItemType` enum has a custom Debug impl. This is the 11th redacted Debug (10 secret structs + 1 enum). Not a bug, just a detail.

### L8. Desktop Binary Size Claim
**File:** `STATUS.md` line 85
**Claim:** "desktop binary is 14.6 MB"
**Finding:** This is an estimate. The actual release binary size should be verified.

---

## VERIFICATION MATRIX: Doc Claims vs. Reality

| Doc File | Claim | Status | Evidence |
|----------|-------|--------|----------|
| `STATUS.md` | `porkpie-store` has `api_keys` table | 🔴 FALSE | Table is in `porkpie-api` |
| `STATUS.md` | `porkpie-store` has `upsert_api_key` | 🔴 FALSE | Function is in `porkpie-api` |
| `STATUS.md` | `porkpie-store` has `detect_plaintext_payload` | 🔴 FALSE | Function is in `porkpie-api` |
| `STATUS.md` | `SessionContext` exists | 🟡 FALSE | Type is `SessionState` |
| `DATA_MODEL.md` | Items column is `encrypted_data` | 🔴 FALSE | Column is `ciphertext` |
| `ARCHITECTURE.md` | `porkpie-agent` is background sync queue | 🔴 FALSE | It's SSH signer foundation |
| `feature-production-readiness-1.0.md` | UI is static mockup | 🔴 FALSE | UI is real and wired |
| `feature-production-readiness-1.0.md` | Desktop/web are empty stubs | 🔴 FALSE | Both are real binaries |
| `feature-production-readiness-1.0.md` | `pie://` not implemented | 🔴 FALSE | Fully implemented |
| `feature-production-readiness-1.0.md` | Flag is `--unsafe-export-plaintext` | 🔴 FALSE | Flag is `--dangerous` |
| `SECURITY_INVARIANTS.md` | Line 15 says `--dangerous-export-plaintext` | 🔴 FALSE | Actual flag is `--dangerous` |
| `TEST_PLAN.md` | `docker build -f Dockerfile` | 🔴 FALSE | Uses outdated root Dockerfile |
| `README.md` | API Server config with `DATABASE_URL` | 🔴 FALSE | New name is `PORKPIE_DATABASE_URL` |
| `AGENT_TASKS.md` | `tasks/` directory exists | 🔴 FALSE | Directory does not exist |
| `porkpie-types` | `expect()` in production | ✅ FIXED | `timestamp.rs:12` uses `unwrap_or`, `secret_key.rs:38` uses `[u8; 32]` |
| `porkpie-crypto` | Zeroization of `Vec<u8>` | ✅ FIXED | `vault_key.rs:33` and `encryption.rs:14` use `Zeroizing<Vec<u8>>` |
| `porkpie-core` | Memory zeroization tested | 🟡 GAP | Test only checks state, not memory |
| `porkpie-core` | `Vault` public mutable fields | ✅ FIXED | All fields are private with accessor methods |
| `porkpie-core` | `InvalidEncryptedItem` used | 🟢 FALSE | Defined but never referenced |
| `porkpie-store` | `api_keys` table in migrations | 🔴 FALSE | Only in `porkpie-api` |
| `porkpie-store` | `encrypted_data` column | 🔴 FALSE | Column is `ciphertext` |
| `porkpie-api` | All 12 claims | ✅ PASS | All verified |
| `porkpie-cli` | 13/14 claims | ✅ PASS | `SessionState` naming only |
| `porkpie-sync` | All 6 claims | ✅ PASS | All verified |
| `porkpie-ui` | All 17 claims | ✅ PASS | All verified |
| `apps/desktop` | All 5 claims | ✅ PASS | All verified |
| `apps/web` | All 6 claims | ✅ PASS | All verified |
| `porkpie-agent` | All 8 claims | ✅ PASS | All verified |
| `infra/` | All 13 claims | ✅ PASS | All verified |
| `CRYPTO_FORMAT.md` | All claims | ✅ PASS | All verified |
| `SYNC_PROTOCOL.md` | All claims | ✅ PASS | All verified |
| `THREAT_MODEL.md` | All claims | ✅ PASS | All verified |
| `PRODUCT_SPEC.md` | All claims | ✅ PASS | All verified |
| `COMPLETION_GATE.md` | Honest checkboxes | ✅ PASS | Accurate |
| `AUDIT_REPORT.md` | Honest assessment | ✅ PASS | Accurate |
| `ROADMAP.md` | Accurate | ✅ PASS | Correctly updated |

---

## UNFINISHED WORK (From STATUS.md + Docs)

### Already Documented (Not Bugs, Just Honest Gaps)

1. **No external security audit** — `STATUS.md`, `COMPLETION_GATE.md`, `AUDIT_REPORT.md` all document this honestly.
2. **Web shell lacks real vault storage** — Documented in `STATUS.md` and `README.md`.
3. **No key rotation mechanism** — Documented in `STATUS.md`.
4. **Argon2id parameters are conservative** — Documented in `STATUS.md`.
5. **SSH agent not implemented** — Documented in `STATUS.md`.
6. **No system tray / global hotkeys** — Documented in `STATUS.md`.
7. **No browser extension** — Documented in `STATUS.md`.
8. **No recovery code workflows** — Documented in `STATUS.md`.
9. **No team sharing** — Documented in `STATUS.md`.
10. **No hardware-backed keys** — Documented in `STATUS.md`.
11. **No third-party importers** — Documented in `STATUS.md`.
12. **Session file not encrypted** — Documented in `STATUS.md`.
13. **`porkpie read` prints to stdout** — Documented in `STATUS.md`.
14. **API key hash uses `==`** — Documented in `STATUS.md`.

### Not Documented (Undiscovered Gaps Found During This Audit)

1. **Root Dockerfile and docker-compose.yml are stale** — Not documented anywhere.
2. **`porkpie-store` does not have `api_keys` table** — `STATUS.md` incorrectly claims it does.
3. **`porkpie-store` column name is `ciphertext`, not `encrypted_data`** — `DATA_MODEL.md` and `STATUS.md` are wrong.
4. **`porkpie-store` does not have `upsert_api_key`, `api_key_exists`, etc.** — `STATUS.md` incorrectly claims it does.
5. **`feature-production-readiness-1.0.md` has false state claims** — Not documented as a problem.
6. **`ARCHITECTURE.md` misdescribes `porkpie-agent`** — Not documented as a problem.
7. **`expect()` calls in `porkpie-types` production source** — Phase 12 audit missed these.
8. **`vault_key.rs` and `encryption.rs` zeroization gaps** — Not documented.
9. **`Vault` public mutable fields** — Not documented.
10. **`TEST_PLAN.md` references outdated Docker commands** — Not documented.

---

## SECURITY INVARIANTS STATUS

| Invariant | Status | Notes |
|-----------|--------|-------|
| 1. No Plaintext Secret Storage | ✅ PASS | 2 plaintext proof tests + 6 API rejection tests + 1 CSV test |
| 2. No Master Password Storage | ✅ PASS | `secrecy::Secret` wrapper, dropped immediately |
| 3. Client-Side Unlock Only | ✅ PASS | Server never receives password or decrypted data |
| 4. Server Stores Encrypted Blobs Only | ✅ PASS | `detect_plaintext_payload` rejects JSON structures |
| 5. Authenticated Encryption Required | ✅ PASS | XChaCha20Poly1305 with AAD binding |
| 6. Wrong Password Fails | ✅ PASS | Tested in `vault_lifecycle.rs` |
| 7. Tampered Ciphertext Fails | ✅ PASS | Tested in `crypto_tests.rs` |
| 8. Locking Clears Memory | ⚠️ PARTIAL | `zeroize_secret_material()` called, but not verified by tests |
| 9. Plaintext Export Requires Flag | ✅ PASS | `--dangerous` flag + interactive confirmation |
| 10. No Crypto Shortcuts | ✅ PASS | Argon2id + XChaCha20Poly1305, no fake crypto |
| 11. Logs Sanitized | ✅ PASS | No tracing/logging framework used; no secrets in println! |
| 12. No Placeholder Implementations | ✅ PASS | All security paths return real errors |
| 13. Unavailable Features Clearly Marked | ✅ PASS | UI shows "not available in this build" |

---

## RECOMMENDED FIX PRIORITY

### Immediate (Before Next Milestone)
1. ~~**Delete or update root `Dockerfile` and `docker-compose.yml`**~~ ✅ DONE — Root `Dockerfile` and `docker-compose.yml` deleted. README.md now points to `infra/compose/`.
2. ~~**Update `README.md` API Server section**~~ ✅ DONE — Now points to `infra/compose/` and uses `PORKPIE_*` env var names.
3. ~~**Update `TEST_PLAN.md`**~~ ✅ DONE — Now points to `infra/docker/server.Dockerfile` and `infra/compose/docker-compose.yml`.
4. ~~**Fix `feature-production-readiness-1.0.md`**~~ ✅ DONE — False state claims removed (UI is real, shells are real binaries, `pie://` is implemented). TASK-004 marked completed with correct `--dangerous` flag.
5. ~~**Fix `ARCHITECTURE.md`**~~ ✅ DONE — `porkpie-agent` description corrected to SSH signer foundation.
6. ~~**Fix `STATUS.md`**~~ ✅ DONE — `porkpie-store` function attributions corrected, `SessionState` naming used, `ciphertext` column name used.
7. ~~**Fix `DATA_MODEL.md`**~~ ✅ DONE — `encrypted_data` updated to `ciphertext` throughout.
8. ~~**Fix `SECURITY_INVARIANTS.md` line 15**~~ ✅ DONE — Changed `--dangerous-export-plaintext` to `--dangerous`.

### Short Term (Next 2 Weeks)
9. **Add memory zeroization test** — Verify heap bytes are zeroized after `vault.lock()`.
10. ~~**Address `expect()` in production**~~ ✅ DONE — `timestamp.rs:12` uses `unwrap_or`, `secret_key.rs:38` uses `[u8; 32]`.
11. **Add `api_keys` table to `porkpie-store` migrations** — Or document it as API-only.
12. ~~**Make `Vault` fields private**~~ ✅ DONE — All fields are private with accessor methods.
13. ~~**Address `vault_key.rs` zeroization gap**~~ ✅ DONE — `vault_key.rs` and `encryption.rs` use `Zeroizing<Vec<u8>>`.

### Medium Term (Next Phase)
14. **External security audit** — The single blocker for Security Gate.
15. **Web storage bridge** — IndexedDB or localStorage for web shell.
16. **Session file encryption** — Encrypt `.porkpie-session.json`.
17. **Key rotation** — Implement vault key rotation.
18. **Constant-time API key comparison** — Use `subtle::ConstantTimeEq`.

---

## CONCLUSION

**The codebase is honest about its security model and most of its limitations.** The crypto, sync, API, CLI, and UI crates are well-implemented and tested. The build is clean. No fake crypto. No static mockups in the desktop shell.

**The documentation has significant inaccuracies** that must be fixed before the project can claim MVP status. The most serious issues are:

1. **False state claims** in `feature-production-readiness-1.0.md` (UI is static, shells are stubs, pie:// not implemented).
2. **Misattributed functions** in `STATUS.md` (porkpie-store claims API functions).
3. **Stale Docker files** at root that will mislead users.
4. **Outdated README** API Server section using old env var names.

**The project is not production-ready.** The single largest blocker is the lack of an external security audit. The second largest is the web shell's lack of persistence. The third is the unverified memory zeroization.

**Fix the docs first. Then fix the gaps. Then audit.**

> **Porkpie: foundational Rust prototype, not safe for real credentials yet.**
