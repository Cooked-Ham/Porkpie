# Status

Porkpie is a foundational Rust prototype, not safe for real credentials yet.

## Environment Setup

- **PORKPIE_API_KEY**: Must be a real secret at least 32 characters long. Placeholder values are rejected at startup.
- **PORKPIE_CORS_ALLOWED_ORIGINS**: Comma-separated list of allowed origins. Defaults to `https://app.porkpie.love`. Wildcards are rejected.
- **PORKPIE_DATABASE_URL / DATABASE_URL**: SQLite database path. Defaults to `sqlite:data/porkpie.db`.
- **PORKPIE_SERVER_BIND / API_PORT**: Server bind address. Defaults to `0.0.0.0:8000`.

## Implemented

- **porkpie-types**: Domain types (IDs, Items, Timestamps, 10 item type variants), error types, constants. Local secret key type with hex encoding, zeroize-on-drop, and redacted Debug. AAD builders for item and payload encryption binding. `PieUri` parser for `pie://vault/item/field` URIs. Field-level access methods (`get_field`, `set_field`, `list_fields`) on all item types.
- **porkpie-crypto**: Argon2id key derivation with local secret key support (`password || secret_key` input), XChaCha20Poly1305 authenticated encryption with associated data (AEAD), vault key wrapping, CSPRNG nonce generation.
- **porkpie-core**: Vault lifecycle (create with name + secret key + recovery kit, unlock requiring password + secret key, lock), item CRUD with AAD-bound encryption, password generator, memory zeroization.
- **porkpie-store**: SQLite persistence via SQLx, schema migrations with vault name support, encrypted-only item storage, concurrent access support. End-to-end plaintext proof tests verify no fixture secrets in raw SQLite bytes. Vault lookup by name. Sync state persistence (`load_sync_state`, `save_sync_state`). Item operations with explicit revision control (`load_item_records`, `load_items_with_type_since`, `upsert_item_revision`).
- **porkpie-sync (Phase 08)**: Sync is now a real bidirectional protocol. The `porkpie-sync` crate provides types (`EncryptedSyncItem`, `SyncRequest`, `SyncResponse`, `SyncOutcome`, `SyncCursor`, `ConflictItem`), conflict detection (`detect_conflicts`), merge strategies (`LastWriteWins`, `PreferLocal`, `PreferRemote`), and an orchestration function (`sync_vault`). The `porkpie-api` crate exposes `POST /api/v1/sync/register` (vault metadata registration with real salt/wrapped-key), `POST /api/v1/sync/begin` (pull encrypted changes after a cursor revision), `POST /api/v1/sync/push` (push encrypted items with conflict detection returning HTTP 409), and `GET /api/v1/vault/:vault_id` (fetch encrypted vault metadata for peer reconstruction). The `porkpie-cli` `sync` command performs the full bidirectional flow: register vault, load local sync cursor, pull encrypted changes from server, decrypt and merge items into the unlocked vault, re-encrypt and re-persist all vault items with bumped revisions, push locally-changed encrypted items to server, and persist the updated cursor. Conflicts are preserved — when Profile B pushes an item that Profile A already edited, the server returns `409 Conflict` with `ConflictItem[]` payload carrying both sides' revisions and ciphertext. The `--strategy` flag (`last-write-wins`, `prefer-local`, `prefer-remote`) controls push conflict behaviour. The `porkpie-store` crate gained `load_sync_state`/`save_sync_state` for the client-side sync cursor, `load_item_records`/`load_items_with_type_since`/`upsert_item_revision` for item operations with explicit revision control. The `porkpie-types` crate gained `ItemType::type_label()` for mapping item variants to their string label. A new `bidirectional_sync.rs` integration test in `porkpie-api` exercises the full two-profile flow: Profile A creates vault + item, registers + pushes, Profile B pulls + decrypts + verifies content, both profiles edit the same item, Profile A pushes successfully, Profile B gets a 409 conflict with preserved item metadata, and the server database is verified to contain no fixture plaintext. All ciphertext is opaque binary; the server never receives master passwords, vault keys, or plaintext item fields. Conflict data carries encrypted blobs (server_data) that the peer can decrypt with the correct vault key.
- **porkpie-api (Phase 08 + 10)**: Axum HTTP sync server with `register`, `begin`, `push`, and `vault-metadata` endpoints. Bearer-token auth with SHA-256 hashed API key storage. Encrypted blob storage. Push conflict detection returning HTTP 409 with `ConflictItem[]` payload. Server-db plaintext scan proof in the bidirectional sync test. Server fails startup if `API_KEY` env var is missing. **Phase 10 hardening**: The sync `push` endpoint now rejects plaintext item-shaped payloads — if the `ciphertext` field contains JSON-like structures with sensitive field names (`username`, `password`, `private_key`, `api_key`, `totp`, `notes`), the server returns HTTP 422 (`validation_error`). Auth tests cover missing key, wrong key, revoked key, and missing config. API request bodies and encrypted blobs are never logged. No decrypt endpoint exists. The server remains zero-knowledge. **KDF params**: The sync server now stores Argon2id KDF parameters (`kdf_time_cost`, `kdf_mem_cost`, `kdf_parallelism`) in the `vaults` table and returns them in the `vault-metadata` endpoint so peers can reconstruct the exact vault unlock parameters. **Admin self-revoke guard**: The `admin_revoke_api_key` endpoint prevents an admin from revoking their own key, avoiding accidental lockout.
- **porkpie-cli**: URI-first secret operations with `pie://` as canonical interface. Commands: `init` (with vault name + recovery kit), `unlock`, `lock`, `item list` (redacted), `item get` (redacted), `read <pie-uri>` (field reveal), `write <pie-uri> <value>` (field update), `copy <pie-uri>` (clipboard), `run --env NAME=pie://... -- <cmd>` (env injection), `add`, `edit`, `delete`, `backup export` (encrypted backup), `backup import <file>` (encrypted backup or CSV), `export` (encrypted by default, `--format plaintext --dangerous` for explicit dangerous plaintext export), `import <file>` (encrypted backup or CSV), `sync` (full bidirectional sync with `--server`, `--api-key`, `--strategy` flags, vault registration, pull/merge, push, and local sync cursor persistence), `ssh public-key <pie-uri-or-item>` (public key display without revealing private key), `ssh-agent` (starts a Unix domain socket agent with SSH keys from the unlocked vault). Session management with local secret key.
- **porkpie-import (Phase 11)**: CSV import (Login, APIKey, SecureNote types), encrypted backup export/import with duplicate handling (`SkipDuplicates` / `OverwriteDuplicates`), wrong-password rejection, and secret key requirement. **Phase 11 safety**: Plaintext export requires `--dangerous` flag and an explicit interactive confirmation. The `export` command defaults to encrypted backups. Imported CSV secrets are encrypted immediately by the vault before storage. The UI plaintext export is gated behind a destructive-styled Modal confirm. Third-party format support (Bitwarden, 1Password, LastPass) is not implemented.
- **porkpie-ui (Phase 06)**: Dioxus 0.4 UI is now wired to real vault operations end-to-end. `VaultBackend` enum dispatches to a SQLite-backed implementation (desktop/server) and a `LocalStorage` backend for WASM. `UnlockedVaultHandle` clones via `Arc<Mutex<Vault>>` so a single unlocked vault can be shared across the page tree. The `App` component owns both a `UseRef<VaultBackend>` and a `UseRef<AppState>`, runs `initial_load` to connect SQLite and list vaults, and routes to Onboarding / Unlock / List / Detail / NewItem / PasswordGenerator / ImportExport / Settings. Onboarding creates a real vault and surfaces the recovery kit; Unlock requires password + 64-hex local secret key; List shows redacted item metadata with search, lock, and refresh; Detail is a type-aware form for all 10 item types, with save (create or update), copy-password, and delete-confirmation modal. PasswordGenerator calls `porkpie_core::generate_password` and copies the result. ImportExport performs real `porkpie_import::export_backup_file` and `import_backup` round-trips, with plaintext export gated behind a destructive-styled `Modal` confirm. Settings exposes lock timeout, theme, and a Lock button. No decrypted secret material is written to localStorage, sessionStorage, or any client-side cache. `BackupImportMode` gained a `Default` (SkipDuplicates) impl.
- **porkpie-ui WASM target (Phase 07)**: The shared Dioxus UI now compiles cleanly for `wasm32-unknown-unknown`. Vault I/O methods use a `LocalStorage` backend on WASM that stores encrypted vault metadata and item ciphertext in browser `localStorage`. The same `VaultBackend` abstraction dispatches to SQLite on desktop and `localStorage` on WASM. All pages (onboarding, unlock, list, detail, import/export, settings) work in the browser with encrypted persistence. The `porkpie-core` password generator runs unchanged on WASM. The shared `porkpie-ui` crate is a single source of truth for both shells.
- **apps/desktop (Phase 07)**: The desktop shell is now a real `porkpie-desktop` binary that calls `dioxus_desktop::launch_cfg` with a `LaunchConfig` that derives a window title, width, height, and SQLite URL from environment variables. The default SQLite location is the platform data directory (`%APPDATA%\Porkpie\porkpie.db` on Windows, `~/Library/Application Support/Porkpie/porkpie.db` on macOS, `$XDG_DATA_HOME/porkpie/porkpie.db` on Linux). `cargo run -p porkpie-desktop` produces a working window that renders the shared UI on top of the real SQLite-backed vault store.
- **apps/web (Phase 07)**: The web shell is now a real `porkpie-web` binary that calls `dioxus_web::launch_cfg` with a `LaunchConfig` driven by `PORKPIE_WEB_ROOT`. The shell ships an `apps/web/index.html` template that loads the `wasm-bindgen`-produced `porkpie-web.js` as an ES module and a `apps/web/build-web.ps1` script that compiles the binary for `wasm32-unknown-unknown`, runs `wasm-bindgen --target web`, and copies the `index.html` template into a `dist-web/` directory alongside the snippets. The release WASM artifact is ~815 KB; the debug artifact is ~3.5 MB. The script also has a `-Serve` mode that boots a small static HTTP server on the bundle for end-to-end smoke tests, and a `-Clean` flag that wipes the output. A `test-web.ps1` script fetches `/index.html`, `/porkpie-web.js`, `/porkpie-web_bg.wasm`, and the wasm-bindgen snippet over HTTP and asserts the bundle is well-formed. `cargo build -p porkpie-web --target wasm32-unknown-unknown` succeeds with no warnings.
- **porkpie-agent (Phase 09)**: The `porkpie-agent` crate is no longer a scheduling stub. It now provides a real `SshSigner` trait (`algorithm`, `public_key_bytes`, `sign`), host/key policy structs (`HostKeyPolicy`, `SshKeyIdentity`), and an `Ed25519Signer` in-memory implementation backed by `ed25519-dalek`. The signer trait is tested with real Ed25519 signing and verification. The `porkpie ssh-agent` CLI command starts a real OpenSSH-compatible agent on a Unix domain socket, loading SSH keys from the unlocked vault. It supports identity listing, signing requests, host-based policy restrictions, and interactive confirmation. The SSH key item type (`SSHKeySecret`) now supports `comment` and `allowed_hosts` fields in addition to `name`, `public_key`, `private_key`, and `passphrase`. Private keys are encrypted at rest via the vault's XChaCha20Poly1305 encryption. The `porkpie ssh public-key <target>` command prints only the public key and never the private key.
- **Infrastructure Deployment Scaffold**: The `infra/` directory is no longer empty. `infra/caddy/Caddyfile` provides reverse proxy with automatic HTTPS, security headers, and `zstd`/`gzip` encoding. `infra/compose/docker-compose.yml` defines a production stack (`porkpie-server` + `caddy`) with persistent volumes (`porkpie-data`, `caddy-data`, `caddy-config`), healthchecks, and environment variable injection from `.env`. `infra/compose/docker-compose.dev.yml` provides a dev-only server on port 8080 without Caddy. `infra/compose/.env.example` contains placeholder values only (no real secrets). `infra/docker/server.Dockerfile` is a multi-stage build (`rust:1-bookworm` → `debian:bookworm-slim`) that compiles the `porkpie-api` crate (which defines the `porkpie-server` binary) and produces a minimal runtime image with `ca-certificates` and `sqlite3`. The server runs as the unprivileged `porkpie` user and persists data to `/data`. The server fails startup if `PORKPIE_API_KEY` is missing. The `--healthcheck` flag attempts a TCP connection to the server's own bind address and exits 0 if reachable, 1 otherwise.
- **Phase 12: Final Hostile QA Pass**: A comprehensive audit of the repository was performed. The `docs/AUDIT_REPORT.md` documents the findings. The workspace passes all automated validation (226 tests, clean Clippy, release build). No critical security failures were found in the current implementation. All `unwrap()` and `expect()` calls are confined to test files. No `TODO`/`FIXME`/`dev-key-change-in-production` exist in production code. No Electron/React/TypeScript/Vite. The `println!` calls in CLI commands are status messages only (except `porkpie read` which prints secrets by design). The completion gate was updated to reflect the actual status: **8 of 9 gates pass**, 1 partially passes (Security: 9/10). The project remains labeled a prototype. The remaining blockers are: (1) no external security audit, (2) memory zeroization not verified by tests.

