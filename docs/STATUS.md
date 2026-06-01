# Status

Porkpie is a foundational Rust prototype, not safe for real credentials yet.

## Implemented

- **porkpie-types**: Domain types (IDs, Items, Timestamps, 10 item type variants), error types, constants. Local secret key type with hex encoding, zeroize-on-drop, and redacted Debug. AAD builders for item and payload encryption binding. `PieUri` parser for `pie://vault/item/field` URIs. Field-level access methods (`get_field`, `set_field`, `list_fields`) on all item types.
- **porkpie-crypto**: Argon2id key derivation with local secret key support (`password || secret_key` input), XChaCha20Poly1305 authenticated encryption with associated data (AEAD), vault key wrapping, CSPRNG nonce generation.
- **porkpie-core**: Vault lifecycle (create with name + secret key + recovery kit, unlock requiring password + secret key, lock), item CRUD with AAD-bound encryption, password generator, memory zeroization.
- **porkpie-store**: SQLite persistence via SQLx, schema migrations with vault name support, encrypted-only item storage, concurrent access support. End-to-end plaintext proof tests verify no fixture secrets in raw SQLite bytes. Vault lookup by name.
- **porkpie-sync**: Sync protocol types, revision-based merge, conflict detection, LastWriteWins/PreferLocal/PreferRemote strategies.
- **porkpie-api**: Axum HTTP sync server, bearer-token auth with SHA-256 hashed API key storage, encrypted blob storage, conflict detection, audit logging. Server fails startup if `API_KEY` env var is missing.
- **porkpie-cli**: URI-first secret operations with `pie://` as canonical interface. Commands: `init` (with vault name + recovery kit), `unlock`, `lock`, `item list` (redacted), `item get` (redacted), `read <pie-uri>` (field reveal), `write <pie-uri> <value>` (field update), `copy <pie-uri>` (clipboard), `run --env NAME=pie://... -- <cmd>` (env injection), `add`, `edit`, `delete`, `export`, `import`, `sync`. Session management with local secret key.
- **porkpie-import**: CSV import, encrypted backup export/import with duplicate handling, wrong-password rejection, and secret key requirement.
- **porkpie-ui (Phase 06)**: Dioxus 0.4 UI is now wired to real vault operations end-to-end. `VaultBackend` enum dispatches to a SQLite-backed implementation (desktop/server) and an `Unavailable` stub for pure WASM. `UnlockedVaultHandle` clones via `Arc<Mutex<Vault>>` so a single unlocked vault can be shared across the page tree. The `App` component owns both a `UseRef<VaultBackend>` and a `UseRef<AppState>`, runs `initial_load` to connect SQLite and list vaults, and routes to Onboarding / Unlock / List / Detail / NewItem / PasswordGenerator / ImportExport / Settings. Onboarding creates a real vault and surfaces the recovery kit; Unlock requires password + 64-hex local secret key; List shows redacted item metadata with search, lock, and refresh; Detail is a type-aware form for all 10 item types, with save (create or update), copy-password, and delete-confirmation modal. PasswordGenerator calls `porkpie_core::generate_password` and copies the result. ImportExport performs real `porkpie_import::export_backup_file` and `import_backup` round-trips, with plaintext export gated behind a destructive-styled `Modal` confirm. Settings exposes lock timeout, theme, and a Lock button. No decrypted secret material is written to localStorage, sessionStorage, or any client-side cache. `BackupImportMode` gained a `Default` (SkipDuplicates) impl.

## Partial / Scaffolded

- **porkpie-agent**: Single `AgentSchedule` struct for periodic scheduling. No background worker, no sync scheduler, no runtime.

## Stubs / Empty Shells

- **apps/desktop**: Re-exports `porkpie_ui::App`. No window creation, no native menus, no system tray, no binary target.
- **apps/web**: Re-exports `porkpie_ui::App`. No WASM target, no HTML template, no bundler config.
- **apps/server**: Re-exports `porkpie_api::{build_router, AppState}`. The actual server binary lives in `crates/porkpie-api`.

## Not Implemented

