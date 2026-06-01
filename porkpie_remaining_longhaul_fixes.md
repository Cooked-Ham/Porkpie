# Porkpie Remaining Long-Haul Security Fixes

Generated: 2026-06-01T13:49:11+00:00

This worklist is based on a hostile source inspection after the latest "done" claim following commits `2686915` and `f071c71`.

The latest fixes are real and several previous blockers are closed, but the long-haul security work is **not fully done**. Do not add new features before fixing the remaining correctness/security issues below.

---

## Global Binding Rules

Repository: `Cooked-Ham/Porkpie`

Hard rules:

1. Do not introduce Electron.
2. Do not introduce React as the app frontend.
3. Do not introduce TypeScript/Vite as the product UI foundation.
4. Do not weaken crypto.
5. Do not store plaintext secrets.
6. Do not log secrets.
7. Do not silently fall back to weak secret storage.
8. Do not leave scaffold commands described as implemented.
9. Do not suppress Clippy broadly.
10. Do not delete tests to make CI green.
11. Do not claim real-secret safety.
12. Do not claim external audit has happened.

Required validation:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

If web/Dioxus files are touched:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```

If API/infra is touched:

```bash
docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config
```

---

# Task 1: Fix `rotate_local_secret` Keychain Deletion Bug

## Severity

Critical.

## Problem

`rotate_local_secret` stores the new local secret key in the OS keychain, updates the DB wrapped key, then calls:

```rust
s.delete_local_secret_key(&vault_id)
```

But the `SecretStore` keychain entry is keyed only by `vault_id`, so deleting by the same vault ID deletes the entry that now contains the **new** local secret key.

This can leave the vault rewrapped to require the new local secret, while the keychain no longer contains it. The user may still have the printed recovery kit, but session/keychain behavior is broken.

## Required Fix

Change local-secret rotation to avoid deleting the newly stored key.

Preferred safe sequence:

1. Load and verify current vault.
2. Generate new local secret key and recovery kit.
3. Show/write recovery kit.
4. Require explicit confirmation that the recovery kit was saved.
5. Compute new wrapped vault key.
6. Store the new local secret key in keychain.
7. Verify the keychain can load the new key and it matches.
8. Update DB wrapped key.
9. Save session using `SessionState::unlocked(vault_id)`.
10. Do **not** delete the keychain entry by vault ID after storing the new key, because that is the same slot.

If keychain storage fails, abort before updating DB unless the user explicitly chooses a no-store mode.

## Required Tests

Use `FakeKeychain` or an injected `SecretStore`.

Tests must prove:

- after rotation, fake keychain contains the **new** local secret key,
- after rotation, fake keychain does not contain the old local secret key,
- no delete-after-store removes the new key,
- keychain write failure aborts DB update,
- old local secret fails after rotation,
- new local secret succeeds after rotation,
- recovery kit local secret matches the new key.

## Acceptance Criteria

- No `delete_local_secret_key(&vault_id)` is called after storing a new key into the same vault ID slot.
- Rotation cannot silently remove the newly stored keychain secret.
- Tests cover the regression.
- Global validation passes.

---

# Task 2: Add KDF Parameters to Sync Server Vault Metadata

## Severity

Critical for multi-device sync after KDF upgrade.

## Problem

Local vault metadata now stores KDF parameters and `Vault::unlock()` uses `self.kdf_params`. Good.

But the sync API still registers and returns vault metadata without KDF parameters:

```rust
SyncRegisterRequest {
    vault_id,
    name,
    salt,
    master_key_wrapped,
    created_at,
}
```

and `VaultMetadataResponse` contains no KDF params.

The API server `vaults` table also lacks KDF columns. This means a vault upgraded to `hardened`/`paranoid` KDF can sync metadata that another device reconstructs with default KDF params, causing unlock failure or broken restore-from-sync behavior.

## Required Fix

Add KDF params to server-side sync metadata.

### API model changes

Update:

```text
crates/porkpie-api/src/models.rs
```

Add fields:

```rust
pub kdf_time_cost: u32,
pub kdf_mem_cost: u32,
pub kdf_parallelism: u32,
```

to:

- `SyncRegisterRequest`
- `VaultMetadataResponse`

Or add a nested serializable `Argon2Params` if dependency boundaries allow it cleanly.

### API DB schema changes

Update:

```text
crates/porkpie-api/src/db.rs
```

Add columns to server `vaults` table:

```sql
kdf_time_cost INTEGER NOT NULL DEFAULT 2
kdf_mem_cost INTEGER NOT NULL DEFAULT 19456
kdf_parallelism INTEGER NOT NULL DEFAULT 1
```

Add migration for existing server DBs.

Update:

- `register_vault`
- `load_vault_metadata`
- any stub vault registration path
- tests

### CLI sync changes

Update:

```text
crates/porkpie-cli/src/commands/sync.rs
```

When registering a vault, send:

```rust
vault_data.kdf_params.time_cost
vault_data.kdf_params.mem_cost
vault_data.kdf_params.parallelism
```

When pulling vault metadata for a new/peer device, ensure the returned metadata can reconstruct `EncryptedVaultData` with the correct params.

### Docs

Update:

```text
docs/SYNC_PROTOCOL.md
docs/DATA_MODEL.md
docs/CRYPTO_FORMAT.md
docs/THREAT_MODEL.md
```

Document that KDF params are non-secret metadata required for unlock compatibility.

## Required Tests

Add API/sync tests:

1. Create a vault with non-default KDF params.
2. Register it with sync server.
3. Fetch vault metadata.
4. Assert returned KDF params match.
5. Reconstruct locked vault from returned metadata.
6. Unlock succeeds with the upgraded params.
7. A simulated default-param unlock fails after KDF upgrade.

## Acceptance Criteria

- Server stores KDF metadata.
- Sync register sends KDF metadata.
- Vault metadata fetch returns KDF metadata.
- New device/peer reconstruction works after KDF upgrade.
- Global validation passes.

---

# Task 3: Recheck API Key Admin Endpoint Semantics

## Severity

Medium.

## Problem

Admin endpoints now require admin auth and revoke by key ID. Good.

But `admin_add_api_key` docs/comments say the plaintext key is returned only once, while the implementation returns `key_hash`, not the plaintext key. It also expects the client to provide the plaintext `api_key` instead of generating one server-side.

This is not immediately dangerous, but the docs and API semantics are muddy.

## Required Decision

Pick one behavior and make code/docs match.

### Option A: Client-provided API keys

If clients provide keys:

- Keep request body requiring `api_key`.
- Do not claim server generates or returns plaintext.
- Do not return `key_hash` unless there is a real admin need.
- Return `key_id`, `label`, and status.

### Option B: Server-generated API keys

If server generates keys:

- Request body accepts `label`.
- Server generates a strong random API key.
- Server stores only its hash.
- Server returns plaintext API key exactly once.
- Tests prove raw key is not stored.

## Acceptance Criteria

- API docs match behavior.
- Admin add endpoint does not misleadingly describe plaintext return.
- Raw keys are never stored.
- Tests cover chosen behavior.
- Global validation passes.

---

# Task 4: Recovery Restore Honesty

## Severity

Medium.

## Problem

`porkpie recovery restore` is still intentionally not implemented. That is acceptable only if docs and status make it explicit.

## Required Fix

Either implement real restore or keep the command explicitly unavailable.

Current acceptable behavior:

```text
porkpie recovery restore is not implemented yet.
```

Required docs must say the same.

## Acceptance Criteria

- README, STATUS, COMPLETION_GATE, and AUDIT_REPORT do not claim recovery restore is complete.
- If the command remains, it clearly prints not implemented.
- Global validation passes.

---

# Task 5: Final Docs Gate Reconciliation

## Severity

Medium.

## Problem

After Tasks 1-4, docs need to reflect the real state.

Update:

```text
README.md
STATUS.md
docs/COMPLETION_GATE.md
docs/AUDIT_REPORT.md
docs/THREAT_MODEL.md
docs/SYNC_PROTOCOL.md
docs/DATA_MODEL.md
docs/CRYPTO_FORMAT.md
```

Docs must specifically state:

- OS keychain integration status,
- KDF metadata sync status,
- local-secret rotation safety status,
- API key rotation semantics,
- recovery restore status,
- SSH agent integration level,
- real-secret safety status.

## Acceptance Criteria

- Docs match source.
- No scaffold is marked complete.
- No “safe for real credentials” claim.
- Global validation passes.
