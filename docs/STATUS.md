# Status

Porkpie is a foundational Rust prototype, not safe for real credentials yet.

## Implemented

- **porkpie-types**: Domain types (IDs, Items, Timestamps, 10 item type variants), error types, constants. Local secret key type with hex encoding, zeroize-on-drop, and redacted Debug. AAD builders for item and payload encryption binding. `PieUri` parser for `pie://vault/item/field` URIs. Field-level access methods (`get_field`, `set_field`, `list_fields`) on all item types.
- **porkpie-crypto**: Argon2id key derivation with local secret key support (`password || secret_key` input), XChaCha20Poly1305 authenticated encryption with associated data (AEAD), vault key wrapping, CSPRNG nonce generation.
- **porkpie-core**: Vault lifecycle (create with name + secret key + recovery kit, unlock requiring password + secret key, lock), item CRUD with AAD-bound encryption, password generator, memory zeroization.
- **porkpie-store**: SQLite persistence via SQLx, schema migrations with vault name support, encrypted-only item storage, concurrent access support. End-to-end plaintext proof tests verify no fixture secrets in raw SQLite bytes. Vault lookup by name. Sync state persistence (`load_sync_state`, `save_sync_state`). Item operations with explicit revision control (`load_item_records`, `load_items_with_type_since`, `upsert_item_revision`).
- **porkpie-sync (Phase 08)**: Sync is now a real bidirectional protocol. The `porkpie-sync` crate provides types (`EncryptedSyncItem`, `SyncRequest`, `SyncResponse`, `SyncOutcome`, `SyncCursor`, `ConflictItem`), conflict detection (`detect_conflicts`), merge strategies (`LastWriteWins`, `PreferLocal`, `PreferRemote`), and an orchestration function (`sync_vault`). The `porkpie-api` crate exposes `POST /api/v1/sync/register` (vault metadata registration with real salt/wrapped-key), `POST /api/v1/sync/begin` (pull encrypted changes after a cursor revision), `POST /api/v1/sync/push` (push encrypted items with conflict detection returning HTTP 409), and `GET /api/v1/vault/{vault_id}` (fetch encrypted vault metadata for peer reconstruction). The `porkpie-cli` `sync` command performs the full bidirectional flow: register vault, load local sync cursor, pull encrypted changes from server, decrypt and merge items into the unlocked vault, re-encrypt and re-persist all vault items with bumped revisions, push locally-changed encrypted items to server, and persist the updated cursor. Conflicts are preserved — when Profile B pushes an item that Profile A already edited, the server returns `409 Conflict` with `ConflictItem[]` payload carrying both sides' revisions and ciphertext. The `--strategy` flag (`last-write-wins`, `prefer-local`, `prefer-remote`) controls push conflict behaviour. The `porkpie-store` crate gained `load_sync_state`/`save_sync_state` for the client-side sync cursor, `load_item_records`/`load_items_with_type_since`/`upsert_item_revision` for item operations with explicit revision control. The `porkpie-types` crate gained `ItemType::type_label()` for mapping item variants to their string label. A new `bidirectional_sync.rs` integration test in `porkpie-api` exercises the full two-profile flow: Profile A creates vault + item, registers + pushes, Profile B pulls + decrypts + verifies content, both profiles edit the same item, Profile A pushes successfully, Profile B gets a 409 conflict with preserved item metadata, and the server database is verified to contain no fixture plaintext. All ciphertext is opaque binary; the server never receives master passwords, vault keys, or plaintext item fields. Conflict data carries encrypted blobs (server_data) that the peer can decrypt with the correct vault key.
- **porkpie-api (Phase 08)**: Axum HTTP sync server with `register`, `begin`, `push`, and `vault-metadata` endpoints. Bearer-token auth with SHA-256 hashed API key storage. Encrypted blob storage. Push conflict detection returning HTTP 409 with `ConflictItem[]` payload. Server-db plaintext scan proof in the bidirectional sync test. Server fails startup if `API_KEY` env var is missing.
- **porkpie-cli**: URI-first secret operations with `pie://` as canonical interface. Commands: `init` (with vault name + recovery kit), `unlock`, `lock`, `item list` (redacted), `item get` (redacted), `read <pie-uri>` (field reveal), `write <pie-uri> <value>` (field update), `copy <pie-uri>` (clipboard), `run --env NAME=pie://... -- <cmd>` (env injection), `add`, `edit`, `delete`, `export`, `import`, `sync` (full bidirectional sync with `--server`, `--api-key`, `--strategy` flags, vault registration, pull/merge, push, and local sync cursor persistence). Session management with local secret key.
- **porkpie-import**: CSV import, encrypted backup export/import with duplicate handling, wrong-password rejection, and secret key requirement.
- **porkpie-ui (Phase 06)**: Dioxus 0.4 UI is now wired to real vault operations end-to-end. `VaultBackend` enum dispatches to a SQLite-backed implementation (desktop/server) and an `Unavailable` stub for pure WASM. `UnlockedVaultHandle` clones via `Arc<Mutex<Vault>>` so a single unlocked vault can be shared across the page tree. The `App` component owns both a `UseRef<VaultBackend>` and a `UseRef<AppState>`, runs `initial_load` to connect SQLite and list vaults, and routes to Onboarding / Unlock / List / Detail / NewItem / PasswordGenerator / ImportExport / Settings. Onboarding creates a real vault and surfaces the recovery kit; Unlock requires password + 64-hex local secret key; List shows redacted item metadata with search, lock, and refresh; Detail is a type-aware form for all 10 item types, with save (create or update), copy-password, and delete-confirmation modal. PasswordGenerator calls `porkpie_core::generate_password` and copies the result. ImportExport performs real `porkpie_import::export_backup_file` and `import_backup` round-trips, with plaintext export gated behind a destructive-styled `Modal` confirm. Settings exposes lock timeout, theme, and a Lock button. No decrypted secret material is written to localStorage, sessionStorage, or any client-side cache. `BackupImportMode` gained a `Default` (SkipDuplicates) impl.
- **porkpie-ui WASM target (Phase 07)**: The shared Dioxus UI now compiles cleanly for `wasm32-unknown-unknown`. Vault I/O methods that have no meaning in a browser (`unlock_vault`, `create_vault`) have cfg-gated WASM stubs that always return `VaultStoreError::Unavailable`, and the dependent UI paths (onboarding, unlock, import/export, list, detail) gate the actual storage call on `cfg(not(target_arch = "wasm32"))` and show "not available in this build" notices in the browser. The `porkpie-core` password generator runs unchanged on WASM. The shared CSS, Onboarding / Unlock / List / Detail / NewItem / PasswordGenerator / ImportExport / Settings pages, and the navigation sidebar all render in the browser. The shared `porkpie-ui` crate is a single source of truth for both shells.
- **apps/desktop (Phase 07)**: The desktop shell is now a real `porkpie-desktop` binary that calls `dioxus_desktop::launch_cfg` with a `LaunchConfig` that derives a window title, width, height, and SQLite URL from environment variables. The default SQLite location is the platform data directory (`%APPDATA%\Porkpie\porkpie.db` on Windows, `~/Library/Application Support/Porkpie/porkpie.db` on macOS, `$XDG_DATA_HOME/porkpie/porkpie.db` on Linux). `cargo run -p porkpie-desktop` produces a working window that renders the shared UI on top of the real SQLite-backed vault store.
- **apps/web (Phase 07)**: The web shell is now a real `porkpie-web` binary that calls `dioxus_web::launch_cfg` with a `LaunchConfig` driven by `PORKPIE_WEB_ROOT`. The shell ships an `apps/web/index.html` template that loads the `wasm-bindgen`-produced `porkpie-web.js` as an ES module and a `apps/web/build-web.ps1` script that compiles the binary for `wasm32-unknown-unknown`, runs `wasm-bindgen --target web`, and copies the `index.html` template into a `dist-web/` directory alongside the snippets. The release WASM artifact is ~815 KB; the debug artifact is ~3.5 MB. The script also has a `-Serve` mode that boots a small static HTTP server on the bundle for end-to-end smoke tests, and a `-Clean` flag that wipes the output. A `test-web.ps1` script fetches `/index.html`, `/porkpie-web.js`, `/porkpie-web_bg.wasm`, and the wasm-bindgen snippet over HTTP and asserts the bundle is well-formed. `cargo build -p porkpie-web --target wasm32-unknown-unknown` succeeds with no warnings.

