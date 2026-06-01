# Porkpie No-Excuses Execution Protocol

Generated: 2026-06-01T15:03:28+00:00

This file overrides passive agent behavior for Porkpie work.

The rule is simple:

> A blocker is not a stopping point. A blocker is a task to resolve, route around, provision for, or explicitly remove from scope only if the feature claim is removed too.

No more stopping at:
- missing OS support,
- missing infra,
- missing package,
- missing test environment,
- missing keychain backend,
- missing SSH transport,
- missing CI runner,
- "manual QA required",
- "future phase",
- "not implemented yet",
- "external dependency",
- "platform limitation."

If Porkpie claims it, implement it.
If it cannot be implemented in the current environment, build or provision the environment.
If it still cannot be implemented, remove the claim and mark the feature unsupported.

No vibes. No victory laps. No "done except the hard part." That phrase belongs in a museum of preventable failures.

---

## Global Binding Rules

Repository: `Cooked-Ham/Porkpie`

Primary product rules:

1. Rust-first.
2. Dioxus UI.
3. Axum API.
4. SQLx persistence.
5. SQLite local DB.
6. `pie://Vault/Item/field` remains the canonical secret reference format.
7. No Electron.
8. No React as the app frontend.
9. No TypeScript/Vite as the product UI foundation.
10. No fake crypto.
11. No plaintext secret storage.
12. No server-side vault decryption.
13. No hardcoded production secrets.
14. No broad `#[allow(...)]` to fake clean Clippy.
15. No deleting tests to pass CI.
16. No unsupported feature advertised as supported.

Required validation after every implementation pass:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

If web/Dioxus is touched:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```

If API/infra is touched:

```bash
docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config
```

---

# Blocker Handling Rule

When an agent hits a blocker, it must choose one of these five actions.

## Action 1: Fix It

If the blocker is code, config, test, docs, dependency, build script, schema, migration, API behavior, or CLI behavior:

- fix it,
- add tests,
- update docs,
- rerun validation.

Examples:

- broken admin self-revoke guard,
- KDF params missing from sync metadata,
- SQLite path bug,
- CLI flag conflict,
- docs claiming stale behavior.

These are not blockers. These are tasks.

---

## Action 2: Provision It

If the blocker is missing infrastructure, create/provision it.

Examples:

- missing Linux Secret Service test environment,
- missing Windows runner,
- missing macOS keychain validation,
- missing Docker Compose service,
- missing Caddy config,
- missing test database,
- missing CI matrix.

Required behavior:

1. Add or update infra.
2. Document setup.
3. Add repeatable commands.
4. Add CI or manual QA checklist.
5. Do not mark complete until validated.

---

## Action 3: Matrix It

If the blocker is platform-specific, create a platform matrix and fill it.

Supported platforms must be explicitly tested or manually verified.

Minimum matrix:

| Feature | Windows | macOS | Linux desktop | Linux headless | Web/WASM |
|---|---:|---:|---:|---:|---:|

Applies to:

- OS keychain,
- SSH agent,
- clipboard,
- desktop app,
- file paths,
- SQLite persistence,
- recovery restore,
- browser localStorage,
- Docker Compose.

If a platform is unsupported, docs and CLI must say so clearly.

No generic "cross-platform" claims until the matrix backs it.

---

## Action 4: Build a Harness

If the blocker is "hard to test," build a harness.

Examples:

- fake keychain backend,
- fake SSH agent client,
- temp SQLite DB,
- recovery roundtrip fixture,
- API test server,
- CLI parser harness,
- WASM build check,
- property test harness.

Manual QA is acceptable only when automated testing is impractical, but it must be documented as a repeatable checklist.

---

## Action 5: Remove the Claim

If the feature will not be implemented now:

1. Remove it from marketing/docs/completion gate.
2. Hide or feature-gate the command if needed.
3. Make CLI output explicit.
4. Keep tests proving the unsupported behavior is honest.

Example:

```text
Windows SSH agent is not supported.
```

is acceptable.

This is not acceptable:

```text
SSH agent support is complete.
```

when only Unix works.

---

# Current Mandatory Work

These items must be finished. Do not stop at the first inconvenience.

## 1. Fix Admin Self-Revoke Guard

### Problem

Admin revoke checks self-revoke after mutation.

### Required implementation

1. Add DB helper:

```rust
pub async fn api_key_hash_by_id(pool: &SqlitePool, key_id: i64) -> Result<String>
```

2. In handler:

```rust
let target_hash = db::api_key_hash_by_id(&state.pool, key_id).await?;

if current_hash.0 == target_hash {
    return Err(ApiError::BadRequest(
        "Cannot revoke the API key currently in use.".to_string(),
    ));
}

