# Phase 02: Remove Immediate Security Footguns

## Binding

You are bound to Phase 02 only.

Your job is to remove obvious security footguns before any new features are built on top. Do not add UI polish. Do not add sync features. Do not broaden scope.

## Goal

Eliminate dangerous defaults, raw API key storage, and raw debug output from secret-bearing types.

## Required Context

Read first:

- `docs/SECURITY_INVARIANTS.md`
- `docs/STATUS.md`
- `docs/COMPLETION_GATE.md`
- API config files
- Docker Compose files
- secret-bearing type definitions
- CLI output code paths

## Allowed Areas

- `crates/porkpie-api/**`
- `crates/porkpie-types/**`
- `crates/porkpie-cli/**` only if needed to stop debug/secret leakage
- `infra/**`
- `.env.example`
- `docs/**`
- tests touching the above

## Forbidden

- No public default API keys.
- No raw API key storage.
- No raw `Debug` derive on secret-bearing types.
- No broad `#[allow(...)]`.
- No fake hashing.
- No logging secrets.
- No changing stack or adding Electron/React/TypeScript/Vite.

## Tasks

### 1. Remove Public Default API Key Behavior

- Remove defaults such as `dev-key-change-in-production`.
- Server must fail startup if required API key config is missing.
- Docker Compose must reference env vars rather than shipping a usable default secret.
- Add `.env.example` with placeholder values only.
- Ensure `.env` is not committed.

### 2. Hash API Keys at Rest

- Do not store raw API keys.
- Store hashed API keys.
- Use safe comparison for API key validation where practical.
- Add tests proving raw API key strings are not stored.

### 3. Remove Raw Debug from Secret Types

Audit all secret-bearing structs/enums.

Remove raw `Debug` derives from anything containing or wrapping:

- passwords
- API keys
- SSH private keys
- TOTP seeds
- recovery codes
- private notes
- database credentials
- server credentials
- identity secrets

If debug output is required, implement redacted `Debug`.

Example shape:

```rust
LoginSecret {
    username: "[redacted]",
    password: "[redacted]",
}
```

Add tests proving fixture secrets do not appear in debug output.

## Acceptance Criteria

- No public default API key remains.
- API server fails clearly when required API key config is missing.
- Docker Compose does not include committed usable secrets.
- API keys are not stored raw.
- Secret-bearing types do not expose raw debug output.
- Tests prove debug output redacts fixture secrets.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test results
- API key storage design
- Debug redaction design
- Remaining risks
- Next recommended phase
