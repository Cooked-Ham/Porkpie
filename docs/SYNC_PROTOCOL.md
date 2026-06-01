# Sync Protocol

Porkpie sync is revision-based and ciphertext-only. Clients encrypt and decrypt locally; the server stores vault metadata and item blobs without master passwords or plaintext item fields.

## Types

- `SyncRequest`: `{ vault_id, last_revision }`
- `EncryptedSyncItem`: `{ item_id, item_type, ciphertext, created_at, updated_at, sync_revision }`
- `SyncResponse`: `{ items, new_revision, conflicts }`
- `ConflictItem`: `{ item_id, local_revision, server_revision, server_data }`

## Flow

1. Client calls `POST /api/v1/sync/begin` with its last synced revision.
2. Server validates the bearer API key and returns encrypted items with `sync_revision > last_revision`.
3. Client merges items locally using the configured merge strategy.
4. Client calls `POST /api/v1/sync/push` with encrypted changed items and a base revision.
5. Server detects server-side changes after the base revision. Conflicts return HTTP `409`.

## Merge Strategies

- `LastWriteWins`: compare `updated_at`, preserving conflict metadata.
- `PreferLocal`: reject same-item conflicts with conflict data.
- `PreferRemote`: reject same-item conflicts with conflict data.

## Security Boundary

Sync payloads contain ciphertext and non-secret metadata only. The API does not link against vault unlock logic and never receives the master password.
