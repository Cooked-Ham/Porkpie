# Test Plan

## Required Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
cargo build --workspace
cargo build --workspace --release
target/debug/porkpie --version
docker build -f infra/docker/server.Dockerfile -t porkpie:latest .
cd infra/compose && docker compose -f docker-compose.yml up --build
```

## Coverage Areas

- Domain types serialize and validate correctly.
- Crypto tests verify key derivation, encrypt/decrypt round trips, wrong-password behavior, and tamper detection.
- Core tests verify vault lifecycle, lock/unlock, item CRUD, and password generation.
- Store tests verify SQLite migrations, constraints, encrypted-only item persistence, and concurrent access.
- CLI tests verify argument parsing and session behavior.
- UI tests verify state filtering, navigation state, timeout behavior, and form validation.
- Sync/API tests verify serialization, conflict handling, auth failures, health/status routes, and encrypted-only persistence.
- Import tests verify CSV validation, encrypted backup round trips, duplicate handling, and wrong-password rejection.
