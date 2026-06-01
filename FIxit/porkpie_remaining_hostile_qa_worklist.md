# Porkpie Remaining Hostile-QA Worklist

Generated: 2026-06-01T10:14:35+00:00

This document is a single agent handoff for the remaining issues found after the latest claimed fix pass.

Status: Porkpie is much improved, but **do not call it done yet**. Several phase claims were mostly handled, but a few important gaps remain. No Electron/React/TypeScript/Vite nonsense was found in the inspected files, so at least the agents did not summon that particular demon.

---

## Global Binding Rules

You are working on Porkpie, a Rust-first, local-first, zero-knowledge password manager.

Repository: `Cooked-Ham/Porkpie`

Hard rules:

1. Do not introduce Electron.
2. Do not introduce React as the app frontend.
3. Do not introduce TypeScript/Vite as the product UI foundation.
4. Do not weaken crypto.
5. Do not store plaintext secrets.
6. Do not log secrets.
7. Do not bypass redaction.
8. Do not delete tests to make CI green.
9. Do not suppress Clippy broadly.
10. Do not claim MVP/safe-for-real-secrets until the completion gate truly passes and external audit is done.

Required validation before reporting completion:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

If web files are touched:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```

If infra/API config is touched:

```bash
docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config
```

Required final agent report:

```markdown
# Remaining Work Report

## Summary

## Files Changed

## Commands Run

## Test Results

## Security Notes

## Docs Updated

## Remaining Risks

## Whether Real Credentials Are Safe
No, unless external audit has completed and completion gate passes.
```

---

# Task 1: Finish Vault-Scoped Item Access in Local Store

## Severity

High.

## Problem

The schema now uses composite `PRIMARY KEY (vault_id, id)`, but several local store functions still query or mutate items by `item_id` alone.

This defeats the whole point of the composite key on the client/local store side. If two vaults contain the same item ID, these functions can load, update, or delete the wrong row.

Known risky functions in:

```text
crates/porkpie-store/src/item_store.rs
```

Current problematic patterns:

```sql
SELECT ciphertext FROM items WHERE id = ?
SELECT ... FROM items WHERE id = ?
UPDATE items SET ... WHERE id = ?
DELETE FROM items WHERE id = ?
```

## Required Fix

Update local store APIs so item operations are vault-scoped.

Preferred signatures:

```rust
pub async fn load_item(pool: &SqlitePool, vault_id: &VaultId, item_id: &ItemId) -> Result<Vec<u8>>

pub async fn load_item_record(
    pool: &SqlitePool,
    vault_id: &VaultId,
    item_id: &ItemId,
) -> Result<EncryptedItemData>

pub async fn update_item(
    pool: &SqlitePool,
    vault_id: &VaultId,
    item_id: &ItemId,
    ciphertext: &[u8],
) -> Result<()>

