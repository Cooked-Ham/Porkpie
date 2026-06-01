# Phase 01: Make CI and Status Honest

## Binding

You are bound to Phase 01 only.

Your job is to make the repository build cleanly and stop overclaiming its completion status. Do not add product features. Do not redesign architecture. Do not touch UI behavior except where docs or status references require it.

## Goal

Make the repo pass its quality gate and honestly describe Porkpie as a foundational Rust prototype, not a complete password manager.

## Required Context

Read first:

- `README.md`
- `docs/STATUS.md` if present
- `docs/ROADMAP.md` if present
- `docs/COMPLETION_GATE.md` if present
- `.gitlab-ci.yml`
- any files under `tasks/` or `docs/` claiming final/completed/production status

## Allowed Areas

- `README.md`
- `docs/**`
- `.gitlab-ci.yml` only if CI commands are inconsistent with the required validation gate
- Rust source files only to fix Clippy failures

## Forbidden

- Do not add new product features.
- Do not suppress Clippy broadly.
- Do not weaken tests.
- Do not claim production readiness.
- Do not remove warnings by deleting meaningful code.
- Do not add Electron, React, TypeScript, or Vite.

## Tasks

1. Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

2. Fix all Clippy failures without broad `#[allow(...)]`.

3. Run:

```bash
cargo fmt --all
cargo test --workspace
cargo build --workspace
```

4. Update README to clearly say:

```text
Porkpie is a foundational Rust prototype, not safe for real credentials yet.
```

5. Add or update:

```text
docs/STATUS.md
docs/COMPLETION_GATE.md
```

6. Remove or rewrite claims like:

- complete
- production-ready
- final
- safe for real credentials
- secure password manager ready for real use

7. Add a README warning near the top:

```text
Do not use Porkpie with real credentials yet. This is a prototype pending hardening and security review.
```

## Acceptance Criteria

- Global validation command passes.
- README clearly warns against real-secret use.
- `docs/STATUS.md` describes implemented, partial, mocked/static, planned, and unsafe areas.
- `docs/COMPLETION_GATE.md` defines gates before Porkpie can be called an MVP.
- No misleading completion claims remain.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Full validation output summary
- Any Clippy failures fixed
- Remaining docs/status risks
- Next recommended phase