## Recent Fixes (2026-06-01)

1. **rotate_local_secret keychain deletion bug** — Fixed `vault_cmd::rotate_local_secret` to delete the *old* keychain entry after storing the *new* one, instead of deleting the new entry immediately. Added `rotate_local_secret_keychain_replaces_old_key` test.
2. **KDF params in sync server** — The `vaults` table now stores `kdf_time_cost`, `kdf_mem_cost`, `kdf_parallelism` columns. The `sync/register` endpoint accepts them, `vault-metadata` returns them, and the CLI sync client sends them. Added migration for existing databases. Added `vault_metadata_returns_kdf_params` and `sync_register_persists_kdf_params` tests.
3. **Admin self-revoke guard** — The `admin_revoke_api_key` endpoint now rejects requests to revoke the key currently in use, preventing accidental admin lockout. Added `admin_revoke_api_key_rejects_self_revoke` test.
4. **Recovery restore** — Implemented: `porkpie recovery restore --kit <kit> --backup <backup>` reads a recovery kit JSON, extracts the vault_id and local_secret_key, reads the encrypted backup, prompts for the master password, decrypts the backup, and stores the vault + items in the local database. The secret key is stored in the OS keychain. The command is no longer behind `#[cfg(feature = "experimental-recovery")]`.

## Partial / Scaffolded

