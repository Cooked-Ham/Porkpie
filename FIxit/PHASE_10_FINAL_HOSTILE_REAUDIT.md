# Phase 10: Final Hostile Reaudit

## Binding

You are bound to Phase 10 only.

Your job is to perform the final hostile QA after Phases 01-09. Do not implement broad new features. Fix only critical/high issues found during audit, then document the rest honestly.

## Goal

Verify the repo no longer has the hostile-QA blockers except genuinely external items like independent audit.

## Required Searches

Run searches for:

```text
TODO
FIXME
dev-key-change-in-production
replace-with-a-generated-secret
change-me
changeme
password
secret
private_key
api_key
master_password
local_secret_key
RecoveryKit
PlaintextExport
generated_password
plaintext
base64
Debug
println!
eprintln!
dbg!
tracing
unwrap()
expect()
Electron
React
TypeScript
Vite
localStorage
sessionStorage
CorsLayer::permissive
LastWriteWins
ON CONFLICT(id)
id TEXT PRIMARY KEY
.porkpie-session.json
porkpie-recovery-kit
```

Every hit in production code must be inspected.

## Required Manual Review Areas

- Config rejects placeholders.
- `.gitignore` covers generated sensitive files.
- RecoveryKit Debug redacts local secret key.
- PasswordGeneratorState Debug redacts generated password.
- CLI secret prompts are hidden or safe.
- `porkpie write` has `--stdin` and/or `--prompt`.
- Web behavior matches docs.
- Server item identity is vault-scoped.
- Invalid sync strategy errors.
- CORS is not permissive by default.
- README commands are accurate.
- Completion gate blockers are current and honest.
- Audit report does not contradict current code.
- No forbidden frontend stack exists.
- No production code logs secrets.

## Required Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo build --workspace --release
```

If web exists:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```

If infra exists:

```bash
docker compose -f infra/compose/docker-compose.yml --env-file infra/compose/.env.example config
```

## Required Output

Update:

```text
docs/AUDIT_REPORT.md
docs/COMPLETION_GATE.md
STATUS.md
```

Include:

- exact validation commands and results
- test count
- what changed since hostile QA
- remaining security risks
- whether real credentials are safe
- whether external audit remains required
- whether MVP gate passes

## Acceptance Criteria

- Audit docs are current.
- No stale known-bad claims remain.
- No high/critical findings remain except explicitly external audit if true.
- The repo still says not safe for real credentials unless an external audit has happened.
- Global validation command passes.

## Final Label

Use this unless every gate truly passes and an external audit has completed:

```text
Porkpie is a serious Rust prototype with real crypto and real architecture, but it is not safe for real credentials yet.
```
