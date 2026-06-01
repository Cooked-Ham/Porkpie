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
