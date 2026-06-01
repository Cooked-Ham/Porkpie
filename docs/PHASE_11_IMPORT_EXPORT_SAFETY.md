# Phase 11: Import and Export Safety

## Binding

You are bound to Phase 11 only.

Your job is to make backup/import/export behavior clear and safe. Do not silently create plaintext exports. Do not hide warnings behind optimism. Optimism is not an access-control model.

## Goal

Implement safe encrypted backups, explicit dangerous plaintext export, and import paths that encrypt immediately.

## Required Context

Read first:

- `crates/porkpie-import/**`
- `crates/porkpie-cli/**`
- `crates/porkpie-core/**`
- `crates/porkpie-crypto/**`
- `docs/CRYPTO_FORMAT.md`
- `docs/STATUS.md`

## Allowed Areas

- `crates/porkpie-import/**`
- `crates/porkpie-cli/**`
- `crates/porkpie-core/**`
- `crates/porkpie-crypto/**` only if backup crypto support is missing
- docs/tests related to import/export

## Forbidden

- No plaintext export without explicit `--dangerous`.
- No silent plaintext backup.
- No imported plaintext lingering in local DB.
- No logging imported secrets.
- No pretending Bitwarden/1Password import is complete unless tested.
- No weakening crypto or storage paths.

## Tasks

1. Implement:

```bash
porkpie backup export
porkpie backup import
porkpie export plaintext --dangerous
```

2. Plaintext export must require explicit confirmation.
3. Encrypted backup roundtrip must work.
4. Imported secrets must be encrypted immediately.
5. Use `zeroize`/`secrecy` where practical for temporary plaintext buffers.
6. Add tests proving imported fixture secrets do not appear in raw DB.
7. Update docs with backup/recovery explanation.
8. Document which third-party import formats are actually supported.

## Acceptance Criteria

- Encrypted backup roundtrip works.
- Plaintext export is blocked without `--dangerous`.
- Imported secrets are encrypted at rest.
- Docs are honest about supported import formats.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test results
- Backup format
- Plaintext export safety behavior
- Import support status
- Remaining import/export risks
- Next recommended phase
