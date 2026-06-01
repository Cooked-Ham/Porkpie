# Porkpie Hostile-QA Agent Phase Packet

Generated: 2026-06-01T09:03:48Z

This packet converts the hostile QA findings into scoped agent phases.

The previous claim that "everything is fixed except external audit" is false. The repo is much improved, but several security, docs, and UX blockers remain. This packet exists so coding agents can fix them without wandering off and birthing Electron in a broom closet.

## Phase Order

1. `PHASE_01_CONFIG_AND_SECRET_ARTIFACTS.md`
2. `PHASE_02_DEBUG_REDACTION_AND_SECRET_STATE.md`
3. `PHASE_03_CLI_SECRET_INPUT_HARDENING.md`
4. `PHASE_04_WEB_STORAGE_AND_DOC_TRUTH.md`
5. `PHASE_05_SERVER_ITEM_ID_INTEGRITY.md`
6. `PHASE_06_SYNC_CONFLICT_AND_STRATEGY_SAFETY.md`
7. `PHASE_07_API_CORS_AND_PAYLOAD_HARDENING.md`
8. `PHASE_08_DOCS_AND_README_TRUTH_PASS.md`
9. `PHASE_09_QA_TEST_EXPANSION.md`
10. `PHASE_10_FINAL_HOSTILE_REAUDIT.md`

## Main Findings This Packet Fixes

- `.env.example` says placeholder API keys are rejected, but config accepts them.
- `.gitignore` does not ignore generated recovery kits, sessions, DB files, backups, or plaintext exports.
- `RecoveryKit` derives raw `Debug` while containing the local secret key.
- Password generator state can expose generated passwords through `Debug`.
- CLI secret prompts use visible input for many secret fields.
- WASM/web storage docs and code contradict each other.
- Server item primary key is globally keyed by `id`, not `(vault_id, id)`.
- Unknown sync strategy silently falls back to LastWriteWins.
- API uses permissive CORS.
- Plaintext-payload rejection is a heuristic but docs imply stronger guarantees.
- README CLI examples are stale.
- Audit docs contradict each other on test counts, web storage, and resolved findings.

## Global Rules

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

