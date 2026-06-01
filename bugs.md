# Porkpie Desktop First-Use Bugfix Sprint

Generated: 2026-06-01T20:49:49+00:00

This worklist comes from real first-use testing of the Windows installer and desktop GUI.

The user successfully installed Porkpie, launched the app, created a vault, saved a recovery key, unlocked the vault using the recovery key, and saved a first login item. That means the core desktop flow is alive.

However, the desktop app currently feels like an app-shaped prototype. These issues must be fixed before more SSH/security/backend work. The user is sick of the command line. The next milestone is a smooth double-click desktop password manager.

---

## Global Binding Rules

Repository: `Cooked-Ham/Porkpie`

Hard rules:

1. Do not add new backend/security features during this sprint.
2. Do not introduce Electron.
3. Do not introduce React.
4. Do not introduce TypeScript/Vite.
5. Keep Dioxus/Rust desktop.
6. Do not break CLI tests.
7. Do not break installer packaging.
8. Do not fake file export by dumping huge JSON into the UI.
9. Do not ship white-on-white or black-on-white broken themes.
10. Do not make the user open a terminal for normal app usage.

Required validation:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

Windows validation:

```powershell
cargo build -p porkpie-desktop --release
cargo run -p porkpie-desktop
```

Installer validation:

```powershell
# Use the repo's actual installer build command.
# Then install the produced MSI/EXE on Windows and launch from Start Menu/Desktop.
```

Manual QA must be recorded in:

```text
docs/DESKTOP_WINDOWS_QA.md
```

---

# Severity Summary

## P0: Product-breaking

1. Desktop app launches with a blank console window.
2. Sidebar buttons open external browser windows to `file:///C:/Program Files/Porkpie/%23`.
3. After first-run vault creation, the UI says vault is locked when adding the first login.
4. White theme is unusable.

## P1: First-use trust breakers

5. Saving an item changes header title to literal `{title}`.
6. Encrypted export dumps a huge vertical code snippet instead of saving/downloading a file.
7. Recovery kit save flow still depends too much on clipboard/manual behavior.

## P2: Polish / UX

8. Export/import page still has stale `not implemented` language for features unrelated to current flow.
9. Sidebar should show only contextually useful actions during onboarding/unlock.
10. App needs one-click `Open data folder` / `Reset local dev vault` in GUI error state if not already working.

---

# Bug 1: Desktop App Opens a Console Window

## User Report

Installer works, but launching the app opens the GUI and also opens a blank console window.

## Likely Cause

The Windows desktop binary is being built as a console subsystem binary. For a GUI app, the desktop binary should use Windows subsystem mode in release/installer builds.

Current desktop entrypoint is normal `fn main()` with no Windows subsystem attribute.

## Required Fix

Add to the top of:

```text
apps/desktop/src/main.rs
```

```rust
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
```

This hides the console in release builds while preserving console output in debug builds.

Also verify the installer shortcut points to:

```text
porkpie-desktop.exe
```

not:

```text
porkpie.exe
```

or another console binary.

## Required Tests / QA

1. Build release desktop binary on Windows.
2. Launch from Explorer.
3. Launch from Start Menu shortcut.
4. Confirm no console window appears.
5. Debug `cargo run -p porkpie-desktop` may still show a console. That is acceptable.

## Acceptance Criteria

- Installed app launches without a console window.
- Installer shortcut targets the GUI binary.
- Debug builds remain developer-friendly.

---

# Bug 2: Sidebar Buttons Open Browser Windows to `file:///.../%23`

## User Report

Clicking sidebar buttons opens a browser window saying file not found:

```text
file:///C:/Program%20Files/Porkpie/%23
```

## Root Cause

The app uses anchor tags with `href="#"` for navigation.

Known source pattern:

```rust
a {
    href: "#",
    onclick: move |_| state.with_mut(|s| s.screen = target.clone()),
    "{label}"
}
```

In Dioxus Desktop/WebView, anchor navigation can be treated as external navigation from the local app path. That opens the browser to a nonsense `file:///.../#` route.

There are additional raw anchors:

```rust
a { class: "btn btn-secondary", href: "#unlock", "Open existing" }
a { class: "btn btn-secondary", href: "#onboarding", "Create new" }
```

## Required Fix

Replace all app-navigation anchors with real buttons.

### 1. Change `NavLink`

In `crates/porkpie-ui/src/app.rs`, replace:

```rust
a {
    href: "#",
    onclick: move |_| state.with_mut(|s| s.screen = target.clone()),
    "{label}"
}
```

with:

```rust
button {
    class: "nav-link",
    r#type: "button",
    onclick: move |_| state.with_mut(|s| s.screen = target.clone()),
    "{label}"
}
```

or reuse the existing `Button` component.

### 2. Update CSS

Replace:

```css
.nav a
.nav a:focus-visible
```

with:

```css
.nav button,
.nav-link
.nav button:focus-visible,
.nav-link:focus-visible
```

Ensure the visual style stays identical.

### 3. Replace onboarding/unlock anchors

In `onboarding.rs`, replace `Open existing` anchor with a button that sets:

```rust
state.screen = Screen::Unlock
```

In `unlock.rs`, replace `Create new` anchor with a button that sets:

```rust
state.screen = Screen::Onboarding
```

### 4. Search and remove local navigation anchors

Search for:

```text
href: "#"
href: "#unlock"
href: "#onboarding"
```

There should be no internal navigation anchors left.

## Required Tests / QA

Manual QA:

1. Launch installed app.
2. Click every sidebar item.
3. Click `Open existing`.
4. Click `Create new`.
5. Confirm no external browser opens.
6. Confirm screen changes in-app.

## Acceptance Criteria

- No browser windows open from app navigation.
- No `file:///.../%23` behavior remains.
- Navigation remains keyboard accessible.

---

# Bug 3: First Login Flow Says Vault Is Locked After Onboarding

## User Report

After saving recovery key, trying to add the first new login said the vault was locked. User had to unlock manually using the key from recovery kit, then could save the first item.

## Likely Cause

Onboarding creates the vault, stores recovery kit, and navigates to `Screen::List`, but does not set `unlocked_handle`.

Current onboarding completion appears to set:

```rust
state.vaults.push(summary.clone());
state.current_vault = Some(summary);
state.items.clear();
state.screen = Screen::List;
state.status = Some("Vault created. Save your recovery kit before locking.");
```

But it does not set the unlocked vault handle. That leaves the UI looking like a vault exists but actions requiring `unlocked_handle` fail with `Vault is locked`.

## Required Fix

After successful vault creation, either:

### Preferred

Keep the vault unlocked automatically after onboarding.

`VaultBackend::create_vault(...)` should return an unlocked handle or onboarding should immediately unlock the newly created vault using the already-known password and local secret key.

State after recovery kit confirmation must include:

```rust
state.current_vault = Some(summary);
state.unlocked_handle = Some(handle);
state.items = vec![];
state.screen = Screen::List;
```

### Alternative

After recovery kit confirmation, route to Unlock screen with a clear message:

```text
Vault created. Unlock it to start adding items.
```

Preferred UX is to land in an unlocked empty vault.

## Required Tests

Add a GUI/backend state test:

1. Create vault through onboarding path.
2. Confirm recovery kit saved.
3. Assert `state.unlocked_handle.is_some()`.
4. Assert `Screen::List`.
5. Add first Login item without manual unlock.
6. Assert item saves and appears in list.

## Manual QA

1. Fresh install / fresh DB.
2. Create vault.
3. Save recovery kit.
4. Click Add Login.
5. Save first login.
6. Confirm no `Vault is locked` error.

## Acceptance Criteria

- First-run onboarding lands in an unlocked empty vault.
- User can add first login immediately after saving recovery kit.
- No manual recovery-key unlock is required in normal first-run flow.

---

# Bug 4: Saved Item Header Shows Literal `{title}`

## User Report

After saving a login, the title near the top of the item detail screen switches to:

```text
{title}
```

under `Item detail`.

## Root Cause

The Dioxus render code literally contains:

```rust
h1 { if is_new { "New item" } else { "{title}" } }
```

Inside that conditional string, `"{title}"` is rendered literally instead of interpolating.