- **porkpie-agent**: The `SshSigner` trait, `Ed25519Signer`, and policy structs are real and tested. The `porkpie ssh-agent` CLI command starts a real OpenSSH-compatible Unix domain socket agent. Windows named pipes are not supported.
- **apps/server**: Re-exports `porkpie_api::{build_router, AppState}`. The actual server binary lives in `crates/porkpie-api` (binary name `porkpie-server`). The `apps/server` crate is a library-only wrapper.

## Stubs / Empty Shells

None as of Phase 07. The `apps/desktop` and `apps/web` shell crates are no longer re-exports — both have real binary targets and launch code paths.

## Not Implemented

- Desktop shell integration beyond launch: no system tray, no global hotkeys, no clipboard auto-clearing.
- Browser extension / autofill.
- OpenSSH agent socket/named-pipe integration (the `porkpie-agent` crate has the signer trait and in-memory signer, but no actual socket or named-pipe agent).
- Recovery code workflows.
- Team sharing / public-key recipient wrapping.
- Hardware-backed key support (FIDO2/WebAuthn/YubiKey).
- Multi-device sync client configuration.
- Third-party importers (1Password, Bitwarden, LastPass native formats).
- Security audit or penetration testing.

## Unsafe / Unverified

### Security Audit

- No external security audit or penetration testing has been performed.
- A hostile QA pass (Phase 12) was completed on 2026-06-01. The `docs/AUDIT_REPORT.md` documents the findings. No critical security failures were found in production code, but several gaps remain before the project can claim MVP status.
- The remaining blockers are: (1) no external security audit.

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
- `porkpie export --format plaintext --dangerous` is implemented with an interactive confirmation prompt.

