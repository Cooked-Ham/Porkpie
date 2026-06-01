# Porkpie

> **WARNING:** Do not use Porkpie with real credentials yet. This is a prototype pending hardening and security review.

Porkpie is now suitable for Developer Alpha and controlled Personal Dogfood with explicit risk acceptance.
It is not externally audited and should not be broadly recommended for other people's high-value credentials yet.

Local-first, zero-knowledge, self-hostable password manager for developers, homelab users, and small teams.

- **CLI:** porkpie
- **URI Scheme:** pie://
- **Built With:** Rust, Dioxus, Axum, SQLx
- **Storage:** SQLite with encrypted vault metadata and item ciphertext only
- **Sync:** Axum API with bearer-token auth and encrypted blob replication
- **SSH Agent:** Protocol foundation only (wire format implemented; no socket integration yet)
- **Desktop:** Dioxus desktop (WebView2 on Windows, WebKitGTK on Linux, WebKit on macOS)
- **Web:** Dioxus web (WASM, no Electron, no React, no TypeScript, no Vite)

## Quick Start

```bash
cargo build --workspace
cargo test --workspace
cargo run --bin porkpie -- --help
```

## CLI Basics

```bash
porkpie init
porkpie unlock
porkpie add login
porkpie item list
porkpie read pie://Personal/GitHub/password
porkpie export
porkpie import porkpie-backup-1780000000000.json.enc
porkpie vault change-password
porkpie vault rotate-local-secret
porkpie vault rotate-key --skip-backup
porkpie vault upgrade-kdf hardened
porkpie vault calibrate-kdf 500
porkpie recovery verify <kit>
```

**Note:** `porkpie recovery restore` is now implemented. Pass `--kit` and `--backup` to restore a vault from a recovery kit and encrypted backup.

CSV imports use the columns `item_type,title,username,password,notes`. Encrypted backups use `.json.enc` files and contain encrypted item blobs only.

## Desktop App

The desktop shell is a Dioxus desktop binary that opens a WebView window and runs the same UI flow as the web shell. All vault I/O uses a real SQLite database.

```bash
cargo run -p porkpie-desktop
```

The binary is `porkpie-desktop` (a single-file `porkpie-desktop.exe` on Windows, `porkpie-desktop` on Linux/macOS).

The first launch drops you on the Onboarding screen. The local SQLite database defaults to the platform data directory:

- Windows: `%APPDATA%\Porkpie\porkpie.db`
- macOS: `~/Library/Application Support/Porkpie/porkpie.db`
- Linux: `$XDG_DATA_HOME/porkpie/porkpie.db` (or `~/.local/share/porkpie/porkpie.db`)

Override the location, window title, or size with environment variables:

| Variable                | Default                                                  | Purpose                          |
| ----------------------- | -------------------------------------------------------- | -------------------------------- |
| `PORKPIE_DATABASE_URL`  | `sqlite://<data dir>/porkpie.db?mode=rwc`                | SQLite URL or in-memory fallback |
| `PORKPIE_DATA_DIR`      | platform data dir                                        | Parent dir for `porkpie.db`      |
| `PORKPIE_WINDOW_TITLE`  | `Porkpie`                                                | Window title                     |
| `PORKPIE_WINDOW_WIDTH`  | `1180`                                                   | Initial window width in logical px |
| `PORKPIE_WINDOW_HEIGHT` | `820`                                                    | Initial window height in logical px |

### Desktop Prerequisites

