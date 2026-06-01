# Phase 12: Final Hostile QA Pass

## Binding

You are bound to Phase 12 only.

Your job is to audit the repository before the next milestone claim. Do not add major features unless required to fix a critical QA failure. Be hostile. Assume previous agents were trying to sneak unfinished work past you, because statistically they were.

## Goal

Verify the repo honestly meets its current milestone and produce an audit report.

## Required Context

Read all relevant top-level docs and inspect the codebase.

Required docs:

- `README.md`
- `docs/STATUS.md`
- `docs/COMPLETION_GATE.md`
- `docs/SECURITY_INVARIANTS.md`
- `docs/CRYPTO_FORMAT.md`
- `docs/SYNC_PROTOCOL.md`
- `docs/ROADMAP.md`

## Allowed Areas

- Any file needed to fix critical/high issues found during QA
- `docs/AUDIT_REPORT.md`
- tests
- docs that overclaim

## Forbidden

- Do not claim complete unless completion gate passes.
- Do not ignore failed validation.
- Do not hide TODO/FIXME in critical paths.
- Do not suppress warnings.
- Do not remove tests to pass.
- Do not approve real-secret use without external security review.

## Required Validation Command

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

## Required Searches

Search the repo for:

```text
TODO
FIXME
dev-key-change-in-production
password
secret
private_key
api_key
master_password
plaintext
base64
Debug
println!
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
```

These hits are not automatically bugs. Every hit in a critical path must be inspected.

## Required Review Areas

- CI status
- docs honesty
- crypto implementation
- local secret key model
- recovery kit
- associated-data binding
- local SQLite plaintext leak tests
- CLI redaction and `pie://` behavior
- Dioxus UI functionality
- desktop launch
- web launch
- bidirectional sync
- conflict handling
- SSH-agent status honesty
- API payload rejection
- API auth safety
- import/export safety

## Required Output

Create or update:

```text
docs/AUDIT_REPORT.md
```

Include:

- implemented features
- partial features
- mocked/static features
- unsafe areas
- security issues fixed
- security issues remaining
- exact validation commands and results
- whether real credentials are safe to use
- next recommended phase

## Acceptance Criteria

- Audit report exists.
- Global validation command passes, or failures are documented with severity and file paths.
- Remaining risks are documented honestly.
- Project remains labeled prototype unless every completion gate passes.
- No false claim that Porkpie is production-ready.

## Final Safety Label

Unless external security review has happened and every completion gate passes, the safe label remains:

```text
Porkpie: foundational Rust prototype, not safe for real credentials yet.
```