### Session File

- The session file (`.porkpie-session.json`) encrypts the local secret key using a key derived from the vault ID. The `secret_key_encrypted` field replaces the old plaintext `secret_key_hex`. The old field is still read for backward compatibility but new sessions are written encrypted. The vault ID is also stored in the file, so an attacker with the session file can derive the same key — this is obfuscation, not strong encryption, but it raises the bar from "read plaintext" to "reverse the key derivation".
- Commands that use the old session-based unlock flow (`export`, `import`, `sync`) still prompt for the master password but rely on the session file for the secret key. This is consistent with the new model but means the session file remains a high-value target.

### Crypto Parameters

- Argon2id parameters (time_cost=2, mem_cost=19456 KiB, parallelism=1) are conservative defaults. Production deployments may want higher values for stronger brute-force resistance.
- `Vault::rotate_vault_key(password, secret_key)` generates a new vault key, re-encrypts all items, and returns the new ciphertexts for persistence. The old vault key is zeroized on drop.
- API key hash comparison uses `subtle::ConstantTimeEq` to avoid timing side-channels.

### Logging and Output

- Systematic audit of `println!` / `eprintln!` output performed: no secrets leak in status messages. `porkpie read` prints secrets by design (explicit reveal command). No `tracing` framework is used.
- Error messages may contain vault IDs or item IDs, which are non-secret but could aid an attacker in targeting specific data.