db::revoke_api_key_by_id(&state.pool, key_id).await?;
```

3. Do not call `revoke_api_key_by_id` before the self-check.

### Required tests

- self-revoke fails,
- self-revoke leaves key active,
- another admin can revoke a different key,
- last-active-key guard works,
- force does not allow self-revoke unless explicitly and intentionally designed.

---

## 2. Finish SSH Agent or Narrow Claims

### Required decision

Choose A or B. No limbo.

## Path A: Implement supported SSH agent

Required:

- Unix socket works with OpenSSH.
- `ssh-add -L` works.
- `ssh -T git@github.com` works with a Porkpie-held Ed25519 key.
- Standard OpenSSH Ed25519 private key import works.
- Raw seed-only support is not enough for a normal user-facing claim.
- Docs include exact setup.

Commands should support:

```bash
porkpie ssh-agent start
porkpie ssh-agent env
porkpie ssh-agent status
porkpie ssh-agent stop
```

Manual QA:

```bash
eval "$(porkpie ssh-agent env)"
ssh-add -L
ssh -T git@github.com
```

Tests:

- identity response format,
- sign response format,
- signature verifies,
- private key not printed,
- locked vault denies agent,
- standard OpenSSH key parser test.

## Path B: Narrow claim

If not implementing full SSH agent:

- say Unix experimental,
- say raw Ed25519 seed only,
- say standard OpenSSH private key import not supported,
- say Windows unsupported,
- remove "complete SSH agent" claim.

---

## 3. Windows SSH Agent: Implement or Explicitly Exclude

Do not leave this as fog.

## Path A: Implement

Provision Windows environment or CI runner.

Implement Windows OpenSSH-compatible transport:

- named pipe if appropriate,
- PowerShell setup docs,
- manual QA checklist.

Test:

```powershell
ssh-add -L
ssh -T git@github.com
```

## Path B: Exclude

Docs and CLI must say:

```text
SSH agent is supported on Unix-like systems only. Windows SSH agent integration is not implemented.
```

No "fully cross-platform SSH agent" claim remains.

---

## 4. OS Keychain Platform Matrix

Create:

```text
docs/KEYCHAIN.md
```

Required table:

| Platform | Backend | Supported | Validation |
|---|---|---:|---|
| Windows | Credential Manager / keyring | yes/no | command |
| macOS | Keychain / keyring | yes/no | command |
| Linux desktop | Secret Service / keyring | yes/no | command |
| Linux headless | none by default | yes/no | behavior |
| Web/WASM | browser storage only | yes/no | behavior |

Add commands:

```bash
porkpie keychain status
porkpie keychain test
porkpie keychain forget <vault>
```

Required behavior:

- no silent weak fallback,
- no local secret key in session file,
- explicit no-store mode if keychain unavailable,
- fake backend tests.

If platform cannot be tested locally, provision it or document exact manual QA.

---

## 5. Memory Zeroization: Implement Best-Effort, Stop Pretending Perfect Proof

Required claim:

```text
Porkpie zeroizes owned secret buffers where practical and clears decrypted vault state on lock. It does not claim total process-memory erasure.
```

Required implementation:

- `Zeroizing<T>` or `zeroize::Zeroize` for local secret key, vault key, generated password, temporary import/export buffers where practical.
- lock clears decrypted items,
- UI lock clears selected/decrypted state,
- password generator clears generated password,
- import buffers do not persist.

Required tests:

- vault lock clears decrypted map,
- generated password is cleared/reset,
- UI lock clears selected item/detail state,
- import fixture secret absent from DB.

Do not claim allocator-level proof.

---

## 6. Recovery Restore Roundtrip Tests

Recovery restore exists. Now prove it.

Required test:

1. Create vault.
2. Add item with fixture secret.
3. Export encrypted backup.
4. Export recovery kit.
5. Restore into clean DB.
6. Unlock restored vault.
7. Decrypt item.
8. Assert fixture secret matches.
9. Assert raw DB does not contain fixture secret.
10. Wrong kit fails.
11. Wrong password fails.
12. kit/backup vault ID mismatch fails.
13. keychain failure behavior is tested.

---

## 7. Admin API Key Rotation Hardening

Required:

- self-revoke fixed,
- add/revoke are admin-authenticated,
- revocation by key ID,
- no raw key storage,
- no `key_hash` returned unless justified,
- `last_used_at` updates,
- multiple keys coexist,
- last active key guard works,
- audit log records add/revoke/deny events.

---

## 8. External Audit: Move to Public Trust Gate, Not Development Blocker

External audit is not something agents can implement. It should not block development or personal dogfooding.

Docs must define gates:

## Developer Alpha

Fake/test credentials only.

## Personal Dogfood

Developer may use low-risk real credentials with explicit personal risk acceptance.

No external audit required.

## Public Recommendation

Requires broader testing, security review, release hardening.

## Commercial / Enterprise

Requires independent external audit.

No docs should say Porkpie is externally audited.

No docs should recommend broad real-credential use without review.

---

## 9. Final Product Claim Scrub

Search docs/code for:

```text
not implemented
future phase
limitation
unsupported
not yet
scaffold
experimental
honest status
OpenSSH integration not yet implemented
safe for real credentials
production-ready
complete
external audit
```

For each hit:

- implement it,
- remove it,
- narrow it,
- or place it in an explicit unsupported section.

No stale limitation text.

No false completion text.

---

# Escalation Policy

Agents must not stop with "blocked."

Use this format:

```markdown
## Blocker Encountered

### Blocker

### Why it blocks completion

### Resolution path chosen
Fix / Provision / Matrix / Harness / Remove Claim

### Work performed

### Validation

### Remaining risk
```

If additional infra is needed, create it.

If another OS is needed, add CI matrix or manual QA instructions.

If a dependency is missing, add it.

If a platform cannot support the feature, remove the platform claim.

---

# Definition of Done

This pass is done only when:

- admin self-revoke is fixed before mutation,
- SSH agent claim is fully true or narrowed,
- Windows SSH agent is implemented or explicitly excluded,
- keychain platform behavior is deterministic,
- memory zeroization claim is realistic and tested,
- recovery restore has roundtrip tests,
- admin API key rotation is hardened,
- external audit is moved to public/commercial trust gate,
- docs have no stale false limitations,
- strict validation passes.

Final acceptable status:

```text
Porkpie is suitable for developer alpha and controlled personal dogfooding with explicit risk acceptance. It is not externally audited and should not yet be broadly recommended to other users for high-value production credentials.
```
