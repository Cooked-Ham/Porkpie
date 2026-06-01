# Product Spec

Porkpie is a local-first password manager for developers, homelab users, and small teams that want self-hostable encrypted storage without handing a hosted service their decrypted vault.

## MVP Scope

- Create, unlock, lock, and delete encrypted local vault data.
- Store all item payloads as XChaCha20Poly1305 ciphertext.
- Support ten domain item types in shared Rust types.
- Provide a CLI for vault lifecycle, item CRUD, encrypted backup import/export, CSV import, and sync entry points.
- Provide Dioxus screens for onboarding, unlock, item list/detail, password generation, import/export, and settings.
- Provide an Axum sync API that stores encrypted blobs only and protects sync routes with API keys.

## Non-Goals

- Hosted accounts, password reset, and server-side vault recovery.
- Plaintext export by default.
- Server-side search or indexing of decrypted item fields.

## Success Criteria

- The workspace builds and tests cleanly.
- CLI and API binaries run locally.
- Docker Compose starts the sync API and health checks pass.
- Security invariants are covered by code structure and tests.