### Code-Level Gaps Found During Verification

The following gaps were discovered by reading every source file:

1. ~~**`expect()` in production source**~~ ✅ FIXED — `timestamp.rs:12` now uses `unwrap_or(Duration::ZERO)` instead of `expect()`. `secret_key.rs:38` was eliminated by changing `LocalSecretKey` to store `[u8; 32]` instead of `Vec<u8>`.
2. ~~**Memory zeroization not verified**~~ ✅ FIXED — `lock_clears_items_from_memory` test verifies `items.is_empty()` and `VaultLocked` state. Added `zeroize_secret_material_clears_item_fields` test that asserts `String` fields are truncated after zeroization. Added `lock_zeroizes_vault_key` test that asserts the vault key is dropped after lock.
3. ~~**Zeroization gaps in `porkpie-crypto`**~~ ✅ FIXED — `vault_key.rs:33` and `encryption.rs:14` now use `Zeroizing<Vec<u8>>` to overwrite decrypted/serialized buffers on drop.
4. ~~**`Vault` public mutable fields**~~ ✅ FIXED — All fields are now private with accessor methods (`items()`, `items_mut()`, `sync_revision()`, `master_key_wrapped()`, `is_locked()`). External code can no longer bypass `lock()`/`unlock()` invariants.
5. **`CoreError::InvalidEncryptedItem` unused** — Defined in `errors.rs:25` but never referenced.
6. **Password generator ASCII assumption** — `password_gen.rs:105` uses `char::from(b)` on ASCII bytes. Safe for current sets but could silently break with non-ASCII.

### UI

- **Phase 06 made the UI real**: pages, forms, and dialogs are wired to live vault I/O, not mock data. Decrypted item material is only ever held in `AppState::current_item` (an `Option<DecryptedItem>`) while the user is actively viewing or editing a single item, and is cleared on lock, on screen change, and on logout. The unlock form, onboarding form, import form, and detail form all reflect real state via `use_state`/`use_ref`. The plaintext export is gated by a `Modal` confirm with a destructive button style, but the in-app "I understand" affordance is a soft check rather than a typed-phrase challenge; treat it as UI safety, not as a strong guarantee. Theme switching is wired to state and re-renders live CSS variables via the `data-theme` attribute on the app-shell.
- **Phase 07 launches the UI**: `cargo run -p porkpie-desktop` opens a WebView2 window (WebKitGTK on Linux, WebKit on macOS). The web shell `cargo build -p porkpie-web --target wasm32-unknown-unknown` followed by `wasm-bindgen --target web` produces a static `dist-web/` bundle (`index.html` + `porkpie-web.js` + `porkpie-web_bg.wasm` + snippets) that any HTTP server can serve. `pwsh apps/web/build-web.ps1 -Serve` automates the build + serve path. The web shell uses a `localStorage` bridge for client-side persistence (WASM-only). Vault create, unlock, list, item CRUD, import/export all work in the browser using `VaultBackend::LocalStorage`. Data is still encrypted at rest (XChaCha20Poly1305 ciphertexts are stored in localStorage). The web build is Rust-only at runtime: no Electron, no React, no TypeScript, no Vite. The desktop binary is 14.6 MB; the release WASM bundle is ~815 KB.

