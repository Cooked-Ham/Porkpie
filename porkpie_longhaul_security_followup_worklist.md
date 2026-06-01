# Porkpie Long-Haul Security Follow-up Worklist

Generated: 2026-06-01T12:51:19+00:00

This worklist is based on a source inspection after commit `10d54bc` claimed all 10 long-horizon security phases were complete.

Verdict: the commit adds useful pieces, but the long-haul security work is **not complete**. Several items are scaffolds or partial integrations. The two highest-risk issues are:

1. OS keychain storage exists as a module, but `unlock` still writes an obfuscated local secret key into `.porkpie-session.json`.
2. KDF upgrade can rewrap the vault key using non-default Argon2 parameters, but the vault metadata does not persist those parameters and unlock still uses defaults. That can brick a vault.

Do not add new shiny features before this file is handled.

---

## Global Binding Rules

You are working on Porkpie.

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

If infra/API is touched:

```bash
docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config
```

Required final report:

```markdown
# Long-Haul Follow-up Report

## Summary

## Files Changed

## Commands Run

## Test Results

## Security Notes

## Docs Updated

## Remaining Risks

## Whether Real Credentials Are Safe
No.
```

---

# Phase 01: Fix OS Keychain Integration for Real

## Severity

Critical.

## Problem

A `SecretStore` trait and OS keychain backend exist, but the CLI unlock flow still asks for the local secret key visibly and saves a session using `SessionState::unlocked_with_key()`. That method stores `secret_key_encrypted`, which is only obfuscated with a key derived from the vault ID.

This contradicts the CLI module comment saying session state does not store the local secret key and that it is stored in the OS keychain.

## Required Fix

### 1. Change unlock flow

In `crates/porkpie-cli/src/commands/unlock.rs`:

- Use `prompt_secret()` or equivalent hidden input for the local secret key.
- After successful unlock, call `default_secret_store().store_local_secret_key(&vault_id, &secret_key)`.
- Save session using `SessionState::unlocked(vault_id)`, not `SessionState::unlocked_with_key(...)`.
- If keychain storage fails, do **not** silently store the secret in the session file.
- Decide one explicit fallback:
  - fail unlock with a clear error, or
  - allow `--no-store-secret` / `--allow-insecure-session-secret` as an explicit dangerous option.

Recommended default: fail closed unless user opts into not remembering the local secret key.

### 2. Remove or quarantine legacy session secret writing

In `crates/porkpie-cli/src/session.rs`:

- Remove `unlocked_with_key()` from normal production use.
- Mark `secret_key_hex` and `secret_key_encrypted` as legacy read-only migration fields.
- Remove `encrypt_secret_key()` from new-session creation paths.
- Remove `derive_session_key()` from normal production paths unless it is only used for legacy migration.
- Add comments making the legacy path clearly deprecated.

### 3. Make migration testable

Current migration depends on `default_secret_store()`, which makes it hard to test.

Refactor:

```rust
pub fn migrate_legacy_secret_with_store(
    &mut self,
    store: &dyn SecretStore,
) -> Result<MigrationOutcome>
```

Then `migrate_legacy_secret()` can call the default store wrapper.

Tests must prove:

- legacy `secret_key_hex` migrates into fake keychain and clears the session field,
- legacy `secret_key_encrypted` migrates into fake keychain and clears the session field,
- failed keychain write does not clear legacy fields silently,
- new session JSON never contains `secret_key_hex`,
- new session JSON never contains `secret_key_encrypted`,
- unlock stores local secret key in keychain backend.

### 4. Add keychain unavailable behavior

Define exact behavior when keychain is unavailable:

Recommended:

- CLI unlock succeeds but does not mark session as reusable unless user explicitly passes `--no-store-secret`.
- Subsequent commands requiring the local secret key should ask for it again or fail with a clear message.
- Never silently write obfuscated local secret key to disk.

### 5. Update docs

Update:

```text
docs/SECURITY_INVARIANTS.md
docs/THREAT_MODEL.md
docs/COMPLETION_GATE.md
README.md
```

Docs must say:

- keychain storage is used when available,
- session file stores no local secret key in new sessions,
- legacy sessions are migrated,
- headless Linux may require Secret Service setup or explicit no-store mode.

## Acceptance Criteria

- `unlock` no longer writes `secret_key_encrypted` for new sessions.
- local secret key prompt is hidden.
- OS keychain storage is actually called after unlock.
- New session JSON contains no local secret material.
- Legacy migration is tested with `FakeKeychain`.
- Global validation passes.

---

# Phase 02: Persist KDF Parameters Before Any KDF Upgrade

## Severity

Critical.

## Problem

`porkpie vault upgrade-kdf` derives a new master key using selected Argon2 parameters and rewraps the vault key, but vault metadata has nowhere to store those parameters. `Vault::unlock()` always uses `Argon2Params::default()`.

That means upgrading to `hardened` or `paranoid` can make future unlocks fail, because the wrapped key was produced using non-default KDF params but unlock keeps deriving with default params.

