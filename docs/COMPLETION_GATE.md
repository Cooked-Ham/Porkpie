# Completion Gate

Porkpie cannot be called an MVP until every gate below passes. These gates are the minimum bar before the project can honestly describe itself as a functional password manager.

## Build Gate

- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo build --workspace` passes.
- [ ] `cargo build --workspace --release` passes.

## Security Gate

- [ ] No plaintext secret storage anywhere (local DB, server DB, sync payloads, logs, test fixtures).
- [ ] No raw `Debug` derives on secret-bearing types.
- [ ] No CLI command dumps whole decrypted items by default.
- [ ] `porkpie item list` and `porkpie item get` are redacted by default.
- [ ] Master password is never stored or logged.
- [ ] Memory zeroization on vault lock is verified by tests.
- [ ] No static nonces, no reused nonces, no hardcoded keys.
- [ ] Wrong password fails decryption (tested).
- [ ] Tampered ciphertext fails decryption (tested).
- [ ] Security audit or review has been scheduled or completed.

## CLI Gate

- [ ] `porkpie read <pie-uri>` works for explicit secret reveal.
- [ ] `porkpie write <pie-uri>` works for explicit field writes.
- [ ] `porkpie copy <pie-uri>` works for clipboard-based reveal.
- [ ] `porkpie run --env NAME=<pie-uri> -- <command>` works for env injection.
- [ ] `porkpie item list` returns redacted output by default.
- [ ] `porkpie item get` returns redacted output by default.
- [ ] All `pie://` URIs resolve correctly against vault/item names.

## UI Gate

- [ ] Dioxus UI connects to real vault operations (not static mockups).
- [ ] Unlock screen actually unlocks a vault.
- [ ] Item list loads real encrypted items from the store.
- [ ] Item detail decrypts and displays on demand.
- [ ] Password generator writes results to the vault or clipboard.
- [ ] Import/export triggers real operations.
- [ ] Navigation works between screens (not a single scrollable page).

## Desktop/Web Gate

- [ ] Desktop app launches as a real windowed application.
- [ ] Web app compiles to WASM and serves in a browser.
- [ ] Both connect to the real vault store.

## Sync Gate

- [ ] Sync push and pull both work end-to-end.
- [ ] Conflict resolution is tested with real concurrent changes.
- [ ] Docker Compose deployment starts the API and passes health checks.

## Import/Export Gate

- [ ] CSV import works for all supported item types.
- [ ] Encrypted backup round-trip works (export then import).
- [ ] Third-party importers (if claimed) are tested with real export files.

## Documentation Gate

- [ ] README warns against real-secret use until all gates pass.
- [ ] `docs/STATUS.md` accurately reflects implementation state.
- [ ] No misleading claims of completeness, production readiness, or safety.

## Current Status

**Gates passing: 1 of 9 (Build Gate only).**

Porkpie is a foundational Rust prototype. It is not an MVP.
