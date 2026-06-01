# Phase 04: Prove Local Storage Does Not Leak Plaintext

## Binding

You are bound to Phase 04 only.

Your job is to add proof tests that the real core-to-store path does not persist plaintext secrets. Do not replace the storage engine. Do not add UI. Do not skip the raw SQLite scan because “it should be encrypted.” That phrase has betrayed civilizations.

## Goal

Create end-to-end tests showing persisted SQLite data does not contain fixture secrets.

## Required Context

Read first:

- `crates/porkpie-store/**`
- `crates/porkpie-core/**`
- `crates/porkpie-crypto/**`
- `crates/porkpie-types/**`
- existing store tests

## Allowed Areas

- `crates/porkpie-store/**`
- `crates/porkpie-core/**` only if needed for testability
- `crates/porkpie-types/**` only if needed for item creation
- tests
- `docs/SECURITY_INVARIANTS.md` if documenting the new proof test

## Forbidden

- No test-only fake encryption.
- No storing plaintext then deleting it before scan.
- No weakening storage tests.
- No claiming lower-level arbitrary-byte persistence tests prove encryption.
- No logging fixture secrets.

## Fixture Secrets

Use these exact markers or similarly obvious markers:

```text
DO_NOT_LEAK_PASSWORD_123
DO_NOT_LEAK_API_KEY_123
DO_NOT_LEAK_PRIVATE_KEY_123
DO_NOT_LEAK_NOTE_123
DO_NOT_LEAK_DATABASE_PASSWORD_123
DO_NOT_LEAK_RECOVERY_CODE_123
```

## Tasks

1. Create a vault through real core logic.
2. Add item types containing fixture secrets:
   - login password
   - API key
   - SSH private key
   - secure note body
   - database password
   - recovery code
3. Persist through the real store layer.
4. Read raw SQLite file bytes.
5. Assert every fixture marker is absent.
6. Keep lower-level store byte tests separate.

## Acceptance Criteria

- Raw SQLite file does not contain fixture secrets.
- Test fails if plaintext is accidentally stored.
- Test uses real core/store path, not fake ciphertext.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test results
- Fixture markers checked
- Raw storage scan method
- Remaining storage risks
- Next recommended phase
