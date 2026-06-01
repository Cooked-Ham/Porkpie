# Porkpie Agentic Worklist Bundle

Generated: 2026-06-01T02:12:20Z

This bundle breaks the Porkpie recovery/hardening plan into phase-bound markdown files optimized for coding agents.

## Use Order

1. `PHASE_01_CI_AND_STATUS_HONESTY.md`
2. `PHASE_02_REMOVE_SECURITY_FOOTGUNS.md`
3. `PHASE_03_STRENGTHEN_CRYPTO_MODEL.md`
4. `PHASE_04_PROVE_LOCAL_STORAGE_NO_PLAINTEXT.md`
5. `PHASE_05_FIX_CLI_AROUND_PIE_URIS.md`
6. `PHASE_06_MAKE_UI_REAL_RUST_DIOXUS.md`
7. `PHASE_07_MAKE_DESKTOP_AND_WEB_LAUNCHABLE.md`
8. `PHASE_08_MAKE_SYNC_REAL.md`
9. `PHASE_09_BUILD_SSH_AGENT_FOUNDATION.md`
10. `PHASE_10_HARDEN_API.md`
11. `PHASE_11_IMPORT_EXPORT_SAFETY.md`
12. `PHASE_12_FINAL_HOSTILE_QA.md`

## Recommended Agent Pattern

Assign one phase per agent session.

Do not let agents jump ahead. The whole point is to stop the previous “complete” cosplay and bind each pass to a narrow, testable scope. Tiny miracle, but apparently necessary.

## Global Rules

# Global Porkpie Agent Rules

These rules bind every phase. Do not reinterpret them. Do not “simplify” them into a different stack. Do not helpfully scaffold a TypeScript/Vite/Electron app because some template looked friendly. That is how prototypes become haunted.

## Project Identity

- Project: Porkpie
- Domain: porkpie.love
- CLI binary: `porkpie`
- Canonical secret reference format: `pie://Vault/Item/field`
- Primary language: Rust
- UI framework: Dioxus
- API framework: Axum
- Async runtime: Tokio
- Persistence: SQLx
- Local DB: SQLite
- Deployment: Docker Compose for self-hosting
- Crypto target: Argon2id + XChaCha20Poly1305

## Required Architecture

- Rust-first Cargo workspace.
- Dioxus UI.
- Axum server/API.
- SQLx persistence.
- SQLite local storage.
- Zero-knowledge server.
- Encrypted blobs only on server.
- `pie://` as the stable secret-reference interface.
- CLI, UI, API, sync, store, crypto, and types must remain separated by crate boundaries.

## Forbidden

- No Electron.
- No React frontend as the product foundation.
- No TypeScript frontend as the product foundation.
- No Vite scaffold as the main app.
- No fake crypto.
- No base64 pretending to be encryption.
- No hardcoded keys.
- No static nonces.
- No reused nonces.
- No plaintext secret storage.
- No server-side vault decryption.
- No public default API keys.
- No raw `Debug` derives on secret-bearing types.
- No CLI command that dumps whole decrypted items by default.
- No static UI mockups presented as real functionality.
- No sync implementation that only pushes and pretends pulling does not matter.
- No broad `#[allow(...)]` to fake clean Clippy.
- No weakening tests to pass CI.

## Canonical `pie://` Model

Format:

```text
pie://Vault/Item/field
```

Examples:

```text
pie://Personal/GitHub/password
pie://Homelab/server1/ssh_private_key
pie://Infrastructure/Cloudflare/api_token
pie://Dev/Postgres/connection_string
pie://Personal/HyVee/recovery_codes
```

Rules:

- `pie://` is the stable interface for CLI, env injection, future agents, docs, and integrations.
- Do not implement `porkpie secret read`.
- Use `porkpie read <pie-uri>` for explicit secret reveal.
- Use `porkpie write <pie-uri>` for explicit field writes.
- Use `porkpie copy <pie-uri>` for clipboard-based reveal.
- Use `porkpie run --env NAME=<pie-uri> -- <command>` for env injection.
- `porkpie item list` and `porkpie item get` must be redacted by default.

## Global Validation Command

Run this before reporting a phase complete:

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

## Required Agent Completion Report

Every agent report must include:

- Summary
- Files changed
- Commands run
- Test results
- Security notes
- Remaining limitations
- Next recommended task

