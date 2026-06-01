# Phase 05: Server Item ID Integrity

## Binding

You are bound to Phase 05 only.

Your job is to fix server-side item identity so one vault cannot collide with or mutate another vault's item row. Do not change cryptography. Do not change UI. Do not redesign sync beyond what is required.

## Goal

Change server item uniqueness from global `id` to vault-scoped `(vault_id, id)`.

## Source Findings

The server `items` table uses:

```sql
id TEXT PRIMARY KEY NOT NULL
```

and `upsert_item()` uses `ON CONFLICT(id) DO UPDATE`.

Because item IDs are client-supplied, a malicious or buggy client can reuse an item ID across vaults. With a global primary key, this risks cross-vault integrity bugs.

## Allowed Files

- `crates/porkpie-api/src/db.rs`
- `crates/porkpie-api/tests/**`
- `crates/porkpie-sync/**` only if needed for tests or request models
- `docs/SYNC_PROTOCOL.md`
- `docs/DATA_MODEL.md`
- `docs/SECURITY_INVARIANTS.md`

## Forbidden

- Do not decrypt server-side.
- Do not make item IDs server-generated unless the whole sync protocol is updated and tested.
- Do not remove conflict detection.
- Do not drop data without migration notes.
- Do not silently ignore vault ID in item queries.

## Tasks

### 1. Update schema

For new DBs, make item identity vault-scoped.

Preferred schema:

```sql
CREATE TABLE IF NOT EXISTS items (
    id TEXT NOT NULL,
    vault_id TEXT NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    ciphertext BLOB NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    sync_revision INTEGER NOT NULL DEFAULT 0,
    created_at_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (vault_id, id)
);
```

If SQLite migration compatibility is needed, add a migration path or document fresh-prototype reset limitations.

### 2. Update queries

Every item query must include both `vault_id` and `id` where applicable:

- `load_item`
- `upsert_item`
- conflict detection lookup
- item delete if present
- any test helpers

`ON CONFLICT` must use:

```sql
ON CONFLICT(vault_id, id) DO UPDATE
```

### 3. Add malicious collision test

Add an integration test:

1. Create Vault A and Vault B.
2. Register both.
3. Push item with same `item_id` to Vault A.
4. Push item with same `item_id` to Vault B.
5. Assert both rows exist independently.
6. Assert pulling Vault A returns only Vault A ciphertext.
7. Assert pulling Vault B returns only Vault B ciphertext.
8. Assert neither vault's sync revision corrupts the other.

### 4. Update docs

Document:

- item IDs are unique within a vault
- server storage key is `(vault_id, item_id)`
- conflict detection is vault-scoped

## Acceptance Criteria

- Server item rows are vault-scoped.
- Malicious same-ID cross-vault test passes.
- Existing bidirectional sync test still passes.
- Global validation command passes.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```
