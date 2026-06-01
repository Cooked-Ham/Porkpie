# Phase 08: Docs and README Truth Pass

## Binding

You are bound to Phase 08 only.

Your job is to make docs match code. Do not add code features except tiny fixes needed to make documented commands accurate. Do not overclaim. Do not write marketing fluff. This is documentation, not a perfume ad for software.

## Goal

Remove contradictions across README, STATUS, AUDIT_REPORT, COMPLETION_GATE, and feature docs.

## Source Findings

Current doc contradictions include:

- README says web has no storage; STATUS says web uses localStorage.
- AUDIT_REPORT says no `localStorage`; code includes localStorage backend.
- AUDIT_REPORT says API key comparison uses `==`; code uses `subtle::ConstantTimeEq`.
- STATUS and AUDIT_REPORT disagree on test counts.
- README CLI basics show stale commands like `porkpie list` instead of `porkpie item list`.
- Completion gate still lists blockers beyond external audit.

## Allowed Files

- `README.md`
- `STATUS.md`
- `docs/AUDIT_REPORT.md`
- `docs/COMPLETION_GATE.md`
- `docs/SECURITY_INVARIANTS.md`
- `docs/SYNC_PROTOCOL.md`
- `docs/ROADMAP.md`
- `docs/TEST_PLAN.md`
- `docs/DATA_MODEL.md`
- `docs/ARCHITECTURE.md`
- tiny CLI help/doc fixes if needed

## Forbidden

- Do not claim production-ready.
- Do not claim real-secret safe.
- Do not claim external audit completed.
- Do not claim web storage works unless Phase 04 made it work.
- Do not invent test counts.
- Do not delete inconvenient limitations.

## Tasks

### 1. Fix README CLI examples

Replace stale commands with actual CLI shape:

```bash
porkpie init
porkpie unlock
porkpie add login
porkpie item list
porkpie item get <item-id>
porkpie read pie://Personal/GitHub/password
porkpie backup export
porkpie backup import <file>
```

Mention:

```bash
porkpie export --format plaintext --dangerous
```

only in a danger section.

### 2. Align web docs

Based on Phase 04 outcome, make all docs agree on exactly one statement:

Path A:

```text
The web build stores encrypted vault data in browser localStorage with documented origin risks.
```

Path B:

```text
The web build is a UI/generator shell and does not persist vault data.
```

### 3. Align audit status

Update `docs/AUDIT_REPORT.md` to remove stale claims:

- no longer claim API key comparison uses `==` if code uses constant-time comparison
- no longer claim no localStorage if localStorage exists
- update test counts based on actual current test run
- update remaining blockers based on current completion gate

### 4. Align completion gate

Completion gate must include all current blockers, not just external audit.

Likely blockers after earlier phases:

- external audit not completed
- session local secret key not protected by OS keychain
- web storage risk or limitation
- memory zeroization limits
- SSH-agent OpenSSH integration not complete
- third-party importers not implemented if product claims them

### 5. Add "Real Credentials" section

All major docs must keep this label until audit:

```text
Do not use Porkpie for real credentials yet.
```

### 6. Update test plan

Ensure test plan includes:

- config placeholder rejection
- generated artifact gitignore check
- RecoveryKit Debug redaction
- hidden CLI prompts/manual tests
- cross-vault item ID collision test
- invalid sync strategy test
- CORS tests
- docs consistency check

## Acceptance Criteria

- README commands match actual CLI.
- STATUS, AUDIT_REPORT, and COMPLETION_GATE agree.
- No doc claims “everything fixed except external audit” unless literally true.
- Docs keep prototype warning.
- Global validation command passes.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```
