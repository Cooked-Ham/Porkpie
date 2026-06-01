---
project: Porkpie
mission: Build production Rust password manager MVP
status: Ready for agent assignment
total_tasks: 10
estimated_duration: 15 days
sequential: true
---

# 🎯 PORKPIE AGENT MISSION INDEX

**You are the lead implementation agent for Porkpie.**

**Your mission:** Build a foundational Rust vertical slice—production code, not a disposable mockup.

---

## 📊 Project Overview

| Aspect | Details |
|--------|---------|
| **What** | Local-first, zero-knowledge, self-hostable password manager |
| **Who** | Developers, homelab users, small teams |
| **Domain** | porkpie.love |
| **Language** | Rust (production stack from day one) |
| **Duration** | ~15 days (10 sequential tasks) |
| **Difficulty** | High (security-critical code) |

---

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

---

## ⚠️ NON-NEGOTIABLE SECURITY INVARIANTS

These are fundamental requirements. Violating any is a security failure.

1. ✓ **No plaintext secret storage** — Anywhere (local DB, server DB, sync, logs, tests, backups)
2. ✓ **No master password storage** — Derived from user input on each unlock
3. ✓ **Client-side unlock only** — Server never sees master password or decrypted vault
4. ✓ **Server stores encrypted blobs only** — No server-side decryption
5. ✓ **Authenticated encryption required** — Every item encrypted with AEAD (XChaCha20Poly1305)
6. ✓ **Wrong password fails** — Never silently decrypt with wrong password
7. ✓ **Tampered ciphertext fails** — MAC must validate (authentication tag check)
8. ✓ **Locking clears memory** — All decrypted data purged on lock
9. ✓ **Plaintext export requires flag** — Must be explicit: `--dangerous-export-plaintext`
10. ✓ **No crypto shortcuts** — No base64-only, no mock crypto, no static keys, no placeholder crypto
11. ✓ **Logs sanitized** — Never include decrypted item contents
12. ✓ **No TODO in security paths** — Unimplemented crypto = immediate QA failure
13. ✓ **Unavailable features clearly marked** — UI shows "Not implemented" not fake data

**Copy these rules into every task. Verify them before declaring complete.**

---

## 📋 TASK QUEUE

### Sequence Overview

```
Phase 1: Foundation (Tasks 1-3)
  ↓
Phase 2: Cryptography (Task 4)
  ↓
Phase 3: Vault Logic (Task 5)
  ↓
Phase 4: Storage (Task 6)
  ↓
Phase 5: Interfaces (Tasks 7-8)
  ↓
Phase 6: Sync & Integration (Task 9)
  ↓
Phase 7: Polish & Completion (Task 10)
  ↓
✅ MVP COMPLETE
```

### Task List

| # | Task | File | Status |
|---|------|------|--------|
| 1 | Workspace & Documentation Scaffold | [TASK-01-workspace-scaffold.md](./TASK-01-workspace-scaffold.md) | Ready |
| 2 | Security & Architecture Documentation | [TASK-02-security-architecture-docs.md](./TASK-02-security-architecture-docs.md) | Ready |
| 3 | Core Types & Error Definitions | [TASK-03-core-types.md](./TASK-03-core-types.md) | Ready |
| 4 | Cryptographic Operations | [TASK-04-crypto.md](./TASK-04-crypto.md) | Ready |
| 5 | Vault Core Logic | [TASK-05-vault-core.md](./TASK-05-vault-core.md) | Ready |
| 6 | Local Storage (SQLx & SQLite) | [TASK-06-storage.md](./TASK-06-storage.md) | Ready |
| 7 | Command-Line Interface | [TASK-07-cli.md](./TASK-07-cli.md) | Ready |
| 8 | User Interface (Dioxus) | [TASK-08-ui.md](./TASK-08-ui.md) | Ready |
| 9 | Sync Protocol & HTTP API | [TASK-09-sync.md](./TASK-09-sync.md) | Ready |
| 10 | Import/Export & Final Validation | [TASK-10-final.md](./TASK-10-final.md) | Ready |

---

## 🚀 GETTING STARTED

### Step 1: Start Task 1

Open: **[TASK-01-workspace-scaffold.md](./TASK-01-workspace-scaffold.md)**

This task:
- Creates Rust workspace with 10 crates
- Sets up directory structure
- Creates documentation file stubs
- Verifies everything compiles

**Estimated time:** 2-3 hours

### Step 2: Follow Sequence

After completing Task 1:
- Task 2 becomes available
- Complete Task 2
- Task 3 becomes available
- And so on...