## Partial / Scaffolded

- **porkpie-agent**: Single `AgentSchedule` struct for periodic scheduling. No background worker, no sync scheduler, no runtime.
- **apps/server**: Re-exports `porkpie_api::{build_router, AppState}`. The actual server binary lives in `crates/porkpie-api`. A standalone launch command is not yet wired in this crate.
- **apps/web clipboard + persistence**: The web shell renders the full UI surface but the `VaultBackend::Unavailable` path means the browser shell cannot yet persist items or copy values to the clipboard. The CSS, navigation, and password generator (which is pure Rust) all work in the browser; the data-bearing screens report "not available in this build". A future phase would add a JS-side IndexedDB or `localStorage` bridge that the `VaultBackend::Unavailable` path can be wired through.

## Stubs / Empty Shells

None as of Phase 07. The `apps/desktop` and `apps/web` shell crates are no longer re-exports — both have real binary targets and launch code paths.

## Not Implemented

- Desktop shell integration beyond launch: no system tray, no global hotkeys, no clipboard auto-clearing.
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

- **Phase 06 made the UI real**: pages, forms, and dialogs are wired to live vault I/O, not mock data. Decrypted item material is only ever held in `AppState::current_item` (an `Option<DecryptedItem>`) while the user is actively viewing or editing a single item, and is cleared on lock, on screen change, and on logout. The unlock form, onboarding form, import form, and detail form all reflect real state via `use_state`/`use_ref`. The plaintext export is gated by a `Modal` confirm with a destructive button style, but the in-app "I understand" affordance is a soft check rather than a typed-phrase challenge; treat it as UI safety, not as a strong guarantee. Theme switching is wired to state but does not yet re-render the live CSS variables — the Settings page is honest about the gap and tells the user to restart.
- **Phase 07 launches the UI**: `cargo run -p porkpie-desktop` opens a WebView window that renders the real `porkpie-ui` app on top of a real SQLite-backed vault store. The web shell `cargo build -p porkpie-web --target wasm32-unknown-unknown` followed by `wasm-bindgen --target web` produces a static `dist-web/` bundle (`index.html` + `porkpie-web.js` + `porkpie-web_bg.wasm` + snippets) that any HTTP server can serve. `pwsh apps/web/build-web.ps1 -Serve` automates the build + serve path. The web shell has no SQLite in the browser, so data-bearing flows return `VaultStoreError::Unavailable` and the UI shows "not available in this build" notices — this is the documented web shell mode, not a regression. Real client-side persistence would need an IndexedDB or `localStorage` bridge; that is a separate phase. The web build is Rust-only at runtime: no Electron, no React, no TypeScript, no Vite. The desktop binary is 14.6 MB; the release WASM bundle is 815 KB.

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

