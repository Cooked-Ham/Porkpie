# Phase 05: Fix CLI Around `pie://`

## Binding

You are bound to Phase 05 only.

Your job is to make Porkpie CLI safe by default and centered around the correct `pie://` model. Do not design a verbose `secret read` command tree. That was wrong. Bury it.

## Goal

Implement URI-first secret operations and prevent accidental secret dumping from normal item commands.

## Required Context

Read first:

- `crates/porkpie-cli/**`
- `crates/porkpie-core/**`
- `crates/porkpie-types/**`
- `docs/STATUS.md`
- any existing CLI docs

## Allowed Areas

- `crates/porkpie-cli/**`
- `crates/porkpie-core/**` only if needed for field-level access
- `crates/porkpie-types/**` only if needed for URI parsing/types
- tests for CLI/core
- README/docs CLI sections

## Forbidden

- Do not implement `porkpie secret read`.
- Do not dump whole decrypted items by default.
- Do not print passwords/API keys/private keys/TOTP seeds/recovery codes/notes unless explicitly requested via `porkpie read <pie-uri>`.
- Do not leak secrets in error messages.
- Do not log secret values.
- Do not add Electron/React/TypeScript/Vite.

## Canonical URI Format

```text
pie://Vault/Item/field
```

Examples:

```text
pie://Personal/GitHub/password
pie://Homelab/server1/ssh_private_key
pie://Infrastructure/Cloudflare/api_token
pie://Dev/Postgres/connection_string
```

## Required Commands

```text
porkpie read <pie-uri>
porkpie write <pie-uri>
porkpie copy <pie-uri>
porkpie run --env NAME=<pie-uri> -- <command>
porkpie item list
porkpie item get <item-id-or-name>
porkpie backup export
porkpie backup import
porkpie export plaintext --dangerous
```

## Behavior Rules

- `porkpie item list` is redacted.
- `porkpie item get` is redacted by default.
- `porkpie read <pie-uri>` reveals only the requested field.
- `porkpie write <pie-uri>` updates only the requested field.
- `porkpie copy <pie-uri>` copies without printing where platform support allows.
- `porkpie run --env NAME=<pie-uri> -- <command>` injects secrets into child process env.
- Plaintext export requires `--dangerous` and explicit confirmation.

## Tests

Add tests proving:

- normal item list output does not contain fixture secrets
- normal item get output does not contain fixture secrets
- `read` reveals only the requested field
- `write` changes only the requested field
- `run --env` injects env var without dumping it to stdout
- invalid `pie://` URIs fail cleanly without leaking values

## Acceptance Criteria

- CLI uses `pie://` as canonical secret reference.
- No `porkpie secret read` command exists.
- Item list/get do not dump secrets by default.
- Secret reveal requires explicit `porkpie read pie://...`.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test results
- Implemented CLI commands
- Redaction behavior
- Remaining CLI risks
- Next recommended phase
