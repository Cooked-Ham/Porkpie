# Porkpie

Local-first, zero-knowledge, self-hostable password manager for developers, homelab users, and small teams.

- **CLI:** porkpie
- **URI Scheme:** pie://
- **Built With:** Rust, Dioxus, Axum, SQLx
- **Storage:** SQLite with encrypted vault metadata and item ciphertext only
- **Sync:** Axum API with bearer-token auth and encrypted blob replication

## Quick Start

```bash
cargo build --workspace
cargo test --workspace
cargo run --bin porkpie -- --help
```

## CLI Basics

```bash
porkpie init
porkpie unlock
porkpie add login
porkpie list
porkpie export
porkpie import porkpie-backup-1780000000000.json.enc
```

CSV imports use the columns `item_type,title,username,password,notes`. Encrypted backups use `.json.enc` files and contain encrypted item blobs only.

## API Server

```bash
docker compose up --build
curl http://localhost:8000/api/v1/health
```

Configure with `DATABASE_URL`, `API_PORT`, and `API_KEY`. Sync routes require `Authorization: Bearer {API_KEY}`.

See [docs/](docs/) for architecture, security invariants, sync protocol, data model, test plan, and roadmap.
