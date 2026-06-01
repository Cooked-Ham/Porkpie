# Phase 01: Config and Secret Artifact Hardening

## Binding

You are bound to Phase 01 only.

Your job is to prevent dangerous demo configuration and accidental committing of Porkpie-generated secret files. Do not change crypto. Do not change UI. Do not redesign sync. Do not add features.

## Goal

Fix the mismatch where infra docs claim the placeholder API key is rejected, while the actual server accepts any non-empty API key. Also harden `.gitignore` so generated Porkpie secrets and vault files do not get committed.

## Source Findings

- `infra/compose/.env.example` includes `PORKPIE_API_KEY=replace-with-a-generated-secret` and says the server rejects this placeholder.
- `crates/porkpie-api/src/config.rs` only rejects missing or empty API keys.
- `crates/porkpie-api/src/main.rs` upserts whatever `Config::from_env()` returns.
- `.gitignore` only ignores env files and build artifacts, not Porkpie DB/session/recovery/export artifacts.
- CLI creates recovery kits in the working directory.
- CLI default session path is `.porkpie-session.json`.
- Plaintext export defaults to `porkpie-export-plaintext.json`.

## Allowed Files

- `.gitignore`
- `.env.example`
- `infra/compose/.env.example`
- `infra/compose/README.md`
- `crates/porkpie-api/src/config.rs`
- `crates/porkpie-api/src/main.rs` only if needed for startup behavior
- `crates/porkpie-api/tests/**`
- `docs/SECURITY_INVARIANTS.md`
- `docs/STATUS.md`

## Forbidden

- Do not allow any public placeholder API key to start the server.
- Do not replace the API key with a committed generated secret.
- Do not commit `.env`.
- Do not change the server to run without auth.
- Do not weaken auth tests.
- Do not move unrelated app files.

## Tasks

### 1. Reject placeholder API keys

Update `Config::from_env()` so these values are rejected:

```text
replace-with-a-generated-secret
change-me
changeme
dev
test
password
secret
porkpie
```

Also reject keys that are too short. Minimum: 32 characters.

Return a clear config error such as:

```text
PORKPIE_API_KEY must be a non-placeholder secret at least 32 characters long
```

### 2. Add config tests

Add tests proving:

- missing key is rejected
- empty key is rejected
- `replace-with-a-generated-secret` is rejected
- `change-me` is rejected
- short keys are rejected
- a long random-looking key is accepted
- `Debug` output for Config redacts the API key

### 3. Harden `.env.example`

Update examples to say:

```env
PORKPIE_API_KEY=generate-a-64-character-random-secret
```

Add a one-line generation command. Use one-line commands only.

Examples:

```bash
openssl rand -hex 32
```

or, cross-platform PowerShell:

```powershell
[Convert]::ToHexString([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(32)).ToLower()
```

### 4. Harden `.gitignore`

Add ignore patterns for generated Porkpie secret artifacts:

```gitignore
# Porkpie local vault/session artifacts
porkpie.db
*.db
*.sqlite
*.sqlite3
*.db-wal
*.db-shm
.porkpie-session.json
porkpie-recovery-kit-*.json
porkpie-backup-*.json.enc
porkpie-export-plaintext.json
*.porkpie-backup
*.porkpie-export
```

Do not ignore source-controlled `.env.example`.

### 5. Update docs

Update docs to state:

- `.env.example` will not boot a production server as-is.
- Server refuses missing, empty, short, or known-placeholder API keys.
- Recovery kits and session files are sensitive.
- Running the CLI inside a Git repo can generate sensitive files unless paths are overridden.

## Acceptance Criteria

- Placeholder API keys are rejected by code, not just docs.
- Tests cover placeholder rejection.
- `.gitignore` ignores local DB/session/recovery/export artifacts.
- Docs match actual behavior.
- Global validation command passes.
- Infra compose config renders.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

```bash
docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config
```