## Required Fix

Replace with one of:

```rust
h1 { if is_new { "New item" } else { title.as_str() } }
```

or precompute:

```rust
let heading = if is_new || title.trim().is_empty() {
    "Untitled item".to_string()
} else {
    title.clone()
};
```

then:

```rust
h1 { "{heading}" }
```

## Required Tests / QA

Manual QA:

1. Add login named `GitHub`.
2. Save.
3. Detail header should show `GitHub`, not `{title}`.
4. Edit title to `GitLab`.
5. Save.
6. Detail header should show `GitLab`.

## Acceptance Criteria

- Literal `{title}` never appears in item detail.
- Header updates after save/edit.

---

# Bug 5: Encrypted Export Dumps an Unusable Vertical Code Snippet

## User Report

With one saved password, encrypted export generates an insanely long vertical code snippet. Plaintext export works great.

## Root Cause

The encrypted export sets:

```rust
export_handle.set(Some(export.json));
```

then renders:

```rust
pre { class: "generated", "{text}" }
```

The CSS for `.generated` uses:

```css
overflow-wrap: anywhere;
```

This makes long encrypted/base64/JSON payloads wrap brutally. Showing encrypted backup payload inline is bad UX. Users want a file, not a scroll monument.

## Required Fix

### 1. Use file-save flow for encrypted export

On desktop, encrypted export should save directly to a file.

Use a native file dialog if possible, likely via `rfd` crate:

```rust
rfd::FileDialog::new()
    .set_title("Save encrypted Porkpie backup")
    .set_file_name(&export.suggested_filename)
    .add_filter("Porkpie backup", &["json"])
    .save_file()
```

Then write `export.json` to the selected path.

If no file dialog is implemented yet, at minimum:

- provide `Copy encrypted backup`,
- provide `Save encrypted backup` once dialog works,
- do not render the whole payload inline by default.

### 2. Change UI behavior

After export:

```text
Encrypted backup saved to C:\Users\...\backup.json
```

No giant inline payload.

### 3. Optional advanced disclosure

If inline viewing remains, hide behind:

```text
Show raw backup JSON
```

and render it in a textarea with max height:

```css
.backup-payload {
  max-height: 280px;
  overflow: auto;
  white-space: pre;
  overflow-wrap: normal;
  font-family: ui-monospace, Consolas, monospace;
}
```

## Required Tests / QA

Manual QA:

1. Create one login.
2. Click Export encrypted.
3. Save dialog opens.
4. Save file.
5. File exists.
6. Import from that file works.
7. No huge inline vertical payload appears by default.

## Acceptance Criteria

- Encrypted export produces a usable file.
- UI does not dump long payload inline by default.
- Plaintext export remains explicitly dangerous and confirmed.

---

# Bug 6: White Theme Is Unusable

## User Report

White theme has large black contrasting areas and text that is white on white backgrounds.

## Likely Causes

Theme CSS mixes hardcoded dark colors with CSS variables. Known examples:

```css
.sidebar { background: #121419; }
[data-theme="light"] .sidebar { background: #ffffff; }
```

Some components may still hardcode dark/light values or use `btn-primary` accent colors poorly. Also browser/OS `prefers-color-scheme` rules may override theme variables or interact badly with `data-theme`.

## Required Fix

### 1. Remove hardcoded dark colors from app surfaces

Search for hardcoded colors:

```text
#101114
#121419
#181b20
#20242b
#f3f6f8
#ffffff
black
white
rgba(0,0,0
```

Every UI surface/text should use variables:

```css
--bg
--surface
--surface-2
--surface-3
--text
--muted
--line
--accent
--accent-ink
--danger
--shadow
--modal-backdrop
```

### 2. Add complete light/dark theme variable sets

Define both explicit themes:

```css
[data-theme="dark"] { ... }
[data-theme="light"] { ... }
```

Do not rely on `prefers-color-scheme` to override an explicit app setting.

### 3. Fix modal, sidebar, form, table, input, toast, and disabled states

Light theme QA must cover:

- sidebar,
- workspace,
- panels,
- inputs,
- tables,
- modals,
- toasts,
- inline errors,
- disabled buttons,
- import/export payload areas.

