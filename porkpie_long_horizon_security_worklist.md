# Porkpie Long-Horizon Security Hardening Worklist

Generated: 2026-06-01T10:34:45+00:00

This is an agent-ready worklist for the next Porkpie hardening cycle.

External security audit is intentionally moved far down the line. Porkpie is a free personal project right now, not an enterprise product with procurement goblins circling the building. The goal here is to keep improving the security posture in practical, buildable increments.

Current baseline:

- Porkpie is a serious Rust prototype with real crypto and architecture.
- It is still not safe for real credentials until the remaining security gates are addressed.
- External audit is not part of this worklist’s near-term acceptance criteria.
- Every task must keep the Rust/Dioxus/Axum/SQLx architecture intact.

---

## Global Binding Rules

You are working on Porkpie.

Repository: `Cooked-Ham/Porkpie`

Product identity:

- Project: Porkpie
- Domain: `porkpie.love`
- CLI binary: `porkpie`
- Secret URI scheme: `pie://Vault/Item/field`
- Language: Rust
- UI: Dioxus
- API: Axum
- Runtime: Tokio
- Persistence: SQLx
- Local DB: SQLite
- Sync: zero-knowledge encrypted blob replication

Hard rules:

1. Do not introduce Electron.
2. Do not introduce React as the app frontend.
3. Do not introduce TypeScript/Vite as the product UI foundation.
4. Do not weaken crypto.
5. Do not store plaintext secrets.
6. Do not log secrets.
7. Do not bypass redaction.
8. Do not store master passwords.
9. Do not server-side decrypt vault items.
10. Do not add hardcoded keys or public default secrets.
11. Do not delete tests to make CI green.
12. Do not suppress Clippy broadly.
13. Do not claim external audit has happened.
14. Do not claim real-secret safety unless the completion gate is updated honestly.

Required validation before any phase is considered complete:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

If web/Dioxus code is touched:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```

If infra/API config is touched:

```bash
docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config
```

Required final report format:

```markdown
# Phase Report

## Summary

## Files Changed

## Commands Run

## Test Results

## Security Notes

## Docs Updated

## Remaining Risks