pub async fn delete_item(
    pool: &SqlitePool,
    vault_id: &VaultId,
    item_id: &ItemId,
) -> Result<()>
```

Every query must use:

```sql
WHERE vault_id = ? AND id = ?
```

Do not leave legacy by-ID-only helpers publicly available unless they are test-only and clearly named unsafe/internal.

## Required Call-Site Updates

Update all callers, including but not necessarily limited to:

```text
crates/porkpie-cli/src/commands/get.rs
crates/porkpie-cli/src/commands/read.rs
crates/porkpie-cli/src/commands/write.rs
crates/porkpie-cli/src/commands/copy.rs
crates/porkpie-cli/src/commands/delete.rs
crates/porkpie-ui/src/vault_store.rs
crates/porkpie-import/*
crates/porkpie-store/tests/*
```

Where the caller only has an item ID, obtain the current vault ID from the unlocked session/vault.

## Required Tests

Add a local-store malicious collision test:

1. Create Vault A.
2. Create Vault B.
3. Insert item with the same `ItemId` in both vaults.
4. `load_item(vault_a, item_id)` returns only Vault A ciphertext.
5. `load_item(vault_b, item_id)` returns only Vault B ciphertext.
6. `update_item(vault_a, item_id)` mutates only Vault A.
7. `delete_item(vault_a, item_id)` deletes only Vault A.
8. Vault B remains intact.

Also add CLI-level regression if practical:

1. Unlock Vault A.
2. Read/update/delete an item with a colliding ID.
3. Assert Vault B item is unaffected.

## Acceptance Criteria

- No production local-store query mutates or loads item rows by item ID alone.
- All item operations use `(vault_id, item_id)`.
- Collision tests pass.
- Global validation passes.

---

# Task 2: Make `porkpie write --stdin` and `--prompt` Mutually Exclusive

## Severity

Medium.

## Problem

The CLI command definition makes both `--stdin` and `--prompt` conflict with `value`, but not with each other.

This means a user can run:

```bash
porkpie write pie://Personal/GitHub/password --stdin --prompt
```

and the code will silently prefer stdin. That contradicts the agent claim that the flags are mutually exclusive.

## Required Fix

Update the Clap args so:

- `value` conflicts with `stdin`
- `value` conflicts with `prompt`
- `stdin` conflicts with `prompt`
- `prompt` conflicts with `stdin`

Suggested shape:

```rust
#[arg(long, conflicts_with_all = ["value", "prompt"])]
stdin: bool,

#[arg(long, conflicts_with_all = ["value", "stdin"])]
prompt: bool,
```

or use an `ArgGroup`.

## Required Tests

Add CLI parser tests for:

```text
porkpie write pie://Personal/GitHub/password literal-value
porkpie write pie://Personal/GitHub/password --stdin
porkpie write pie://Personal/GitHub/password --prompt
porkpie write pie://Personal/GitHub/password --stdin --prompt   # must fail
porkpie write pie://Personal/GitHub/password literal-value --stdin # must fail
porkpie write pie://Personal/GitHub/password literal-value --prompt # must fail
```

## Acceptance Criteria

- `--stdin --prompt` is rejected by argument parsing.
- Existing valid forms still work.
- CLI docs/help still warn that literal values can leak into shell history/process lists.
- Global validation passes.

---

# Task 3: Fix UI Navigation Honesty

## Severity

Medium.

## Problem

Docs claim navigation works between screens and is "not a single scrollable page," but `App` still renders all major page components inside one `main` block with anchor links.

That means the UI may be stateful, but the root composition is still basically a multi-section scroll page. This is a docs/code honesty problem.

## Required Decision

Pick one path.

## Path A: Make the UI truly screen-routed

Preferred.

Render only the active screen based on `AppState.screen`.

Example shape:

```rust
match state_for_render.with(|s| s.screen.clone()) {
    Screen::Onboarding => rsx!(OnboardingPage { /* props */ }),
    Screen::Unlock => rsx!(UnlockPage { /* props */ }),
    Screen::List => rsx!(ItemListPage { /* props */ }),
    Screen::NewItem => rsx!(ItemDetailPage { /* mode: New */ }),
    Screen::Detail(id) => rsx!(ItemDetailPage { /* id */ }),
    Screen::PasswordGenerator => rsx!(PasswordGeneratorPage { /* props */ }),
    Screen::ImportExport => rsx!(ImportExportPage { /* props */ }),
    Screen::Settings => rsx!(SettingsPage { /* props */ }),
}
```

Navigation buttons should update `AppState.screen`, not rely only on anchors.

If `dioxus-router` is already in dependencies and useful, wire it. Do not introduce a new frontend stack.

## Path B: Downgrade docs

If routing is not being implemented now, update docs honestly:

```text
The UI is a Dioxus single-page shell with section anchors and shared state. Full route-level screen isolation is not implemented yet.
```

Then uncheck or soften any completion-gate line saying "not a single scrollable page."

## Required Tests / Manual QA

At minimum document manual QA:

1. Start app.
2. Navigate to Onboarding.
3. Navigate to Unlock.
4. Navigate to Items.
5. Navigate to Generator.
6. Navigate to Import/Export.
7. Confirm inactive pages are not visible if Path A is chosen.

## Acceptance Criteria

- Code and docs agree.
- If completion gate says routed screens, app actually renders only active screen.
- If app remains section-based, docs say so.
- No React/TypeScript/Electron/Vite introduced.
- Global validation passes.
- WASM build passes if app.rs is changed.

---

# Task 4: Reconcile Completion Gate with Remaining Store Bug

## Severity

Medium.

## Problem

`docs/COMPLETION_GATE.md` currently says Desktop/Web, Sync, and Documentation gates pass, and that only external audit + memory zeroization block MVP.

That is not accurate while local-store item operations are still item-ID-only.

## Required Fix

After Task 1 is fixed, update docs accordingly. If Task 1 is not fixed in the same pass, immediately mark the gate partial.

Update:

```text
docs/COMPLETION_GATE.md
docs/AUDIT_REPORT.md
STATUS.md
```

The docs must mention:

- local store now uses `(vault_id, item_id)` for all item load/update/delete operations, if fixed
- or local store item scoping remains a blocker, if not fixed
- `porkpie write --stdin --prompt` mutual exclusion fix, if fixed
- UI navigation truth, either routed or section-based

## Acceptance Criteria

- Docs no longer claim only external audit/memory zeroization remain if Task 1 or Task 2 is unfixed.
- Audit report matches actual code.
- Test count is updated after final validation.
- Global validation passes.

---

# Task 5: Optional Hardening After Required Fixes

These are not blockers for this pass, but should be tracked.

## 5.1 CORS Method/Header Explicitness

Current CORS allowlist is origin-scoped, which is better than permissive. Verify browser sync/API calls that use `Authorization` and JSON POST actually get the required CORS methods/headers.

If browser clients are expected to call the sync API, configure:

- allowed methods: GET, POST
- allowed headers: Authorization, Content-Type
- avoid wildcard origins

Add tests if practical.

## 5.2 Session Secret Storage

The session file still stores the local secret key in weak/obfuscated form. This is documented, but it remains a real security gap.

Track future work:

- OS keychain support
- DPAPI on Windows
- Keychain on macOS
- Secret Service/libsecret on Linux
- option to require recovery kit/local secret key every session

## 5.3 `porkpie read` stdout Safety

`porkpie read` intentionally prints secrets. Keep it, but consider:

- `porkpie copy` as recommended default
- `--no-newline`
- warning when stdout is a terminal
- docs warning about terminal scrollback

## 5.4 Memory Zeroization Verification

Docs still list memory zeroization verification as a gap. Do not claim it is solved unless there is a meaningful test or an explicit statement that Rust heap zeroization cannot be fully verified portably.

---

# Final Acceptance for This Worklist

The agent may report this worklist complete only when:

- Task 1 is fixed with tests.
- Task 2 is fixed with tests.
- Task 3 is resolved by code or honest docs.
- Task 4 docs are updated.
- Global validation passes.
- WASM build passes if UI/app code changed.
- The final report explicitly says whether real credentials are safe.

Expected final answer for real credentials:

```text
No. Porkpie is still not safe for real credentials until external security audit is complete and the completion gate passes.
```
