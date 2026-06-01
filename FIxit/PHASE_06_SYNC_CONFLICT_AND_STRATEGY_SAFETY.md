# Phase 06: Sync Conflict and Strategy Safety

## Binding

You are bound to Phase 06 only.

Your job is to make sync strategy parsing and conflict behavior conservative and explicit. Do not redesign the sync server. Do not touch crypto. Do not add team features.

## Goal

Prevent typo-driven data loss and make conflict preservation the default posture.

## Source Findings

- Unknown CLI sync strategy strings silently become `LastWriteWins`.
- `LastWriteWins` is the default.
- A typo like `--strategy prefer-remtoe` silently accepts the most destructive strategy.
- Server conflict preservation exists, but defaults should be conservative.

## Allowed Files

- `crates/porkpie-cli/src/lib.rs`
- `crates/porkpie-cli/src/commands/sync.rs`
- `crates/porkpie-sync/src/conflict.rs`
- `crates/porkpie-sync/src/protocol.rs`
- CLI tests
- Sync tests
- `docs/SYNC_PROTOCOL.md`
- `README.md`

## Forbidden

- Do not silently coerce invalid strategies.
- Do not make LastWriteWins the hidden fallback for invalid input.
- Do not delete conflict tests.
- Do not remove existing strategies unless docs/tests are updated.

## Tasks

### 1. Make strategy parsing fallible

Change strategy parsing to return `Result<MergeStrategy, CliError>`.

Valid values:

```text
last-write-wins
prefer-local
prefer-remote
preserve-conflict
```

If `preserve-conflict` does not exist, add it or map to the existing conflict-preserving behavior.

Invalid values must error and print allowed values.

### 2. Consider defaulting to conflict preservation

Preferred:

- default strategy: `preserve-conflict`
- LastWriteWins requires explicit `--strategy last-write-wins`

If changing default is too invasive, document why and add a warning.

### 3. Add tests

Add CLI tests:

- valid strategies parse
- invalid strategy errors
- typo does not default to LastWriteWins
- default behavior is documented and tested

Add sync behavior tests:

- conflict is preserved by default
- LastWriteWins only applies when explicitly selected

### 4. Update docs

Update:

- README sync example
- `docs/SYNC_PROTOCOL.md`
- `docs/COMPLETION_GATE.md`

Docs must explain the risk of LastWriteWins.

## Acceptance Criteria

- Invalid sync strategy fails loudly.
- Typos cannot silently trigger LastWriteWins.
- Conflict-preserving behavior is default or explicitly documented.
- Tests prove parsing and conflict behavior.
- Global validation command passes.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```