### API Server

- The server config now reads `PORKPIE_API_KEY`, `PORKPIE_DATABASE_URL`, and `PORKPIE_SERVER_BIND` (with fallbacks to `API_KEY`, `DATABASE_URL`, and `API_PORT` for backward compatibility). The server fails startup if the API key is missing or empty.
- Docker Compose files in `infra/compose/` use `.env` for environment variable injection. No real secrets are committed — `.env` is gitignored and `.env.example` contains placeholders only.
- Root `Dockerfile` and `docker-compose.yml` have been deleted. Use `infra/docker/server.Dockerfile` and `infra/compose/docker-compose.yml` instead.
- `README.md` API Server section updated to point to `infra/compose/` and use `PORKPIE_*` env var names.
- No `.env` file validation or secret generation tooling is provided beyond the placeholder `.env.example`. Users must manually generate and manage API keys.

## Documentation Inaccuracies Found During Verification

The following docs contain claims that do not match the actual codebase:

1. ~~**`STATUS.md` (this file)**~~ ✅ FIXED — Misattributed functions (`upsert_vault_metadata`, `upsert_api_key`, `api_key_exists`, `hash_api_key`, `revoke_api_key`, `detect_plaintext_payload`) are now correctly documented as existing in `porkpie-api`. `encrypted_data` references updated to `ciphertext`. `SessionContext` references updated to `SessionState`.
2. ~~**`docs/DATA_MODEL.md`**~~ ✅ FIXED — `encrypted_data` column name updated to `ciphertext` throughout.
3. ~~**`docs/ARCHITECTURE.md`**~~ ✅ FIXED — `porkpie-agent` description updated to SSH signer foundation.
4. ~~**`docs/feature-production-readiness-1.0.md`**~~ ✅ FIXED — False state claims removed, TASK-004 flag corrected to `--dangerous`.
5. ~~**`docs/SECURITY_INVARIANTS.md`**~~ ✅ FIXED — Line 15 changed to `--dangerous`.
6. ~~**`docs/AGENT_TASKS.md`**~~ ✅ FIXED — `tasks/` reference removed.
7. ~~**`docs/TEST_PLAN.md`**~~ ✅ FIXED — Docker commands now point to `infra/`.

## Build Status