## Whether Real Credentials Are Safe
No, unless all current security gates pass and the user knowingly accepts the no-external-audit status.
```

---

# Phase 01: Memory Zeroization Strategy and Verification

## Goal

Replace the vague “memory zeroization not verified” blocker with a concrete, honest memory-handling model.

This phase should not pretend Rust can magically prove all heap memory is wiped. The goal is to:

- zeroize secret-owning containers where practical,
- minimize plaintext lifetime,
- add tests for types that can be directly verified,
- document the limits honestly.

## Target Areas

Inspect and harden:

```text
crates/porkpie-core/src/vault.rs
crates/porkpie-core/src/item.rs
crates/porkpie-types/src/item_type.rs
crates/porkpie-types/src/secret_key.rs
crates/porkpie-crypto/src/*
crates/porkpie-cli/src/session.rs
crates/porkpie-ui/src/state.rs
docs/SECURITY_INVARIANTS.md
docs/CRYPTO_FORMAT.md
docs/COMPLETION_GATE.md
```

## Tasks

### 1. Identify secret-bearing types

Create or update a doc section listing every type that may hold plaintext secrets in memory:

- master password wrappers
- local secret key
- vault key
- item field values
- decrypted vault item map
- generated passwords
- recovery kit contents
- CLI input buffers
- import/export buffers
- session material
- SSH private keys
- API tokens
- database passwords
- recovery codes
- custom secret values

Document whether each one is:

- zeroized on drop,
- redacted in Debug,
- short-lived only,
- not currently zeroized,
- impossible to prove fully due to Rust allocator/clone behavior.

### 2. Use `secrecy` / `zeroize` more consistently

Prefer:

```rust
SecretString
SecretVec<u8>
Zeroizing<String>
Zeroizing<Vec<u8>>
```

or custom wrappers using `ZeroizeOnDrop`.

Do not expose secret inner values except at the exact boundary where needed.

### 3. Add zeroize-on-drop wrappers where practical

Focus on:

- local secret key bytes,
- vault key bytes,
- generated passwords,
- temporary CLI secret input,
- plaintext import buffers,
- decrypted private key buffers,
- recovery kit local secret key string where feasible.

Do not break serialization of recovery kits. Recovery kits must still export the local secret key intentionally, but Debug must remain redacted.

### 4. Make vault lock call explicit zeroization paths

When locking:

- clear decrypted item map,
- zeroize vault key,
- zeroize temporary plaintext secret buffers if owned by state,
- clear selected item detail state in UI,
- clear generated password state if it contains a generated secret.

The lock path must be intentional and documented.

### 5. Add tests that are realistic

Add tests for:

- `LocalSecretKey` zeroizes owned bytes on drop if testable.
- generated password state zeroizes or clears on reset/lock.
- vault lock clears decrypted item map.
- UI lock clears selected/decrypted item detail state.
- import plaintext buffers are not persisted after import.

Avoid fake tests that inspect undefined allocator behavior and then call it security proof. We are building software, not writing a fantasy novel in `unsafe`.

### 6. Add honest documentation

Update completion gate from:

```text
Memory zeroization is not verified by tests.
```

to one of these, depending on implementation:

Option A, if implemented enough:

```text
Memory hygiene implemented for owned secret buffers where practical. Tests verify explicit zeroize/clear paths. Full heap erasure cannot be proven portably in Rust and remains a documented limitation.
```

Option B, if incomplete:

```text
Memory hygiene partially implemented. Vault lock clears decrypted state, but full zeroization coverage remains incomplete.
```

## Acceptance Criteria

- Secret-bearing memory model is documented.
- Secret wrappers are applied where practical.
- Vault lock zeroizes or clears owned secret state.
- Tests cover explicit clear/zeroize behavior.
- Docs do not overclaim full heap erasure.
- Global validation passes.

---

# Phase 02: OS Keychain Storage for Session Secret

## Goal

Stop storing the local secret key in `.porkpie-session.json` as weak/obfuscated material.

The session file currently improves UX but weakens the local secret key model. This phase moves local-secret-key session storage into platform-protected storage.

## Target Platforms

Implement a trait-based abstraction:

```rust
pub trait SecretStore {
    fn store_local_secret_key(&self, vault_id: &VaultId, key: &LocalSecretKey) -> Result<()>;
    fn load_local_secret_key(&self, vault_id: &VaultId) -> Result<Option<LocalSecretKey>>;
    fn delete_local_secret_key(&self, vault_id: &VaultId) -> Result<()>;
}
```

Backends:

- Windows: DPAPI / Windows Credential Manager
- macOS: Keychain
- Linux: Secret Service / libsecret
- Fallback: disabled by default or explicit file fallback with scary warning

Prefer using a maintained Rust crate if suitable, such as `keyring`, but evaluate whether it supports all target platforms and headless Linux behavior.

## Tasks

### 1. Create session secret storage crate/module

Suggested location:

```text
crates/porkpie-store/src/secret_store.rs
```

or a new crate:

```text
crates/porkpie-keychain/
```

Use a trait so tests can use an in-memory fake backend.

### 2. Change session file model

Session file may store:

- current vault ID,
- unlocked session marker,
- selected profile/server info,
- keychain lookup metadata.

Session file must not store:

- local secret key,
- master password,
- vault key,
- item secrets,
- decryptable local secret key material.

### 3. Add migration behavior

If an old `.porkpie-session.json` contains a local secret key:

- read it once,
- store it in OS keychain,
- rewrite session file without the secret,
- warn user that migration happened,
- securely delete old contents as much as practical.

Do not silently preserve the old secret field.

### 4. Add CLI flags for explicit behavior

Add one or both:

```bash
porkpie unlock --no-store-secret
porkpie unlock --store-secret
```

Default should be conservative and documented.

Suggested behavior:

- Desktop app: offer “remember this device” checkbox.
- CLI: default to keychain if available, otherwise require local secret key or recovery kit each session.
- Headless environments: allow explicit fallback only with a warning.

### 5. Add tests

Add tests using fake keychain backend:

- unlock stores key only in keychain backend,
- session JSON does not contain local secret key,
- lock deletes session state if intended,
- migration removes legacy session secret field,
- missing keychain entry prompts for local secret key again,
- failure to store keychain secret does not write plaintext fallback silently.

### 6. Update docs

Update:

```text
docs/SECURITY_INVARIANTS.md
docs/STATUS.md
docs/COMPLETION_GATE.md
README.md
```

Document:

- OS keychain support,
- headless Linux behavior,
- fallback behavior,
- migration from old session files,
- why `.porkpie-session.json` is no longer sufficient for unlocking.

## Acceptance Criteria

- Session JSON no longer stores local secret key.
- OS keychain abstraction exists.
- Migration path from old session file exists.
- Tests prove session file does not contain local secret key.
- Docs explain platform behavior.
- Global validation passes.

---

# Phase 03: Vault Key Rotation and Password/Secret-Key Change

## Goal

Add a real key rotation story.

Right now, if a vault key is compromised, the user’s cleanest option is creating a new vault. That is fine for a prototype, but we can do better because apparently we enjoy responsible suffering.

## Definitions

There are several distinct rotations:

1. **Change master password:** rederive unlock key and rewrap existing vault key.
2. **Change local secret key:** generate/accept new local secret key and rewrap existing vault key.
3. **Rotate vault key:** generate new vault key and re-encrypt all items.
4. **Rotate item keys:** optional future design if item-level keys exist later.
5. **Rotate sync/API keys:** server auth rotation, not vault crypto.

## Tasks

### 1. Add password change

CLI:

```bash
porkpie vault change-password
```

Behavior:

- require current master password,
- require current local secret key from keychain or prompt,
- unlock vault,
- prompt for new master password twice,
- rewrap existing vault key,
- do not re-encrypt all items,
- update vault metadata,
- keep item ciphertext unchanged,
- update tests.

### 2. Add local secret key rotation

CLI:

```bash
porkpie vault rotate-local-secret
```

Behavior:

- require current unlock,
- generate new local secret key,
- rewrap existing vault key,
- produce new recovery kit,
- update OS keychain/session metadata,
- invalidate old local secret key,
- warn user to save new recovery kit.

### 3. Add full vault key rotation

CLI:

```bash
porkpie vault rotate-key
```

Behavior:

- require current unlock,
- generate new vault key,
- decrypt each item with old vault key,
- re-encrypt each item with new vault key and same AAD model,
- rewrap new vault key with current password + local secret key,
- bump sync revisions for all items,
- save new encrypted metadata and item ciphertexts,
- record rotation event in local metadata.

### 4. Add dry-run and backup requirement

Full vault key rotation should require either:

```bash
porkpie vault rotate-key --backup-first
```

or automatically create encrypted backup first.

Do not allow blind mass re-encryption without a safe rollback plan. We are not animals. Mostly.

### 5. Add tests

Tests:

- change password keeps items decryptable with new password.
- old password fails after change.
- local secret rotation invalidates old local secret key.
- new recovery kit unlocks after local secret rotation.
- vault key rotation changes item ciphertext.
- vault key rotation keeps plaintext item data equivalent after decrypt.
- failed rotation does not corrupt vault.
- sync revisions bump after full key rotation.

### 6. Add docs

Update:

```text
docs/CRYPTO_FORMAT.md
docs/SECURITY_INVARIANTS.md
docs/RECOVERY.md
docs/COMPLETION_GATE.md
README.md
```

## Acceptance Criteria

- User can change master password.
- User can rotate local secret key and receive new recovery kit.
- User can rotate vault key with backup/rollback safety.
- Old credentials fail where expected.
- New credentials work.
- Tests cover failure modes.
- Global validation passes.

---

# Phase 04: Safer Secret Output and Clipboard Ergonomics

## Goal

Make `porkpie read` safer while preserving explicit reveal.

Printing secrets to stdout is expected behavior for a CLI password manager, but it is still dangerous. Add safer ergonomics so users naturally choose `copy` or controlled output.

## Tasks

### 1. Add `--no-newline`

```bash
porkpie read pie://Personal/GitHub/password --no-newline
```

Useful for scripts.

### 2. Add TTY warning

If stdout is a terminal, optionally warn to stderr:

```text
Warning: printing secret to terminal. Prefer `porkpie copy pie://...`.
```

Do not warn in noninteractive scripts unless configured.

Flags:

```bash
porkpie read <pie-uri> --quiet
porkpie read <pie-uri> --warn
```

Suggested default:

- warn when stdout is TTY,
- no warning when stdout is piped.

### 3. Add timed clipboard clearing

For `porkpie copy`:

```bash
porkpie copy pie://Personal/GitHub/password --clear-after 45s
```

Support:

- `--clear-after 0` disables clearing,
- default maybe 45 seconds,
- if clearing unsupported, print warning.

### 4. Add field display policy

`item get` should remain redacted by default.

Optionally add:

```bash
porkpie item get <id> --reveal-field password
```

But prefer `porkpie read pie://...`.

### 5. Add tests

Tests:

- `read --no-newline` has no newline.
- `read` with piped stdout does not warn.
- TTY warning logic unit-tested via abstraction.
- `copy --clear-after` argument parsing works.
- invalid duration fails.
- redacted commands remain redacted.

### 6. Docs

Update CLI docs:

- `copy` recommended for humans,
- `read` for scripts,
- `read` can leak into terminal scrollback,
- literal `write` values can leak into shell history.

## Acceptance Criteria

- `read` output is script-friendly.
- TTY warning exists or is documented as not implemented.
- Clipboard clearing exists where supported.
- Redaction defaults remain.
- Global validation passes.

---

# Phase 05: Argon2id Calibration and Profiles

## Goal

Make key derivation parameters adaptive instead of hardcoded forever.

Current parameters are conservative. That is fine for broad compatibility, but users should be able to choose stronger profiles and optionally calibrate to their machine.

## Tasks

### 1. Define KDF profiles

Examples:

```text
low-memory
standard
hardened
paranoid
calibrated
```

Store KDF params in vault metadata.

### 2. Add calibration command

CLI:

```bash
porkpie vault calibrate-kdf --target-ms 750
```

Behavior:

- benchmark Argon2id with increasing memory/time cost,
- select params near target time,
- cap memory based on platform,
- never reduce existing KDF strength without explicit confirmation.

### 3. Add profile selection at vault creation

CLI:

```bash
porkpie init --kdf-profile hardened
```

UI:

- dropdown or simple choice during onboarding,
- explain tradeoff: slower unlock, better offline resistance.

### 4. Add migration/upgrade command

```bash
porkpie vault upgrade-kdf --profile hardened
```

Behavior:

- unlock vault,
- derive new wrapping key with new params,
- rewrap vault key,
- item ciphertext unchanged,
- store new KDF params,
- old password still same, old KDF params no longer unlock.

### 5. Tests

Tests:

- vault metadata persists KDF params.
- wrong KDF params fail.
- upgrade profile rewraps vault key.
- items remain decryptable after upgrade.
- old metadata backup still restores if correct KDF params are present.
- calibration function can be unit-tested with mocked benchmark runner.

## Acceptance Criteria

- KDF params are stored per vault.
- Users can create/upgrade vaults with stronger KDF profiles.
- Calibration is available or documented as pending.
- Tests cover rewrap behavior.
- Global validation passes.

---

# Phase 06: OpenSSH Agent Integration

## Goal

Turn the current SSH-agent foundation into an actual usable OpenSSH-compatible agent.

Do not claim this is done until an actual `ssh -T git@...` style workflow can use a vault-held key without writing the private key to disk.

## Tasks

### 1. Implement platform sockets

Targets:

- Linux/macOS: Unix socket
- Windows: named pipe or compatible OpenSSH agent integration

CLI:

```bash
porkpie ssh-agent start
porkpie ssh-agent status
porkpie ssh-agent stop
```

or keep:

```bash
porkpie ssh-agent
```

but add real subcommands if cleaner.

### 2. Implement OpenSSH agent protocol

Support at minimum:

- request identities,
- sign request,
- failure response,
- optional extension handling as unsupported.

Do not support adding arbitrary private keys into Porkpie agent from external clients unless explicitly designed.

### 3. Host/key policy

Add config:

```toml
[[ssh.keys]]
item = "pie://Personal/GitHub/private_key"
hosts = ["github.com"]
confirm = true
```

Or store policy in vault item metadata.

### 4. Approval flow

For signing:

- require unlocked vault,
- optional confirmation per key/host,
- timeout approvals,
- no private key export,
- log nonsecret metadata only.

### 5. Git signing support

Optional but useful:

- support SSH commit signing where possible,
- document Git config.

### 6. Tests

Tests:

- protocol identity request returns public key only.
- sign request returns valid signature.
- private key never appears in logs/stdout.
- locked vault refuses signing.
- host policy denies unauthorized host.
- approval timeout denies signing.
- Windows-specific tests if platform available, otherwise feature-gated.

### 7. Docs

Update:

```text
docs/SSH_AGENT.md
README.md
docs/STATUS.md
```

Include setup examples:

```bash
export SSH_AUTH_SOCK=...
ssh -T git@github.com
```

Windows PowerShell equivalent too.

## Acceptance Criteria

- OpenSSH-compatible agent works on at least one platform.
- Private key remains encrypted at rest.
- Signing happens in memory only.
- Locked vault refuses signing.
- Docs are honest about platform support.
- Global validation passes.

---

# Phase 07: Recovery and Emergency Access Model

## Goal

Improve recovery without creating a server-side backdoor.

This is not enterprise emergency access yet. Start with local/family-safe recovery.

## Tasks

### 1. Recovery kit UX

Improve recovery kit:

- printable format,
- QR or chunked text optional,
- clear restore instructions,
- clear warning about local secret key.

### 2. Recovery verification command

CLI:

```bash
porkpie recovery verify --kit path/to/recovery-kit.json
```

Behavior:

- verifies kit structure,
- does not print local secret key,
- optionally attempts unlock with provided vault DB and password.

### 3. Recovery restore command

```bash
porkpie recovery restore --kit recovery-kit.json --backup porkpie-backup.json.enc
```

### 4. Optional trusted-contact model later

Do not implement cryptographic sharing unless properly designed.

Document future idea:

- Shamir secret sharing for local secret key,
- encrypted recovery share,
- time-delay emergency access.

### 5. Tests

Tests:

- recovery kit exports.
- recovery kit Debug redacts secret.
- recovery verify validates structure.
- restore from kit + encrypted backup works.
- wrong recovery kit fails.

## Acceptance Criteria

- Recovery flow is testable.
- Recovery docs are clear.
- No server-side recovery backdoor.
- Global validation passes.

---

# Phase 08: Key Rotation for Sync/API Tokens

## Goal

Make server/API credential rotation practical for self-hosting.

This is separate from vault crypto rotation.

## Tasks

### 1. Add server API key management

CLI or admin utility:

```bash
porkpie-server api-key create
porkpie-server api-key list
porkpie-server api-key revoke
porkpie-server api-key rotate
```

If server binary already owns API key bootstrap, extend that design.

### 2. Store only hashed API keys

Already likely implemented. Preserve it.

### 3. Add key metadata

Track:

- key ID,
- hash,
- created_at,
- last_used_at,
- revoked_at,
- label,
- scope if useful later.

### 4. Support multiple valid keys

So users can rotate without downtime:

1. create new key,
2. deploy clients,
3. revoke old key.

### 5. Tests

Tests:

- create key stores hash only.
- valid key authenticates.
- revoked key fails.
- two keys can coexist.
- last_used_at updates without logging key material.

## Acceptance Criteria

- API keys can be rotated without replacing the whole server DB.
- Raw keys are never stored.
- Revocation works.
- Docs updated.
- Global validation passes.

---

# Phase 09: Fuzzing and Property Tests

## Goal

Add automated stress testing before any external audit someday, because free projects can still have standards. Horrifying, but true.

## Targets

Fuzz / property-test:

- `pie://` parser
- encrypted envelope decode
- backup import parser
- CSV import parser
- sync merge/conflict logic
- item field access
- recovery kit parser

## Tasks

### 1. Add property tests

Use `proptest` or similar.

Properties:

- parse/display roundtrip for valid `pie://`.
- invalid `pie://` never panics.
- field access never panics on malformed item.
- merge is deterministic.
- conflict preservation does not drop local-only or remote-only items.

### 2. Add cargo-fuzz harnesses

Optional but useful:

```text
fuzz/fuzz_targets/pie_uri.rs
fuzz/fuzz_targets/backup_import.rs
fuzz/fuzz_targets/csv_import.rs
fuzz/fuzz_targets/sync_merge.rs
```

### 3. Add CI-compatible smoke fuzz

Do not run long fuzzing in normal CI.

Add docs:

```bash
cargo fuzz run pie_uri -- -max_total_time=60
```

## Acceptance Criteria

- Property tests added and pass.
- Fuzz harnesses exist for highest-risk parsers.
- Fuzzing docs exist.
- Global validation passes.

---

# Phase 10: Threat Model Refresh and Security Roadmap

## Goal

Update docs so Porkpie’s security story is clear and current.

This is where external audit stays parked as a future milestone, not a fake immediate blocker.

## Tasks

### 1. Update threat model

Document attackers:

- stolen server DB,
- stolen local DB,
- stolen session file,
- malware on local machine,
- malicious browser origin,
- compromised sync server,
- malicious client,
- shoulder-surfing / terminal history,
- Git commit leaks,
- lost recovery kit,
- weak master password.

### 2. Map mitigations

For each threat:

- current mitigation,
- remaining gap,
- future hardening phase.

### 3. Create security roadmap

Suggested sections:

- Done
- In progress
- Next
- Later
- External audit someday

External audit should live under “Later,” with clear rationale:

```text
External audit is valuable before recommending Porkpie for real credentials outside the developer’s own risk tolerance. It is not a near-term blocker for personal prototype hardening work.
```

### 4. Update completion gate

Make status nuanced:

- “MVP for testing” gate,
- “Personal dogfood with fake/low-risk credentials” gate,
- “Real credentials” gate,
- “Public release” gate,
- “Enterprise/commercial” gate.

Do not use one giant gate that makes everything sound equally blocked by enterprise audit theater.

## Acceptance Criteria

- Threat model is current.
- Security roadmap is clear.
- External audit is far-down-line but not forgotten.
- Docs distinguish personal dogfooding from public real-secret recommendation.
- Global validation passes.

---

# Suggested Order

Do these in order:

1. Phase 01 — Memory Zeroization Strategy and Verification
2. Phase 02 — OS Keychain Storage for Session Secret
3. Phase 03 — Vault Key Rotation and Password/Secret-Key Change
4. Phase 04 — Safer Secret Output and Clipboard Ergonomics
5. Phase 05 — Argon2id Calibration and Profiles
6. Phase 06 — OpenSSH Agent Integration
7. Phase 07 — Recovery and Emergency Access Model
8. Phase 08 — Key Rotation for Sync/API Tokens
9. Phase 09 — Fuzzing and Property Tests
10. Phase 10 — Threat Model Refresh and Security Roadmap

If you want the fastest practical improvement, start with:

```text
Phase 02: OS Keychain Storage
```

That is probably the highest-impact user-facing security upgrade after the current fixes. The memory-zeroization work is valuable, but OS keychain storage closes a more concrete local risk.
