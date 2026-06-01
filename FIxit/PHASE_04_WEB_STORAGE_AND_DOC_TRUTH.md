# Phase 04: Web Storage and Documentation Truth

## Binding

You are bound to Phase 04 only.

Your job is to make the web/WASM storage story true. Either wire browser storage correctly or remove the claim. Do not add React. Do not add TypeScript. Do not add Vite. Do not add Electron. Humanity has suffered enough.

## Goal

Resolve the contradiction between README, STATUS, and code regarding WASM browser vault persistence.

## Source Findings

- README says the web shell has no SQLite backend and data-bearing flows return “not available in this build.”
- STATUS says the web shell uses `localStorage` for encrypted client-side persistence.
- `vault_store.rs` contains a WASM `LocalStorage` backend.
- `App::initial_load()` currently does nothing on WASM, so the backend is not connected on startup.
- `AppState.unlocked_handle` is only compiled on non-WASM, so web may not actually hold an unlocked handle through the UI.

## Decision Required

Pick exactly one path and make code + docs agree.

## Path A: Wire WASM `localStorage` backend

Choose this if browser vault persistence is intended.

### Tasks

1. In `App::initial_load()`, connect `VaultBackend::connect_local_storage()` on WASM.
2. List vault summaries from browser storage.
3. Update `AppState` so the unlocked handle is available on WASM too, or create a target-neutral handle abstraction.
4. Ensure onboarding creates a vault in localStorage.
5. Ensure unlock works in localStorage.
6. Ensure list/detail/create/update/delete work in localStorage.
7. Ensure import/export behavior works or is honestly limited.
8. Ensure lock clears decrypted state in WASM.
9. Add WASM build validation.
10. Update README and STATUS to say browser storage uses encrypted localStorage and document its risks.

### Required warning

Document this clearly:

```text
The browser build stores encrypted vault metadata and ciphertext in localStorage. This is encrypted at rest by Porkpie's vault crypto, but localStorage is still accessible to JavaScript running on the origin. Use a dedicated trusted origin. IndexedDB/OPFS is preferred for future production work.
```

## Path B: Remove WASM storage claim

Choose this if browser storage is not intended yet.

### Tasks

1. Remove or feature-gate the `LocalStorage` backend.
2. Ensure WASM actions return clear “not available in this build” errors.
3. Update STATUS, AUDIT_REPORT, and README to consistently say web is UI shell + generator only.
4. Keep the web build working.
5. Remove completion-gate claims that web connects to a real vault store.

## Tests / Validation

If Path A:

- Add tests where possible for localStorage serialization helpers.
- Run `cargo build -p porkpie-web --target wasm32-unknown-unknown`.
- Run existing web build script if available.

If Path B:

- Add tests or compile checks proving no false web storage claim remains.
- Run `cargo build -p porkpie-web --target wasm32-unknown-unknown`.

## Acceptance Criteria

- README, STATUS, AUDIT_REPORT, and COMPLETION_GATE agree.
- Web behavior matches docs.
- No React/TypeScript/Vite/Electron introduced.
- WASM build passes.
- Global validation command passes.

## Required Validation

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```