- Windows: WebView2 runtime (already installed on Windows 10/11 with Edge). If `cargo run` fails with a missing DLL, install the [Evergreen Bootstrapper](https://developer.microsoft.com/microsoft-edge/webview2/).
- Linux: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev` (Debian/Ubuntu; on Debian bullseye, `libayatana-appindicator3-dev` is renamed to `libappindicator3-dev`).
- macOS: no extra dependencies.

## Web App

The web shell is a Dioxus web binary compiled to WebAssembly. The WASM module is a Rust binary — no JavaScript framework, no Vite, no Electron, no TypeScript.

### Build the web bundle

The web shell does not depend on the Dioxus CLI. It uses the standard Rust + `wasm-bindgen` toolchain.

Required one-time setup:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.122 --locked
```

Then build the bundle:

```bash
# From the repo root:
pwsh apps/web/build-web.ps1                # debug build, writes dist-web/
pwsh apps/web/build-web.ps1 -Release       # release build, ~800 KB WASM
pwsh apps/web/build-web.ps1 -Serve         # build then start a static server at http://127.0.0.1:8000
pwsh apps/web/build-web.ps1 -Serve -Port 9000
pwsh apps/web/build-web.ps1 -Clean         # wipe dist-web/ before building
```

The `wasm-bindgen-cli` version pin is mandatory: the `wasm-bindgen` crate pulled in by `dioxus-web 0.4.3` is `0.2.122`, and the CLI binary's schema version must match the WASM artifact's schema version. Mismatches print `this binary schema version: 0.2.X` and abort the build.

### What the bundle contains

After the build, `dist-web/` contains:

```
dist-web/
  index.html              # static entry; loads porkpie-web.js as an ES module
  porkpie-web.js          # wasm-bindgen JavaScript glue
  porkpie-web_bg.wasm     # the compiled Dioxus app
  snippets/               # wasm-bindgen inlined snippets (Dioxus interpreter)
```

### Run the web bundle

The bundle is a plain static directory. Serve it with any static file server that understands `.wasm` MIME types:

```bash
# Built-in (the build script's -Serve flag):
pwsh apps/web/build-web.ps1 -Serve

# Or, after building, use any static server, e.g.:
python -m http.server --directory dist-web 8000
npx --yes http-server dist-web -p 8000
```

The web shell mounts on the `<div id="main">` element in `apps/web/index.html`. Override the mount point with `PORKPIE_WEB_ROOT` before building.

### Web app behaviour

The web shell uses the same `porkpie-ui::App` component as the desktop shell. The Dioxus app renders the Onboarding, Unlock, List, Detail, Password Generator, Import/Export, and Settings screens exactly as the desktop shell does.

The browser build stores encrypted vault metadata and item ciphertext in browser `localStorage`. This is encrypted at rest by Porkpie's vault crypto, but `localStorage` is still accessible to JavaScript running on the origin. Use a dedicated trusted origin. IndexedDB/OPFS is preferred for future production work.

- Onboarding creates a vault in `localStorage` and surfaces the recovery kit.
- Unlock loads the vault from `localStorage` and decrypts items in memory.
- Item list, detail, create, update, and delete operate on `localStorage`.
- The Password Generator works fully on WASM — it uses the `porkpie-core` password generator, which is pure Rust with no I/O.
- Import/export works on WASM with the same limitations as the desktop (encrypted by default, plaintext with explicit `--dangerous`).
- Lock clears decrypted state from memory.

All other vault I/O uses the same code paths as the desktop build, with `localStorage` replacing SQLite.

This is the documented web shell mode: the same UI surface area, with `localStorage`-backed encrypted vault persistence in the browser.

## API Server

### Troubleshooting

#### Database path issues

If the desktop app fails to start with a database error, the startup self-check will print the exact failure category:

- `invalid URL` — the `PORKPIE_DATABASE_URL` value is not a valid SQLx SQLite URL.
- `cannot create directory` — the parent directory of the database file could not be created.
- `cannot open database` — the database file could not be opened (e.g., a directory exists at the same path).
- `migration failed` — the schema migration step failed.
- `permission denied` — the process lacks write permission to the database directory.

#### Override the default database path

The desktop app defaults to the platform data directory:

| Platform | Default path |
|----------|-------------|
| Windows | `%APPDATA%\Porkpie\porkpie.db` |
| macOS | `~/Library/Application Support/Porkpie/porkpie.db` |
| Linux | `$XDG_DATA_HOME/porkpie/porkpie.db` (or `~/.local/share/porkpie/porkpie.db`) |

Override the parent directory:

```bash
PORKPIE_DATA_DIR=/custom/path cargo run -p porkpie-desktop
```

Override the full SQLite URL:

```bash
PORKPIE_DATABASE_URL="sqlite:///custom/path/porkpie.db?mode=rwc" cargo run -p porkpie-desktop
```

On Windows, backslashes are automatically converted to forward slashes in the URL:

```powershell
$env:PORKPIE_DATA_DIR="C:\Users\Alice\Porkpie"
cargo run -p porkpie-desktop
# URL becomes: sqlite://C:/Users/Alice/Porkpie/porkpie.db?mode=rwc
```

#### Reset the local dev database

Delete the SQLite file and WAL/SHM artifacts:

```bash
# macOS
rm -f ~/Library/Application\ Support/Porkpie/porkpie.db*

# Linux
rm -f ~/.local/share/porkpie/porkpie.db*

# Windows (PowerShell)
Remove-Item "$env:APPDATA\Porkpie\porkpie.db*"
```

Or set `PORKPIE_DATABASE_URL=sqlite::memory:` to use an ephemeral in-memory database.

## API Server

```bash
# Production (with Caddy reverse proxy + HTTPS)
cd infra/compose
docker compose -f docker-compose.yml up --build

# Development (server only, no Caddy)
cd infra/compose
docker compose -f docker-compose.dev.yml up --build

curl http://localhost:8080/api/v1/health
```

Configure with `PORKPIE_DATABASE_URL`, `PORKPIE_SERVER_BIND`, and `PORKPIE_API_KEY`. Sync routes require `Authorization: Bearer {API_KEY}`.

See [infra/](infra/) for Dockerfiles, Caddy config, and compose files. See [docs/](docs/) for architecture, security invariants, sync protocol, data model, test plan, and roadmap.
