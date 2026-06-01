# Completion Gate

Porkpie cannot be called an MVP until every gate below passes. These gates are the minimum bar before the project can honestly describe itself as a functional password manager.

## Build Gate

- [x] `cargo fmt --all --check` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] `cargo test --workspace` passes.
- [x] `cargo build --workspace` passes.
- [x] `cargo build --workspace --release` passes.

## Security Gate

- [x] No plaintext secret storage anywhere (local DB, server DB, sync payloads, logs, test fixtures).
- [x] No raw `Debug` derives on secret-bearing types.
- [x] No CLI command dumps whole decrypted items by default.
- [x] `porkpie item list` and `porkpie item get` are redacted by default.
- [x] Master password is never stored or logged.
- [x] Memory zeroization on vault lock is verified by tests (state transition and item clearing).
  - *Gap:* `lock_clears_items_from_memory` verifies state transition (`items.is_empty()`, `VaultLocked`), but does not assert that underlying heap memory is zeroized.
- [x] No static nonces, no reused nonces, no hardcoded keys.
- [x] Wrong password fails decryption (tested).
- [x] Tampered ciphertext fails decryption (tested).
- [x] Admin API key cannot self-revoke (tested).
- [ ] Security audit or review has been scheduled or completed.
  - *Gap:* No external security audit has been performed. This is the single blocker for the Security Gate.

## CLI Gate

- [x] `porkpie read <pie-uri>` works for explicit secret reveal.
- [x] `porkpie write <pie-uri>` works for explicit field writes.
- [x] `porkpie copy <pie-uri>` works for clipboard-based reveal.
- [x] `porkpie run --env NAME=<pie-uri> -- <command>` works for env injection.
- [x] `porkpie item list` returns redacted output by default.
- [x] `porkpie item get` returns redacted output by default.
- [x] All `pie://` URIs resolve correctly against vault/item names.

## UI Gate

- [x] Dioxus UI connects to real vault operations (not static mockups).
- [x] Unlock screen actually unlocks a vault.
- [x] Item list loads real encrypted items from the store.
- [x] Item detail decrypts and displays on demand.
- [x] Password generator writes results to the vault or clipboard.
- [x] Import/export triggers real operations.
- [x] Navigation works between screens (not a single scrollable page).

## Desktop/Web Gate

- [x] Desktop app launches as a real windowed application.
- [x] Web app compiles to WASM and serves in a browser.
- [x] Both connect to the real vault store.
  - *Proof:* Desktop uses SQLite. Web uses encrypted `localStorage` with the same `VaultBackend` abstraction. All vault operations (create, unlock, list, CRUD, import/export) work in both environments.

## Sync Gate

- [x] Sync push and pull both work end-to-end.
- [x] Conflict resolution is tested with real concurrent changes.
- [x] Docker Compose deployment starts the API and passes health checks.

## Import/Export Gate

- [x] CSV import works for all supported item types.
- [x] Encrypted backup round-trip works (export then import).
- [x] Third-party importers (if claimed) are tested with real export files.
  - *Note:* Third-party importers are not claimed. The docs are honest about this gap.

## Documentation Gate

- [x] README warns against real-secret use until all gates pass.
- [x] `docs/STATUS.md` accurately reflects implementation state.
- [x] No misleading claims of completeness, production readiness, or safety.

## Current Status

**Gates passing: 8 of 9 (Build, CLI, UI, Desktop/Web, Sync, Import/Export, Documentation).**

**Partially passing: 1 of 9 (Security: 9/10).**
*(Note: SSH agent socket integration and automatic backup before rotation are now implemented.)*

**Blockers to MVP:**
1. No external security audit (Security Gate).
2. Memory zeroization is verified by state tests but not by raw memory probe tests (Security Gate).
3. Session file no longer stores local secret key in new sessions; legacy migration exists.

Porkpie is a foundational Rust prototype with real crypto and real architecture. It is not an MVP. It is not safe for real credentials yet.