**Do not skip tasks.** Each task depends on previous ones.

### Step 3: Report Progress

After each task, provide this report:

```markdown
## Task X Complete

**Summary:** [What was built]

**Files Created/Modified:** [List changes]

**Commands Run:**
\`\`\`
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo build --workspace
\`\`\`

**Test Results:** [# passed, # failed]

**Security Notes:** [Any crypto decisions, invariant compliance]

**Known Limitations:** [What's NOT done]

**Next Task:** Task X+1 (ready for assignment)
```

---

## ❓ COMMON QUESTIONS

### What if I get stuck?

Each task has an "If Blocked..." section with decision trees. Follow them.

### What if a task has a dependency issue?

Stop and escalate. Don't work around security issues.

### What if tests fail?

Run the failing test with `--nocapture`:
```bash
cargo test --package {crate} -- --nocapture
```

Then fix the issue. No task is complete with failing tests.

### What if clippy complains?

Fix the warnings. Zero warnings tolerated. Don't suppress them.

### What about design decisions?

Make reasonable choices aligned with the architecture. Document them in commit messages.

---

## 📚 REFERENCE MATERIALS

### Key Documents

- **Porkpie Architecture and Coding Plan.md** — Full architecture specification
- **Porkpie Implementation Plan.md** — Detailed implementation roadmap
- **Porkpie Rust-First Agent Task Queue.md** — Original task descriptions (longer form)

### Task-Specific References

Each task file includes:
- 🎯 Objective (what to build)
- ✅ Acceptance Criteria (how to know when done)
- 📋 Output Specification (exact file structure)
- 🔗 References (links to relevant docs)
- ✔️ Success Verification (commands to run)
- 🚨 If Blocked (decision trees)
- 📌 What Comes Next (preview of next task)

---

## 🛠️ TECH STACK (REQUIRED)

- **Language:** Rust (primary)
- **Build:** Cargo (workspace)
- **Async:** Tokio
- **UI:** Dioxus (desktop & web)
- **Server:** Axum
- **Database:** SQLx (SQLite, Postgres support)
- **CLI:** clap
- **Crypto:** RustCrypto (Argon2, ChaCha20Poly1305)
- **Memory:** zeroize, secrecy
- **Logging:** tracing
- **Deployment:** Docker Compose

**PROHIBITED:**
- ❌ Electron
- ❌ TypeScript as foundation
- ❌ Mock crypto in production code
- ❌ Plaintext secret storage

---

## ✔️ SUCCESS CRITERIA

When all 10 tasks complete, you will have:

✅ **Functionality**
- Vault creation, unlock, lock
- 10 item types (login, API key, SSH key, note, etc.)
- Full CLI interface
- Web/Desktop UI (Dioxus)
- Sync protocol + HTTP API
- Import/export (CSV, encrypted backup)
- Docker deployment

✅ **Code Quality**
- Zero clippy warnings
- All tests passing
- Full documentation
- No `panic!()` or `unwrap()` on untrusted input
- All public APIs documented

✅ **Security**
- All 13 security invariants verified
- Encryption/decryption working
- Tampering detected
- Wrong passwords fail correctly
- Memory zeroized properly
- Logs sanitized
- No hardcoded credentials
- No secrets in git

✅ **Production Ready**
- Builds cleanly (debug & release)
- Dockerized
- Docker Compose deployment
- Tests comprehensive
- Documentation complete
- Ready for real-world use

---

## 🎓 HOW AGENTS WORK WITH THIS

**You are an AI agent implementing Porkpie.**

Each task is **self-contained** with **everything you need**:

1. Read the task file
2. Check acceptance criteria
3. Implement the code
4. Run success verification
5. Report completion
6. Move to next task

**You don't ask questions.** You make reasonable decisions and document them.

**You don't skip security.** Every security invariant is checked.

**You don't stop at "mostly done."** All acceptance criteria must be met.

---

## 📞 ESCALATION

**Stop and escalate if:**

- Any security invariant is violated
- A third-party library has a known vulnerability
- A task has architectural conflicts
- You're unsure about security-critical decisions

**Otherwise:** Keep implementing. Make decisions. Move forward.

---

## 🎉 READY TO START?

**Remember:** One task at a time. Follow the sequence. Verify everything works.

**Good luck! 🚀**

---

**Project Status:** ✅ Ready for agent assignment
**Estimated Total Effort:** 15 days (10 sequential tasks × 1.5 days avg)
**Success Probability:** High (detailed specs, clear acceptance criteria, security-first)