108 tests pass across all crates. After Phase 08, **109 tests pass** — the 108 previous tests plus a new `bidirectional_sync_with_conflict_preservation` integration test in `porkpie-api/tests/bidirectional_sync.rs` that exercises the full two-profile sync flow: vault register, push, pull, decrypt, offline edits, conflict detection via HTTP 409, conflict assertion with correct item metadata, and a server-DB plaintext scan proof. The web shell additionally passes:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```

and the `pwsh apps/web/build-web.ps1` + `pwsh apps/web/test-web.ps1` end-to-end smoke test. After Phase 07, the **workspace tests still pass at 108** — Phase 07 added no new unit tests because the launchers are exercised by `cargo build --target wasm32-unknown-unknown` (for the web shell) and by smoke-running the desktop binary in CI.

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
- **UI vault store (Phase 06)**: 3 tests covering `VaultBackend::Unavailable` reporting, full create+unlock+CRUD+lock round trip on an in-memory SQLite database, and rejection of wrong passwords. These tests run only on non-WASM targets; the WASM build short-circuits via `VaultStoreError::Unavailable`.
- **API sync (Phase 08)**: 5 existing API tests (health, missing auth, unknown vault, push+begin round trip, conflict detection) plus a new bidirectional sync integration test that exercises vault registration, profile A push, profile B pull + decrypt, offline edits on both sides, conflict detection via HTTP 409, and a server-DB plaintext scan proof (109 total).

## Manual QA Flow (Phase 07)

The Phase 06 acceptance criterion is a working end-to-end flow. The Phase 07 acceptance criterion is that this flow is reachable from a binary launch, not just from `cargo test`. The `cargo test` round-trip is the automated proof, but the user-visible flows are:

1. **Desktop**: `cargo run -p porkpie-desktop` opens a WebView2 window (WebKitGTK on Linux, WebKit on macOS). On first launch, the Onboarding screen is shown with no vaults present. Enter a vault name (e.g. `Personal`) and a master password (≥16 chars). The local secret key is auto-generated and the recovery kit is displayed. Save the recovery kit to an offline location. The app routes to the List screen, which is empty. Use the **+** button to add an item. The Detail form is type-aware: pick a type, fill the fields, save. The saved item appears in the list with redacted metadata. Open it to reveal the field values, copy the password via the **Copy password** button, edit and save, or delete via the delete button (with confirm modal). From Settings, click **Lock vault**. Re-enter the password and secret key (or paste from the recovery kit) to unlock again. The default SQLite location is the platform data directory; override with `PORKPIE_DATABASE_URL` or `PORKPIE_DATA_DIR`. The default window size is 1180x820; override with `PORKPIE_WINDOW_WIDTH`, `PORKPIE_WINDOW_HEIGHT`, or `PORKPIE_WINDOW_TITLE`.
2. **Web (release build)**: From the repo root, run `pwsh apps/web/build-web.ps1 -Release`. The script compiles the web binary for `wasm32-unknown-unknown`, runs `wasm-bindgen --target web`, and writes `dist-web/` (815 KB WASM + 47 KB JS + 1.3 KB HTML + snippets). Run `pwsh apps/web/build-web.ps1 -Serve` to build and serve on `http://127.0.0.1:8000/`. Open the URL in a browser — the Dioxus app boots, renders the same sidebar / pages as the desktop shell, and shows the Onboarding / Unlock / List / Detail / PasswordGenerator / ImportExport / Settings pages. Because the browser shell has no SQLite backend, the data-bearing flows (vault create / unlock / list / item CRUD / import / export) return `VaultStoreError::Unavailable` and the UI displays a "not available in this build" notice on the relevant action. The Password Generator works fully because it is pure Rust and has no I/O. The plaintext-export confirm modal still appears; clicking confirm reports "not available in this build" on the browser.
3. **Web (debug build)**: Same as release but the WASM artifact is ~3.5 MB and the build time is shorter. Use this for fast iteration on the UI.
4. **Web (smoke test)**: `pwsh apps/web/test-web.ps1` boots a static server on a free port in 8765-8800, fetches `/index.html`, `/porkpie-web.js`, `/porkpie-web_bg.wasm`, and the wasm-bindgen snippet, and asserts the bundle is well-formed. Exit code 0 means the bundle is shippable.

The `porkpie-ui` crate's `vault_store.rs` is the single source of truth for vault I/O; pages never call `porkpie_store` or `porkpie_core` directly. Decrypted material is held in `Option<DecryptedItem>` and only while the relevant screen is active. The same `App` component is shared across the desktop shell (real SQLite) and the web shell (browser-side `Unavailable`).