## Acceptance Criteria

- Light theme has no white-on-white text.
- Light theme has no dark leftover panels unless intentionally designed.
- Dark theme still works.
- Theme switching updates all major UI areas live.

---

# Bug 7: Recovery Kit Save UX Still Feels Too Manual

## User Report

User successfully saved recovery key, but the flow should feel more like a real app.

## Current Behavior

The app copies recovery kit JSON to clipboard and tells the user to paste/save it. This is better than nothing, but not a finished desktop product.

## Required Fix

Add native save-file dialog for recovery kit.

Preferred:

```rust
rfd::FileDialog::new()
    .set_title("Save Porkpie recovery kit")
    .set_file_name(format!("{}_recovery_kit.json", summary.name))
    .add_filter("Recovery kit JSON", &["json"])
    .save_file()
```

Then write the recovery kit JSON directly.

After save, show:

```text
Recovery kit saved to <path>
```

Only enable `I have saved my recovery kit` after successful save, or provide a deliberate `I saved it another way` escape.

## Acceptance Criteria

- User can save recovery kit to a file from GUI.
- No clipboard-only recovery kit workflow.
- User cannot accidentally proceed without acknowledging recovery kit save.

---

# Bug 8: Import/Export Page Contains Stale Not-Implemented Noise

## Current UI

The import/export page contains:

```text
Not implemented yet: 1Password, Bitwarden, LastPass CSV importers, scheduled automatic backups, and remote sync to a self-hosted server.
```

This is not helpful in the first-use product flow. It makes the app feel unfinished even when the current action works.

## Required Fix

Move future importer/sync notes to docs/roadmap or a collapsed `Coming later` section.

The main app surface should focus on working actions:

- Export encrypted backup
- Export plaintext backup
- Import encrypted backup

## Acceptance Criteria

- First-use GUI does not display irrelevant `not implemented yet` warnings in the main flow.
- Roadmap remains documented elsewhere.

---

# Final Manual QA Script

Run this after fixes.

## Clean Install

1. Uninstall Porkpie.
2. Delete local app data DB if appropriate.
3. Install latest Porkpie installer.
4. Launch from Start Menu.
5. Confirm no console window.

## Onboarding

1. Create vault.
2. Save recovery kit using file dialog.
3. Confirm app lands in unlocked empty vault.
4. Confirm no browser window opens from sidebar.

## First Login

1. Click Items.
2. Click Add Login.
3. Enter:
   - title: GitHub
   - username: test@example.com
   - password: CorrectHorseBatteryStaple!123
   - URL: https://github.com
   - notes: first gui smoke test
4. Save.
5. Confirm header says `GitHub`, not `{title}`.
6. Confirm item appears in list.
7. Open item.
8. Copy password.
9. Lock vault.
10. Unlock vault.
11. Confirm item persists.
12. Close app.
13. Reopen app.
14. Unlock vault.
15. Confirm item persists.

## Export

1. Export encrypted backup.
2. Save file through file dialog.
3. Confirm no huge inline payload appears.
4. Export plaintext backup.
5. Confirm scary confirmation still appears.
6. Confirm plaintext output/file is readable.

## Theme

1. Switch dark theme.
2. Check all pages.
3. Switch light theme.
4. Check all pages.
5. Confirm no white-on-white text.
6. Confirm no black leftover panels.

## Navigation

1. Click every sidebar button.
2. Confirm no browser opens.
3. Confirm every screen changes inside the app.

---

# Definition of Done

This sprint is complete when:

- installed app opens without a console,
- sidebar navigation never opens browser windows,
- first-run onboarding leaves vault unlocked,
- first login can be created immediately,
- item detail header renders actual title,
- encrypted export saves a file instead of dumping giant wrapped text,
- recovery kit saves to a file,
- white theme is usable,
- manual QA result is recorded in `docs/DESKTOP_WINDOWS_QA.md`,
- validation passes.

Expected user experience after this sprint:

```text
Install Porkpie -> double-click icon -> create vault -> save recovery kit -> add login -> export backup -> close/reopen -> unlock -> everything still works.
```