The workspace passes:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo build --workspace --release
```

226 tests pass across all crates (0 failures). The web shell additionally passes:

```bash
cargo build -p porkpie-web --target wasm32-unknown-unknown
```

and the `pwsh apps/web/build-web.ps1` + `pwsh apps/web/test-web.ps1` end-to-end smoke test.

### Master Verification Audit

A **comprehensive doc-to-code verification** was performed on 2026-06-01. The full findings are in `docs/MASTER_VERIFICATION_HIT_LIST.md`. Key results:

- **All crate implementations verified** against doc claims. 226 tests pass, 0 failures.
- **No critical security failures** in production code.
- **No fake crypto, no static mockups, no Electron/React/TypeScript/Vite.**
- **Significant documentation inaccuracies found** (see below).

### Phase 12 Audit

The `docs/AUDIT_REPORT.md` contains the full hostile QA findings. Key results:
- **No critical security failures** in production code.
- **No `unwrap()` / `expect()`** in production source code (all in tests).
- **No `TODO` / `FIXME` / `dev-key-change-in-production`** in production code.
- **No fake crypto, no base64 encryption, no hardcoded keys, no static/reused nonces**.
- **Completion Gate**: 8 of 9 gates pass, 1 partially passes. Blockers: no external security audit.

### Test Coverage Highlights

- **Crypto**: 11 tests covering key derivation with secret key, AAD binding (wrong vault ID, wrong item ID, wrong schema version), tampering detection, nonce uniqueness.
- **Vault lifecycle**: 7 tests covering create, unlock with correct password + secret key, unlock failure with wrong password, unlock failure with wrong secret key, password-alone-cannot-unlock.
- **Debug redaction**: 11 tests proving fixture secrets never appear in Debug output for all 10 item type variants, Vault, and Item.
- **API key security**: 4 tests proving raw keys are not stored in the database, hash comparison works, different keys produce different hashes, and config rejects missing API key env. Plus 5 config tests (empty key, short key, placeholder rejection, valid key, redacted debug). Plus 5 CORS tests (wildcard rejection, invalid URL rejection, FTP rejection, multiple origins, empty default).
- **Plaintext proof**: 2 end-to-end tests scanning raw SQLite + WAL + SHM bytes to verify no fixture secrets (login password, API key, SSH private key, secure note, database password, recovery code) are persisted.
- **PieUri**: 9 tests covering URI parsing, validation, display, redacted display, and error messages that don't leak values.
- **Field access**: 7 tests covering get/set operations on Login, APIKey, SSHKey, Database, and Custom item types.
- **CLI parsing**: 28 tests covering help text, invalid args, version output, global options, sync options, strategy parsing, write options, item CRUD, read, export, and session state.
- **UI state**: 3 tests covering `AppState::lock()` clearing the current vault + items, lock-timeout detection only applying to unlocked sessions, and `Screen` round-tripping for all 7 screens.
- **UI components**: 4 tests covering item-list filter, master-password validation, password-generator controls, and password-generator-state debug redaction.
- **UI vault store (Phase 06)**: 3 tests covering full create+unlock+CRUD+lock round trip on an in-memory SQLite database, and rejection of wrong passwords. These tests run only on non-WASM targets; the WASM build uses `VaultBackend::LocalStorage` with equivalent functionality verified by the WASM build and manual browser tests.
- **API sync (Phase 08)**: 15 API tests (health, missing auth, unknown vault, push+begin round trip, conflict detection, same-item-id collision, 6 plaintext rejection, 1 encrypted blob acceptance, 3 auth) plus the bidirectional sync integration test.
- **SSH agent (Phase 09)**: 10 tests across `porkpie-agent` — signer trait works with an unlocked in-memory key, Ed25519 signature generation and verification, deterministic seed-based key derivation, algorithm name (`ssh-ed25519`), public key byte matching, different signers produce different keys, host policy unrestricted, host policy restriction to allowed hosts, confirmation flag, and SSH key identity struct holding comment and public key. 4 new CLI tests verify `ssh public-key` parsing, `ssh-agent` parsing, help text inclusion, and the honest status message from the binary.
- **API hardening (Phase 10)**: 15 tests across `porkpie-api` — 6 plaintext payload rejection tests (username, password, private_key, api_key, totp, notes), 1 real encrypted blob acceptance test, 3 auth tests (wrong API key, revoked API key, missing API key config), 1 malicious-collision test (same item ID in two vaults), plus the existing 5 API tests (health, missing auth, unknown vault, push+begin round trip, conflict detection) and the bidirectional sync integration test.
- **Import/export safety (Phase 11)**: 6 new tests — 5 CLI parsing tests (`backup export`, `backup export --output`, `backup import`, `export encrypted default`, `export plaintext --dangerous`) and 1 CSV import test proving `DO_NOT_LEAK_CSV_SECRET` does not appear in the ciphertext. The existing 4 backup tests (roundtrip keeps secrets encrypted, rejects wrong password, skips duplicates, serializes with `.enc` extension) and 2 CSV tests (creates encrypted login rows, rejects missing fields) continue to pass.

## Manual QA Flow (Phase 07)

The Phase 06 acceptance criterion is a working end-to-end flow. The Phase 07 acceptance criterion is that this flow is reachable from a binary launch, not just from `cargo test`. The `cargo test` round-trip is the automated proof, but the user-visible flows are:

1. **Desktop**: `cargo run -p porkpie-desktop` opens a WebView2 window (WebKitGTK on Linux, WebKit on macOS). On first launch, the Onboarding screen is shown with no vaults present. Enter a vault name (e.g. `Personal`) and a master password (≥16 chars). The local secret key is auto-generated and the recovery kit is displayed. Save the recovery kit to an offline location. The app routes to the List screen, which is empty. Use the **+** button to add an item. The Detail form is type-aware: pick a type, fill the fields, save. The saved item appears in the list with redacted metadata. Open it to reveal the field values, copy the password via the **Copy password** button, edit and save, or delete via the delete button (with confirm modal). From Settings, click **Lock vault**. Re-enter the password and secret key (or paste from the recovery kit) to unlock again. The default SQLite location is the platform data directory; override with `PORKPIE_DATABASE_URL` or `PORKPIE_DATA_DIR`. The default window size is 1180x820; override with `PORKPIE_WINDOW_WIDTH`, `PORKPIE_WINDOW_HEIGHT`, or `PORKPIE_WINDOW_TITLE`.
2. **Web (release build)**: From the repo root, run `pwsh apps/web/build-web.ps1 -Release`. The script compiles the web binary for `wasm32-unknown-unknown`, runs `wasm-bindgen --target web`, and writes `dist-web/` (815 KB WASM + 47 KB JS + 1.3 KB HTML + snippets). Run `pwsh apps/web/build-web.ps1 -Serve` to build and serve on `http://127.0.0.1:8000/`. Open the URL in a browser — the Dioxus app boots, renders the same sidebar / pages as the desktop shell, and shows the Onboarding / Unlock / List / Detail / PasswordGenerator / ImportExport / Settings pages. The browser shell uses `VaultBackend::LocalStorage` for encrypted vault persistence. Vault create, unlock, list, item CRUD, import, and export all work in the browser. The Password Generator works fully because it is pure Rust and has no I/O. The plaintext-export confirm modal works and writes the export JSON to the download path.
3. **Web (debug build)**: Same as release but the WASM artifact is ~3.5 MB and the build time is shorter. Use this for fast iteration on the UI.
4. **Web (smoke test)**: `pwsh apps/web/test-web.ps1` boots a static server on a free port in 8765-8800, fetches `/index.html`, `/porkpie-web.js`, `/porkpie-web_bg.wasm`, and the wasm-bindgen snippet, and asserts the bundle is well-formed. Exit code 0 means the bundle is shippable.

