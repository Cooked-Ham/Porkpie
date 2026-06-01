---
task_id: 01-workspace
task_name: Workspace and Documentation Scaffold
sequence: 1
dependencies_complete: []
estimated_duration: 2-3 hours
difficulty: Easy
blockers_resolved: none
can_parallelize: false
---

# Task 1: Workspace and Documentation Scaffold

## 🎯 Objective

Create the Rust workspace structure with all 10 crates and 10 documentation files. This is the foundation—everything else depends on it.

## ✅ Acceptance Criteria

**Workspace & Crates**
- [ ] Root `Cargo.toml` exists with workspace members
- [ ] All 10 crates exist under `crates/`:
  - [ ] porkpie-types/
  - [ ] porkpie-crypto/
  - [ ] porkpie-core/
  - [ ] porkpie-store/
  - [ ] porkpie-sync/
  - [ ] porkpie-api/
  - [ ] porkpie-cli/
  - [ ] porkpie-ui/
  - [ ] porkpie-agent/
  - [ ] porkpie-import/
- [ ] All 3 apps exist:
  - [ ] apps/desktop/
  - [ ] apps/web/
  - [ ] apps/server/
- [ ] Each crate has `Cargo.toml` and `src/lib.rs`
- [ ] `cargo metadata` works

**Documentation**
- [ ] All 10 doc files exist in `docs/`:
  - [ ] PRODUCT_SPEC.md (stub)
  - [ ] ARCHITECTURE.md (stub)
  - [ ] SECURITY_INVARIANTS.md (stub)
  - [ ] THREAT_MODEL.md (stub)
  - [ ] DATA_MODEL.md (stub)
  - [ ] SYNC_PROTOCOL.md (stub)
  - [ ] CRYPTO_FORMAT.md (stub)
  - [ ] AGENT_TASKS.md (stub)
  - [ ] TEST_PLAN.md (stub)
  - [ ] ROADMAP.md (stub)

**Code Quality**
- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes (0 warnings)
- [ ] `cargo test --workspace` passes (placeholder tests OK)
- [ ] `cargo build --workspace` succeeds

## 🔒 STRICT TYPECHECK REQUIREMENTS

**Type safety is non-negotiable.** Rust's type system is your first line of defense.

- ✓ **All type errors must compile** — `cargo build` must succeed with zero type errors
- ✓ **No `unsafe` blocks without justification** — Document why in code comment
- ✓ **No unchecked casts** — Use `as` only where necessary (document reasoning)
- ✓ **No `unwrap()` on external input** — Use `.map_err()` or `?` operator
- ✓ **No `todo!()` or `unimplemented!()` in production code** — Only in stubs
- ✓ **Compiler warnings are failures** — `cargo clippy` must have zero warnings
- ✓ **Type inference must be clear** — Add explicit types where ambiguous
- ✓ **Trait bounds must be explicit** — Don't hide requirements

**Root Documentation**
- [ ] `README.md` exists at root
- [ ] README explains: Porkpie, porkpie.love, pie://, basic usage

## 📋 Output Specification

### Directory Structure (After Task Completion)

```
porkpie/
├── Cargo.toml                          # Workspace root
├── README.md                           # Product overview
├── crates/
│   ├── porkpie-types/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── porkpie-crypto/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── porkpie-core/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── porkpie-store/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── porkpie-sync/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── porkpie-api/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── porkpie-cli/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── porkpie-ui/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── porkpie-agent/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   └── porkpie-import/
│       ├── Cargo.toml
│       ├── src/lib.rs
│       └── tests/
├── apps/
│   ├── desktop/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── web/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── server/
│       ├── Cargo.toml
│       └── src/lib.rs
├── infra/
│   ├── docker/
│   ├── compose/
│   └── caddy/
└── docs/
    ├── PRODUCT_SPEC.md
    ├── ARCHITECTURE.md
    ├── SECURITY_INVARIANTS.md
    ├── THREAT_MODEL.md
    ├── DATA_MODEL.md
    ├── SYNC_PROTOCOL.md
    ├── CRYPTO_FORMAT.md
    ├── AGENT_TASKS.md
    ├── TEST_PLAN.md
    └── ROADMAP.md
```

### Crate Cargo.toml Template

```toml
[package]
name = "porkpie-types"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
```

### README.md Contents (Minimum)

```markdown
# Porkpie

Local-first, zero-knowledge, self-hostable password manager for developers, homelab users, and small teams.

- **Domain:** porkpie.love
- **CLI:** porkpie
- **URI Scheme:** pie://
- **Built With:** Rust (Dioxus, Axum, SQLx)

## Quick Start

\`\`\`bash
cargo build --workspace
cargo test --workspace
\`\`\`

See docs/ for full architecture and design.
```

## 🔗 References

- **Architecture Doc:** See structure diagram in Architecture and Coding Plan
- **Implementation Guide:** Porkpie Implementation Plan — Section "Repository Structure"
- **Task Queue:** See Task 1 in Porkpie Rust-First Agent Task Queue

## ✔️ Success Verification

Run these commands in order. **All must succeed:**

```bash
# Build workspace
cargo build --workspace

# Format check
cargo fmt --all --check

# Lint check (zero warnings tolerated)
cargo clippy --workspace --all-targets -- -D warnings

# Test all
cargo test --workspace

# Verify metadata
cargo metadata --format-version 1 > /dev/null
```

**Expected Output:** No errors, no warnings.

## 🔒 STRICT TYPECHECK REQUIREMENTS

**Type safety is non-negotiable.** Rust's type system is your first line of defense.

- ✓ **All type errors must compile** — `cargo build` must succeed with zero type errors
- ✓ **No `unsafe` blocks without justification** — Document why in code comment
- ✓ **No unchecked casts** — Use `as` only where necessary (document reasoning)
- ✓ **No `unwrap()` on external input** — Use `.map_err()` or `?` operator
- ✓ **No `todo!()` or `unimplemented!()` in production code** — Only in stubs
- ✓ **Compiler warnings are failures** — `cargo clippy` must have zero warnings
- ✓ **Type inference must be clear** — Add explicit types where ambiguous
- ✓ **Trait bounds must be explicit** — Don't hide requirements

**Verification command:**
```bash
cargo check --workspace
cargo build --workspace
```

**If ANY type error appears, stop and fix it. Type errors = broken code.**

## 🚨 If Blocked...

| Problem | Solution |
|---------|----------|
| `cargo metadata` fails | Check Cargo.toml syntax, ensure all crates have `[package]` section |
| Clippy warnings appear | Fix warnings: check `cargo clippy` output, update code |
| Tests fail | Add placeholder test to each crate: `#[test] fn it_works() { assert!(true); }` |
| Format issues | Run `cargo fmt --all` to auto-fix |

## 📌 What Comes Next

**Task 2: Security and Architecture Documentation**

Next agent will write detailed security and architecture specs in the doc files you created. Your workspace scaffolding makes that possible.

---

**Status:** Ready for agent assignment
