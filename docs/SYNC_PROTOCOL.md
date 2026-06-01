# Sync Protocol

Porkpie sync is revision-based and ciphertext-only. Clients encrypt and decrypt locally; the server stores vault metadata and item blobs without master passwords or plaintext item fields.

## Types

- `SyncRequest`: `{ vault_id, last_revision }`
- `EncryptedSyncItem`: `{ item_id, item_type, ciphertext, created_at, updated_at, sync_revision }`
- `SyncResponse`: `{ items, new_revision, conflicts }`
- `SyncRegisterRequest`: `{ vault_id, name, salt, master_key_wrapped, created_at }`
- `VaultMetadataResponse`: `{ vault_id, name, salt, master_key_wrapped, created_at, sync_revision }`
- `SyncPushRequest`: `{ vault_id, base_revision, items, merge_strategy }`
- `SyncPushResponse`: `{ accepted, new_revision, conflicts }`
- `ConflictItem`: `{ item_id, local_revision, server_revision, server_data }`

## API Endpoints

All sync and vault endpoints are protected by bearer API key authentication.

| Method | Path | Purpose |
|--------|------|---------|
| `POST` | `/api/v1/sync/register` | Register vault cryptographic metadata |
| `POST` | `/api/v1/sync/begin` | Pull encrypted items changed after revision |
| `POST` | `/api/v1/sync/push` | Push encrypted item changes; returns conflicts |
| `GET`  | `/api/v1/vault/{vault_id}` | Fetch vault metadata (encrypted blobs only) |

## Bidirectional Sync Flow

### First sync (Profile A)

1. **Register vault**: Profile A calls `POST /api/v1/sync/register` with the vault's `id`, `name`, `salt`, `master_key_wrapped`, and `created_at`. The server stores these encrypted blobs as-is. The server never receives the master password or vault key.

2. **Push encrypted items**: Profile A calls `POST /api/v1/sync/push` with:
   - `vault_id`: the vault's UUID
   - `base_revision`: 0 (start of stream)
   - `items`: array of `EncryptedSyncItem` (ciphertext only, produced locally via `vault.encrypt_item`)
   - `merge_strategy`: optional, defaults to `LastWriteWins`

3. The server assigns monotonically increasing revisions to each item, bumps the vault's `sync_revision`, and returns `{ accepted, new_revision, conflicts }`.

### Peer sync (Profile B)

4. **Fetch vault metadata**: Profile B calls `GET /api/v1/vault/{vault_id}` to retrieve `salt`, `master_key_wrapped`, `name`, and `created_at`. Profile B reconstructs a locked `Vault` via `Vault::from_encrypted_metadata(...)`.

5. **Pull encrypted items**: Profile B calls `POST /api/v1/sync/begin` with `{ vault_id, last_revision: 0 }`. The server returns `EncryptedSyncItem[]` and a `new_revision`. Profile B unlocks the vault locally (master password + secret key), then decrypts each server item via `vault.decrypt_item(...)` and inserts them into the in-memory vault.

6. **Push local changes**: Profile B encrypts edited items via `vault.encrypt_item(...)` and pushes them to the server.

### Push with conflict detection

7. When Profile B tries to push an item that Profile A already edited (same `item_id`, the server's revision of that item is > `base_revision`, and the ciphertext differs), the push endpoint:

   - If `merge_strategy` is `PreferLocal` or `PreferRemote`: returns HTTP **409 Conflict** with a `ConflictItem[]` payload. Each conflict carries `item_id`, `local_revision`, `server_revision`, and the server's `ciphertext`.
   - If `merge_strategy` is `LastWriteWins`: the push succeeds (the server accepts whichever revision wins by `updated_at`).

   The CLI preserves conflicts: it stores them in memory, reports the count to the user, and does not silently overwrite.

## Merge Strategies

- `LastWriteWins` (default): compare `updated_at`; accept the later one. Conflicts are still reported in the `SyncResponse`/`SyncOutcome` metadata.
- `PreferLocal`: reject same-item conflicts. The push returns 409 with conflict data.
- `PreferRemote`: reject same-item conflicts. The push returns 409 with conflict data.

## Client-Side Sync Cursor

After each successful sync the client persists a `SyncState` row in its local SQLite database:

```rust
struct SyncState {
    vault_id: VaultId,
    last_synced_revision: Option<u64>,
    last_synced_at: Option<Timestamp>,
}
```

The `last_synced_revision` cursor is used as the `last_revision` parameter on the next `sync/begin` call, so the client only transfers items that changed after the last successful sync.

## CLI Command

```bash
# Full sync (default strategy: last-write-wins)
porkpie sync --server http://127.0.0.1:8000 --api-key <key>

# Explicit strategy
porkpie sync --server http://127.0.0.1:8000 --api-key <key> --strategy prefer-local

# Environment variable overrides
PORKPIE_SYNC_URL=http://127.0.0.1:8000 PORKPIE_API_KEY=<key> porkpie sync
```

The CLI:
1. Loads the unlocked vault and sync cursor from the local store.
2. Registers the vault on the server (idempotent).
3. Pulls encrypted changes from `sync/begin`.
4. Decrypts and merges server items into the in-memory vault.
5. Pushes locally-changed items to the server.
6. Persists the updated sync cursor.

## Security Boundary

- **Sync payloads** contain ciphertext and non-secret metadata only. The server never receives the master password, the local secret key, the vault key, or any plaintext item fields.
- **Plaintext rejection** (Phase 10): The server validates every pushed `ciphertext` blob. If it detects obvious plaintext patterns — JSON structures containing field names like `username`, `password`, `private_key`, `api_key`, `totp`, or `notes` — the push is rejected with HTTP 422 (`validation_error`). Real encrypted ciphertext must be opaque binary data.
- **Vault registration** sends `salt`, `master_key_wrapped`, and timestamps — all ciphertext-adjacent metadata. The server stores these as opaque blobs and never uses them to decrypt.
- **API logs** record audit events (`sync_push`, `vault_register`) with timestamps and vault IDs only. Item payloads and ciphertext are not logged.
- **Server-side zero-knowledge** is maintained: the server's database has no `name` column in items, no username/password columns, and no decryption logic. The test suite asserts that ciphertext rows do not contain fixture plaintext even when the ciphertexts happen to use ASCII placeholder data.