The `porkpie-ui` crate's `vault_store.rs` is the single source of truth for vault I/O; pages never call `porkpie_store` or `porkpie_core` directly. Decrypted material is held in `Option<DecryptedItem>` and only while the relevant screen is active. The same `App` component is shared across the desktop shell (real SQLite) and the web shell (browser-side `LocalStorage`).

## Self-Host Deployment (Infra Task)

The `infra/` directory contains a complete self-host deployment scaffold:

1. **Production stack**: `docker compose -f infra/compose/docker-compose.yml up -d` starts the server behind Caddy with automatic HTTPS on ports 80/443. The server is exposed internally on port 8080 and reverse-proxied by Caddy. Persistent volumes keep the SQLite database (`porkpie-data`) and Caddy state (`caddy-data`, `caddy-config`) across restarts.
2. **Development stack**: `docker compose -f infra/compose/docker-compose.dev.yml up -d` starts the server directly on `http://localhost:8080` without Caddy. This is useful for local testing and API development.
3. **Configuration**: Copy `infra/compose/.env.example` to `infra/compose/.env`, set `PORKPIE_API_KEY` to a long random secret, and set `PORKPIE_PUBLIC_URL` / `PORKPIE_PUBLIC_HOST` to your domain. The server will refuse to start if the API key is missing or set to the placeholder value.
4. **Security**: The server image runs as an unprivileged `porkpie` user. The `.env` file is gitignored. No real secrets are committed. API keys are hashed with SHA-256 before storage. The Caddyfile includes `X-Content-Type-Options nosniff`, `X-Frame-Options DENY`, and `Referrer-Policy no-referrer` headers.
5. **Healthcheck**: The Docker healthcheck runs `/usr/local/bin/porkpie-server --healthcheck` every 30 seconds. This attempts a TCP connection to the server's bind address and returns 0 if reachable, 1 otherwise.
6. **Build**: The Dockerfile uses a multi-stage build. The builder stage compiles the `porkpie-api` crate (which defines the `porkpie-server` binary). The runtime stage is a minimal `debian:bookworm-slim` image with the compiled binary, `ca-certificates`, and `sqlite3`.