- Desktop shell integration (system tray, global hotkeys, clipboard auto-clearing).
- Browser extension / autofill.
- SSH agent support.
- Recovery code workflows.
- Team sharing / public-key recipient wrapping.
- Hardware-backed key support (FIDO2/WebAuthn/YubiKey).
- Multi-device sync client configuration.
- Third-party importers (1Password, Bitwarden, LastPass native formats).
- Security audit or penetration testing.

## Unsafe / Unverified

### Security Audit

- No security audit or penetration testing has been performed.
- No hostile QA pass has been done.

### CLI Secret Exposure

- CLI `item list` and `item get` commands are redacted by default, showing only metadata (ID, type) without decrypting or displaying secret fields. **Breaking change**: the old top-level `list` and `get` commands were moved under the `item` subcommand.
- Secret field access requires explicit `porkpie read pie://vault/item/field` command.
- `porkpie write pie://vault/item/field <value>` updates only the specified field.
- `porkpie copy pie://vault/item/field` copies to clipboard without printing to stdout. May not work in headless environments without a display server.
- `porkpie run --env NAME=pie://vault/item/field -- <cmd>` injects secrets into child process environment without exposing them in the parent shell. Child processes could still log or leak these values.
- Invalid `pie://` URIs fail cleanly without leaking parsed values in error messages.
- `porkpie read` prints field values to stdout, which could be captured in shell history or terminal scrollback. No `--no-echo` or TTY detection is implemented.
- `porkpie edit` still decrypts the full item for interactive editing. This is necessary for the current UX but exposes all fields in memory during the edit session.
- Item name lookup in `pie://` URIs requires decrypting all items in the vault to match titles. This is correct for security but inefficient for large vaults.
- No `porkpie export plaintext --dangerous` command exists yet (deferred to Phase 11).

### Session File

- The session file (`.porkpie-session.json`) stores the local secret key hex on disk. This is by design for convenience but means local machine compromise exposes the secret key. An attacker with both the session file and the master password can unlock the vault.
- Session file is not encrypted at rest.
- Commands that use the old session-based unlock flow (`export`, `import`, `sync`) still prompt for the master password but rely on the session file for the secret key. This is consistent with the new model but means the session file remains a high-value target.

### Crypto Parameters

- Argon2id parameters (time_cost=2, mem_cost=19456 KiB, parallelism=1) are conservative defaults. Production deployments may want higher values for stronger brute-force resistance.
- No key rotation mechanism exists. If a vault key is compromised, there is no way to rotate it without creating a new vault.
- API key hash comparison uses `==` (not constant-time). Since we compare SHA-256 hashes rather than raw keys, the timing side-channel is minimal but not eliminated.

### Logging and Output

- No audit of log output has been performed to verify no secrets leak via `tracing`, `println!`, or error messages.
- Error messages may contain vault IDs or item IDs, which are non-secret but could aid an attacker in targeting specific data.

### UI

- **Phase 06 makes the UI real**: pages, forms, and dialogs are wired to live vault I/O, not mock data. Decrypted item material is only ever held in `AppState::current_item` (an `Option<DecryptedItem>`) while the user is actively viewing or editing a single item, and is cleared on lock, on screen change, and on logout. The unlock form, onboarding form, import form, and detail form all reflect real state via `use_state`/`use_ref`. The plaintext export is gated by a `Modal` confirm with a destructive button style, but the in-app "I understand" affordance is a soft check rather than a typed-phrase challenge; treat it as UI safety, not as a strong guarantee. Theme switching is wired to state but does not yet re-render the live CSS variables — the Settings page is honest about the gap and tells the user to restart. The web build is browser-only and the desktop shell (`apps/desktop`) does not yet produce a binary, so end-to-end QA for Phase 06 is "manual QA via `cargo test` + the in-progress desktop harness" — the WASM target is not yet bundled, and the desktop binary is not yet shipped (deferred to Phase 07). `apps/web` and `apps/desktop` are re-exports only. No router: navigation is `UseState<Screen>` switches inside the root `App` component.

### API Server

