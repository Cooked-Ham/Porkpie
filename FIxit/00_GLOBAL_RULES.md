# Global Binding Rules for Porkpie Hostile-QA Fix Phases

These rules bind every phase in this packet.

## Product Identity

- Project: Porkpie
- Repository: `Cooked-Ham/Porkpie`
- Domain: `porkpie.love`
- CLI binary: `porkpie`
- Canonical secret URI: `pie://Vault/Item/field`
- Primary language: Rust
- UI framework: Dioxus
- API framework: Axum
- Async runtime: Tokio
- Persistence: SQLx
- Local DB: SQLite
- Sync server: zero-knowledge encrypted blob replication

## Absolute Stack Rules

Required:

- Rust-first Cargo workspace.
- Dioxus UI.
- Axum API/server.
- SQLx persistence.
- SQLite local vault storage.
- `pie://` as the canonical field reference format.
- Zero-knowledge sync server.
- Encrypted item blobs only on the server.
- CLI, UI, API, sync, crypto, store, import, and types remain separated by crate boundaries.

Forbidden:

- No Electron.
- No React as the main frontend.
- No TypeScript frontend foundation.
- No Vite scaffold as the product app.
- No fake crypto.
- No base64 pretending to be encryption.
- No hardcoded production secrets.
- No public default API keys.
- No static nonces.
- No reused nonces.
- No plaintext secret storage.
- No server-side vault decryption.
- No raw `Debug` output for secret-bearing data.
- No broad `#[allow(...)]` to fake a clean Clippy run.
- No deleting tests to pass CI.
- No weakening existing tests.
- No documentation claim that is not backed by working code.

## Required Validation Command

Every phase must run this before reporting completion:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

If a phase touches web/WASM behavior, also run:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```

If a phase touches infra, also run:

```bash
docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config
```

## Required Agent Report Format

Every agent must finish with:

```markdown
# Phase Report

## Summary

## Files Changed

## Commands Run

## Test Results

## Security Notes

## Documentation Updated

## Remaining Risks

## Next Recommended Phase
```

## Status Label

Unless every completion gate passes and an external security audit has been completed, the correct label remains:

```text
Porkpie is a foundational Rust prototype. It is not safe for real credentials yet.
```
