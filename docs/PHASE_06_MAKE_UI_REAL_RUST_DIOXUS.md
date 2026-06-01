# Phase 06: Make UI Real, Rust, and Dioxus

## Binding

You are bound to Phase 06 only.

Your job is to make the UI functional through real Porkpie vault logic, not static preview content. Do not introduce Electron. Do not introduce React. Do not introduce TypeScript/Vite as the product foundation. If you do, congratulations, you have failed in a very expensive way.

## Goal

Replace normal-flow static UI behavior with real Dioxus + Porkpie core behavior.

## Required Context

Read first:

- `crates/porkpie-ui/**`
- `apps/desktop/**`
- `apps/web/**`
- `crates/porkpie-core/**`
- `crates/porkpie-store/**`
- `crates/porkpie-types/**`
- `docs/STATUS.md`

## Allowed Areas

- `crates/porkpie-ui/**`
- `apps/desktop/**` only if needed for UI state wiring
- `apps/web/**` only if needed for UI state wiring
- `crates/porkpie-core/**` only if needed for UI-safe APIs
- `crates/porkpie-store/**` only if needed for local profile persistence
- tests/docs related to UI behavior

## Forbidden

- No Electron.
- No React frontend.
- No TypeScript frontend foundation.
- No Vite main app.
- No static preview items in normal app flow.
- No localStorage/sessionStorage for decrypted secrets.
- No fake controls implying unavailable features work.
- No logging decrypted items.

## Tasks

1. Remove hardcoded preview/demo items from the normal app path.
2. Demo data may exist only in an explicit demo mode.
3. Wire onboarding to real vault creation.
4. Wire unlock to real vault unlock logic.
5. Make errors conditional, not permanently rendered.
6. Wire item list to real unlocked vault state.
7. Wire item detail to real decrypted data.
8. Wire item creation to real vault item creation.
9. Wire item editing to real vault item updates.
10. Wire item deletion to real vault item deletion.
11. Wire lock to clear decrypted UI state.
12. Wire encrypted backup export/import to real logic where possible.
13. Require explicit scary confirmation for plaintext export.
14. Clearly label unavailable features as “Not implemented yet.”

## Manual QA Flow

This must work:

1. Launch UI.
2. Create vault.
3. Add login item.
4. Add SSH key item.
5. Lock vault.
6. Confirm secrets disappear.
7. Unlock vault.
8. Confirm items return.
9. Export encrypted backup.
10. Import backup into clean profile.
11. Confirm restored items decrypt.

## Acceptance Criteria

- UI is Rust/Dioxus.
- UI uses real Porkpie vault/core/store logic.
- Static preview data is not in normal flow.
- Lock clears decrypted state.
- No decrypted secrets stored in localStorage/sessionStorage.
- Manual QA flow is documented.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test/build results
- Manual QA status
- What is now functional
- What remains unavailable
- Remaining UI security risks
- Next recommended phase