- Docker Compose files reference `API_KEY` via environment variable substitution, but no `.env` file validation or secret generation tooling is provided. Users must manually generate and manage API keys.

## Build Status

The workspace passes:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

107 tests pass across all crates. After Phase 06, **108 tests pass** — 3 new tests in `crates/porkpie-ui/tests/vault_store.rs` exercise a full in-memory SQLite round trip: vault create → duplicate name rejection → unlock with correct password+secret key → wrong-password rejection → list items → create item → get item → update item → delete item → lock.

### Test Coverage Highlights

- **Crypto**: 11 tests covering key derivation with secret key, AAD binding (wrong vault ID, wrong item ID, wrong schema version), tampering detection, nonce uniqueness.
- **Vault lifecycle**: 7 tests covering create, unlock with correct password + secret key, unlock failure with wrong password, unlock failure with wrong secret key, password-alone-cannot-unlock.
- **Debug redaction**: 10 tests proving fixture secrets never appear in Debug output for all 10 item type variants, Vault, and Item.
- **API key security**: 3 tests proving raw keys are not stored in the database, hash comparison works, different keys produce different hashes.
- **Plaintext proof**: 2 end-to-end tests scanning raw SQLite + WAL + SHM bytes to verify no fixture secrets (login password, API key, SSH private key, secure note, database password, recovery code) are persisted.
- **PieUri**: 9 tests covering URI parsing, validation, display, redacted display, and error messages that don't leak values.
- **Field access**: 7 tests covering get/set operations on Login, APIKey, SSHKey, Database, and Custom item types.
- **CLI parsing**: 6 tests covering help text, invalid args, version output, global options, sync options, and session state.
- **UI state**: 3 tests covering `AppState::lock()` clearing the current vault + items, lock-timeout detection only applying to unlocked sessions, and `Screen` round-tripping for all 7 screens.
- **UI components**: 3 tests covering item-list filter, master-password validation, and password-generator controls.
- **UI vault store (new, Phase 06)**: 3 tests covering `VaultBackend::Unavailable` reporting, full create+unlock+CRUD+lock round trip on an in-memory SQLite database, and rejection of wrong passwords. These tests run only on non-WASM targets; the WASM build short-circuits via `VaultStoreError::Unavailable`.

## Manual QA Flow (Phase 06)

The Phase 06 acceptance criterion is a working end-to-end flow. The `cargo test` round-trip is the automated proof, but the user-visible flow is:

1. Launch the desktop binary (or the WASM web build once Phase 07 ships it). On first launch, the Onboarding screen is shown with no vaults present.
2. Enter a vault name (e.g. `Personal`) and a master password (≥16 chars). The local secret key is auto-generated and the recovery kit is displayed. Save the recovery kit to an offline location.
3. The app routes to the List screen, which is empty. Use the **+** button to add an item. The Detail form is type-aware: pick a type, fill the fields, save.
4. The saved item appears in the list with redacted metadata. Open it to reveal the field values, copy the password via the **Copy password** button, edit and save, or delete via the delete button (with confirm modal).
5. From Settings, click **Lock vault**. The app clears `AppState::unlocked_handle`, `current_vault`, `items`, and `current_item`, then routes to the Unlock screen. Re-enter the password and secret key (or paste from the recovery kit) to unlock again.
6. From Import/Export, use **Encrypted export** to download a backup JSON (real `porkpie_import::export_backup_file`). Use **Encrypted import** to paste a backup JSON and round-trip it through `porkpie_import::import_backup` + `handle.import_encrypted_with_keys`. The plaintext export button is gated by a destructive-styled confirm modal — it must be clicked twice with explicit acknowledgement.
7. In a build without a database backend (pure WASM), every write path returns `VaultStoreError::Unavailable` and the UI displays a "not available in this build" notice instead of failing silently.

The `porkpie-ui` crate's `vault_store.rs` is the single source of truth for vault I/O; pages never call `porkpie_store` or `porkpie_core` directly. Decrypted material is held in `Option<DecryptedItem>` and only while the relevant screen is active.
