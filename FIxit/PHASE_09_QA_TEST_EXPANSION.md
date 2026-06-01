# Phase 09: QA Test Expansion

## Binding

You are bound to Phase 09 only.

Your job is to add missing tests around the hostile-QA fixes. Do not add product features unless the test exposes a necessary bugfix. Do not remove existing tests. Do not make tests shallow checkbox decorations.

## Goal

Add regression tests proving the fixes from Phases 01-08 stay fixed.

## Required Test Areas

### Config and artifact safety

- placeholder API keys rejected
- short API keys rejected
- valid API key accepted
- Config Debug redacts API key

### Debug redaction

- RecoveryKit Debug redacts local secret key
- PasswordGeneratorState Debug redacts generated password
- PlaintextExport Debug is absent or redacted if it exists

### CLI parsing and secret-safe modes

- `write <uri> --stdin`
- `write <uri> --prompt`
- invalid conflicting args fail
- invalid sync strategy fails
- valid sync strategies parse
- help text warns about literal secret values

### Server integrity

- same item ID in two vaults does not collide
- pull by vault only returns that vault's item
- upsert cannot mutate a different vault's item
- conflict behavior remains vault-scoped

### API CORS

- default CORS is not wildcard permissive
- configured origin allowed
- unconfigured origin denied
- invalid origin config rejected

### Web storage truth

Depending on Phase 04:

Path A:

- localStorage backend creates vault
- unlock loads vault
- item CRUD roundtrip works
- lock clears decrypted state

Path B:

- web build returns unavailable for data-bearing operations
- docs say unavailable

### Documentation consistency

Add a lightweight script or test if practical:

- README does not contain stale `porkpie list`
- README does not claim web behavior opposite of configured path
- audit report does not say known-fixed issues remain unfixed

## Allowed Files

- test files in each crate
- small production fixes revealed by tests
- `scripts/qa-docs-check.*` if useful
- docs test plan

## Forbidden

- Do not hardcode tests to pass without checking behavior.
- Do not use brittle absolute paths.
- Do not require network access.
- Do not require real secrets.
- Do not skip tests silently.

## Acceptance Criteria

- New regression tests cover all high-priority hostile-QA findings.
- Tests fail if the old bugs are reintroduced.
- Global validation command passes.
- WASM build passes if web behavior touched.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

If web touched:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```