## Required Fix

### 1. Make `Argon2Params` serializable and persistent

Add traits:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argon2Params {
    pub time_cost: u32,
    pub mem_cost: u32,
    pub parallelism: u32,
}
```

### 2. Add KDF metadata to vault schema

Add columns to local and server vault metadata where applicable:

```sql
kdf_time_cost INTEGER NOT NULL DEFAULT 2
kdf_mem_cost INTEGER NOT NULL DEFAULT 19456
kdf_parallelism INTEGER NOT NULL DEFAULT 1
```

or a JSON column if preferred.

Update:

```text
crates/porkpie-store/src/migrations.rs
crates/porkpie-store/src/models.rs
crates/porkpie-store/src/vault_store.rs
crates/porkpie-api/src/db.rs
docs/DATA_MODEL.md
docs/CRYPTO_FORMAT.md
```

### 3. Store params in Vault

`Vault` should carry `kdf_params`.

- `Vault::create()` uses requested profile or default and stores it.
- `Vault::from_encrypted_metadata()` accepts params.
- `Vault::unlock()` derives using `self.kdf_params`, not `Argon2Params::default()`.

### 4. Update every rewrap path

These functions must use stored/current params or update metadata atomically:

- `change_password`
- `rotate_local_secret`
- `rotate_vault_key`
- `upgrade_kdf`
- backup export/import
- sync metadata if relevant

### 5. Make KDF upgrade atomic

`upgrade_kdf` must:

1. unlock with current stored params,
2. derive new wrapping key with new params,
3. update `master_key_wrapped` and KDF metadata together,
4. confirm future unlock uses new params.

### 6. Add tests

Tests must prove:

- default vault stores default params,
- `hardened` upgrade persists hardened params,
- vault unlocks after KDF upgrade,
- old/default params fail after upgrade if used manually,
- backup/export/import preserves params,
- change-password after KDF upgrade uses the current params,
- rotate-local-secret after KDF upgrade uses the current params,
- rotate-vault-key after KDF upgrade uses the current params.

## Acceptance Criteria

- No KDF upgrade can brick a vault.
- KDF params are persisted.
- Unlock uses stored params.
- All rewrap commands use the correct params.
- Global validation passes.

---

# Phase 03: Make Vault Rotation Transactional and Backup-Safe

## Severity

High.

## Problem

`rotate_key` currently re-encrypts items, persists item ciphertexts in a loop, then updates the wrapped vault key. If any item update succeeds and a later update or wrapped-key update fails, the vault can be left partially corrupted.

Also, the command says it requires backup first, but the implementation errors unless `--skip-backup` is supplied.

## Required Fix

### 1. Wrap full rotation in a transaction

Implement a store operation such as:

```rust
pub async fn rotate_vault_key_transactional(
    pool: &SqlitePool,
    vault_id: &VaultId,
    new_wrapped_key: &[u8],
    reencrypted_items: &[(ItemId, Vec<u8>)],
) -> Result<()>
```

It must:

- begin transaction,
- update all item ciphertexts,
- update vault wrapped key,
- update KDF params if needed,
- commit,
- rollback on any error.

### 2. Do not mutate persistent state until re-encryption succeeds

Avoid changing in-memory vault state in a way that can no longer match DB if persistence fails.

Preferred pattern:

- clone/decrypt items,
- generate new key,
- produce all ciphertexts in memory,
- persist transaction,
- only then update in-memory state.

### 3. Implement backup behavior honestly

Either:

Option A, preferred:

```bash
porkpie vault rotate-key --backup-first
```

automatically writes an encrypted backup before rotation.

Option B:

```bash
porkpie vault rotate-key --skip-backup
```

continues to exist, but docs must say this is dangerous and not the default.

Do not claim automatic backup exists unless it does.

### 4. Add tests

Tests:

- successful rotation changes ciphertexts and keeps items decryptable,
- injected item update failure rolls back all item ciphertexts,
- injected wrapped-key update failure rolls back all item ciphertexts,
- no partial rotation remains after failure,
- backup is produced unless explicit skip is used.

## Acceptance Criteria

- Rotation is atomic.
- Rotation cannot leave half-new/half-old item ciphertexts.
- Backup behavior is implemented or honestly documented.
- Global validation passes.

---

# Phase 04: Fix Local Secret Rotation Session/Keychain Behavior

## Severity

High.

## Problem

`rotate_local_secret` rewraps the vault with a new local secret key, then tries to store the new secret key in the keychain but ignores the result.

If keychain write fails, the DB may require the new local secret key while the session/keychain still has the old one. The user gets a printed recovery kit, but the active session and subsequent commands can break.

## Required Fix

### 1. Do not ignore keychain write failure

Replace:

```rust
let _ = store.store_local_secret_key(&vault_id, &new_secret_key);
```

with real error handling.

Suggested behavior:

- generate recovery kit,
- ask user to confirm they saved it,
- attempt keychain write,
- if keychain fails, warn and require explicit confirmation before committing DB update,
- or abort before DB update.

### 2. Order operations safely

Safer sequence:

1. unlock and verify current credentials,
2. generate new local secret and recovery kit,
3. display/write recovery kit,
4. require confirmation,
5. store new local secret in keychain,
6. update wrapped vault key in DB,
7. update session state,
8. delete old keychain entry if needed.

### 3. Add tests with fake keychain

Tests:

- keychain success updates wrapped key and session can use new key,
- keychain failure aborts DB update unless explicit override,
- old local secret fails after rotation,
- new local secret succeeds,
- recovery kit secret matches new local secret,
- old keychain entry is removed or replaced.

## Acceptance Criteria

- No ignored keychain write errors.
- Local secret rotation cannot silently strand the user.
- Tests cover keychain failure.
- Global validation passes.

---

# Phase 05: Clarify Recovery Restore: Implement or Rename Scaffold

## Severity

Medium.

## Problem

The report says `porkpie recovery restore` is a scaffold. A scaffold should not be presented as completed recovery.

## Required Fix

Pick one.

### Option A: Implement real restore

`porkpie recovery restore --kit recovery-kit.json --backup backup.json`

Must:

- read kit,
- validate structure,
- extract local secret key without printing it,
- read encrypted backup,
- prompt for master password,
- decrypt/import backup into selected/new local DB,
- verify restored vault unlocks,
- report summary.

Tests:

- restore from kit + encrypted backup works,
- wrong kit fails,
- wrong password fails,
- restore does not print local secret key,
- restored DB does not contain plaintext fixture secrets.

### Option B: Rename and mark unavailable

If restore is not implemented, change command behavior and docs:

```text
porkpie recovery restore is not implemented yet.
```

or hide the command behind an experimental feature flag.

## Acceptance Criteria

- No scaffold is described as a completed phase.
- Restore either works or is honestly unavailable.
- Docs match behavior.
- Global validation passes.

---

# Phase 06: API Key Rotation Admin Safety

## Severity

Medium to High.

## Problem

Admin endpoints were added for API key add/revoke. They need a hostile review.

## Required Checks

Inspect and harden:

```text
crates/porkpie-api/src/lib.rs
crates/porkpie-api/src/handlers.rs
crates/porkpie-api/src/auth.rs
crates/porkpie-api/src/db.rs
docs/SYNC_PROTOCOL.md
```

Questions to answer in code/tests:

1. Are admin endpoints authenticated?
2. Are admin endpoints protected by the same bearer key they can revoke?
3. Can a client revoke all keys and lock out the server?
4. Is revocation by hash safe, or should it be by key ID?
5. Are raw keys ever logged, returned, or stored?
6. Is there a bootstrap admin key flow?
7. Is `last_used_at` tracked?
8. Can multiple keys coexist during rotation?

## Required Fixes

- Prefer key IDs for revocation, not raw hash input from clients.
- Require admin scope or a specific admin API key.
- Return plaintext API key only once at creation.
- Store only hash.
- Add label, created_at, revoked_at, last_used_at.
- Prevent revoking the last active admin key unless `--force`/explicit endpoint.

## Acceptance Criteria

- API key rotation is operationally safe.
- Tests cover key creation, auth, revocation, multiple keys, last active key protection.
- No raw key storage.
- Global validation passes.

---

# Phase 07: SSH Agent Integration Honesty and Next Step

## Severity

Medium.

## Problem

The agent protocol implementation exists, but the report still says `porkpie ssh-agent` prints an honest status about integration level. That means real OpenSSH socket/named-pipe integration is not done.

## Required Fix

No need to implement full agent immediately, but docs and completion gate must distinguish:

- protocol codec implemented,
- in-memory signer implemented,
- OpenSSH socket integration not implemented,
- production SSH-agent workflow not complete.

If implementing next:

- Unix socket on Linux/macOS,
- Windows named pipe,
- `SSH_AUTH_SOCK` docs,
- identity request,
- sign request,
- locked vault denial,
- host/key policy,
- approval prompt.

## Acceptance Criteria

- No docs imply OpenSSH agent is complete unless it truly works with `ssh`.
- If left incomplete, status says “protocol foundation only.”
- If implemented, add real manual QA command: `ssh -T git@github.com`.
- Global validation passes.

---

# Phase 08: Documentation Gate Reconciliation

## Severity

Medium.

## Problem

After these fixes, docs must not claim the long-haul work is fully complete unless the above phases truly pass.

## Required Updates

Update:

```text
docs/COMPLETION_GATE.md
docs/AUDIT_REPORT.md
docs/STATUS.md
docs/THREAT_MODEL.md
README.md
```

Docs must specifically track:

- keychain integration status,
- KDF param persistence status,
- rotation transaction safety,
- recovery restore status,
- API key rotation safety,
- SSH agent integration level,
- real-secret safety status.

## Acceptance Criteria

- Docs match code.
- No scaffold is marked complete.
- Current blockers are clear.
- Global validation passes.
