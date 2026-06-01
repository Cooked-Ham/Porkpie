# Porkpie Desktop UX Sprint Addendum: Settings, Onboarding, Stay Signed In, Tray

Generated: 2026-06-01T21:17:18+00:00

This addendum extends the Desktop First-Use Bugfix Sprint with additional real user feedback from Windows desktop testing.

The user is tired. This is relevant. A desktop password manager should reduce ceremony, not add more tiny rituals performed in a GUI-shaped trench coat.

---

## Global Binding Rules

Repository: `Cooked-Ham/Porkpie`

Hard rules:

1. Do not add new backend/security features unrelated to desktop usability.
2. Do not introduce Electron.
3. Do not introduce React.
4. Do not introduce TypeScript/Vite.
5. Keep Dioxus/Rust desktop.
6. Do not require command-line setup for normal use.
7. Do not ship broken light theme.
8. Do not keep onboarding as a permanent sidebar destination after setup.
9. Do not implement "stay signed in" by storing plaintext master passwords.
10. Do not silently weaken vault security.

Required validation:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
```

Windows desktop validation:

```powershell
cargo build -p porkpie-desktop --release
cargo run -p porkpie-desktop
```

Installer validation must confirm:

- app launches without console,
- Start Menu shortcut launches GUI,
- no terminal required,
- no browser windows from app navigation.

Manual QA must be recorded in:

```text
docs/DESKTOP_WINDOWS_QA.md
```

---

# New User Feedback

The user found these additional issues:

1. White text on white background still appears on the Settings page after enabling light theme.
2. `Onboarding` does not need to be permanently visible in the sidebar.
3. Onboarding should be a forced first-run experience only when no vault exists.
4. If all vaults are deleted, onboarding should come back.
5. App needs a `Stay signed in` option in Settings.
6. App needs minimize-to-tray and close-to-tray behavior.
7. The user is tired of setup ceremony and wants the app to behave like a normal desktop password manager.

---

# Bug 9: Light Theme Still Broken on Settings Page

## Severity

P0.

## User Report

White text on white background still appears on the Settings page after enabling white/light theme.

## Confirmed Source Context

`SettingsPage` renders a panel, fields, selects, backup row, status, error, and a raw anchor styled as a button. The settings page currently only exposes lock timeout and theme, meaning all styling on this page depends on shared CSS variables and component classes.

## Required Fix

### 1. Add settings-specific light-theme QA

The light theme must be checked on:

- Settings screen header,
- Lock timeout field,
- Theme select,
- About section,
- Encrypted backups row,
- Open import/export button,
- Lock vault button,
- toast/status,
- inline error,
- any disabled states.

### 2. Eliminate hardcoded text/surface colors

Search CSS and components for hardcoded colors:

```text
#ffffff
#000000
black
white
#121419
#101114
#181b20
#20242b
#f3f6f8
rgba(0,0,0
```

Every Settings surface/text should use theme variables:

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
--modal-backdrop
```

### 3. Fix `prefers-color-scheme` fighting explicit theme

If the app has an explicit `data-theme`, the CSS should not override selected theme via `@media (prefers-color-scheme: light)`.

Allowed behavior:

- use OS preference only to initialize the default setting,
- after user selects light/dark, `data-theme` wins.

### 4. Add theme smoke tests if practical

Add a simple test/assertion or snapshot-like check that Settings renders with:

```text
data-theme="light"
```

and does not use known broken classes or hardcoded colors.

Manual QA is required even if tests exist, because visual bugs like white-on-white are how CSS quietly humiliates everyone.

## Acceptance Criteria

- Settings page is readable in light theme.
- No white-on-white text.
- No dark leftover panels unless intentionally designed.
- Theme switch updates Settings live.
- `docs/DESKTOP_WINDOWS_QA.md` records Settings light-theme pass.

---

# Bug 10: Onboarding Should Not Be Permanent Sidebar Navigation

## Severity

P0/P1.

## User Report

The `Onboarding` button should not always appear in the sidebar. It should be the forced experience when the app starts without a vault. Then it should disappear unless all vaults are deleted.

## Current Problem

The sidebar currently always renders an `Onboarding` nav item whenever the app is not on the DB error screen.

That makes the app feel unfinished and lets users wander back into first-run setup after they already have a vault. Truly innovative: a password manager that keeps asking if it has been born yet.

## Required Behavior

### App startup

If no vaults exist:

```text
screen = Onboarding
sidebar = hidden or minimal
```

The user should be forced into onboarding.

If one or more vaults exist:

```text
screen = Unlock
sidebar = normal, but no Onboarding link
```

### After vault creation

Once a vault exists:

- Onboarding link disappears.
- User lands in unlocked vault list.
- User can add first item immediately.

### If all vaults are deleted

If vault deletion exists or is added:

- return to Onboarding,
- sidebar hides/minimizes,
- first-run experience returns.

If vault deletion does not exist yet:

- do not add this part now unless needed,
- but model the state logic so zero vaults means Onboarding.

## Required UI Changes

In `App` sidebar:

- Do not show `Onboarding` nav item when `state.vaults` is not empty.
- Consider hiding the whole sidebar during first-run onboarding.
- Keep Unlock visible only when vaults exist.
- Keep Items/Generator/Import/Export/Settings visible only after vault exists.
- Disable or hide item-related screens while locked if they require unlock.

Recommended sidebar states:

### No vault

```text
Porkpie
[Create vault] only, or no sidebar at all
```

### Locked vault exists

```text
Porkpie
Unlock
Settings
```

### Unlocked vault

```text
Porkpie
Items
Generator
Import/export
Settings
Lock
```

## Required Tests / QA

Manual QA:

1. Fresh install / no DB.
2. App opens directly to onboarding.
3. Sidebar does not show permanent Onboarding clutter.
4. Create vault.
5. Onboarding link disappears.
6. Close/reopen app.
7. App opens to Unlock, not Onboarding.
8. If all vaults are deleted, app returns to Onboarding.

## Acceptance Criteria

- Onboarding is a state, not a permanent app section.
- Onboarding appears only when needed.
- No normal user can accidentally re-enter first-run setup from sidebar after vault exists.

---

# Feature 1: Add `Stay Signed In` Setting

## Severity

P1.

## User Need

The app needs a `Stay signed in` option in Settings.

The goal is not to store the master password. The goal is to reduce repeated unlock ceremony on a trusted personal machine while staying sane about security.

## Required Security Rule

Never store:

- master password,
- plaintext vault key,
- plaintext item secrets.

Preferred implementation:

- keep local secret key in OS keychain,
- keep unlocked vault key only in process memory while app/tray process is running,
- optionally skip auto-lock while `Stay signed in` is enabled,
- lock again after reboot/process exit unless a deliberate OS-protected session design is implemented.

## Settings Model

Extend `SettingsState`:

```rust
pub struct SettingsState {
    pub lock_timeout_minutes: u16,
    pub theme: Theme,
    pub stay_signed_in: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub start_minimized_to_tray: bool,
}
```

Use conservative defaults:

```rust
stay_signed_in: false
minimize_to_tray: true
close_to_tray: false
start_minimized_to_tray: false
```

or whatever product decision is chosen, but document it.

## Required Settings UI

Add to Settings:

```text
Session
[ ] Stay signed in on this device
    Keeps the vault unlocked while Porkpie is running. Does not store your master password.

Lock timeout:
[ 5 / 15 / 30 / 60 / Never while app is running ]

Tray
[ ] Minimize to tray
[ ] Close to tray
[ ] Start minimized to tray
```

If `Stay signed in` is enabled, explain:

```text
Porkpie will keep your vault unlocked in memory while the app is running. Use only on a trusted device.
```

## Required Behavior

### Stay signed in OFF

- Current lock timeout behavior applies.
- Closing app exits and drops unlocked state.
- Reopen requires unlock.

### Stay signed in ON

- App does not auto-lock while process remains running, or uses a much longer timeout depending on product choice.
- Minimize/close-to-tray keeps the process alive and vault handle in memory.
- Reopening from tray shows the unlocked vault.
- Full app exit still drops unlocked state.
- Reboot requires unlock again.

## Required Tests

Add state tests:

- default `stay_signed_in` is false,
- enabling `stay_signed_in` changes timeout behavior,
- `AppState::lock()` still locks when explicitly invoked,
- full process restart does not preserve unlocked vault key,
- settings serialize/deserialize if persisted.

## Acceptance Criteria

- Settings page has `Stay signed in`.
- It does not store master password.
- Explicit Lock still locks.
- Full app exit still drops decrypted state.
- Behavior is documented.

---

# Feature 2: Minimize / Close to Tray

## Severity

P1.

## User Need

The app should behave like a normal password manager:

- minimize to tray,
- optionally close to tray,
- reopen quickly,
- keep session alive if stay-signed-in is enabled.

## Required Implementation

Use a Rust-compatible tray integration that works with Dioxus Desktop/Wry/Tao.

Agent must evaluate and implement one:

- `tray-icon`,
- Tao/Wry native tray APIs if available,
- Dioxus desktop window APIs if sufficient.

No Electron. Do not even think it. The door is locked.

## Required Tray Menu

Tray icon menu:

```text
Open Porkpie
Lock Vault
SSH Agent Status
Settings
Quit Porkpie
```

Minimum acceptable v1:

```text
Open Porkpie
Lock Vault
Quit Porkpie
```

## Window Behavior

### Minimize

If `minimize_to_tray` enabled:

- clicking minimize hides the window,
- tray icon remains,
- app process continues.

If disabled:

- minimize behaves normally.

### Close

If `close_to_tray` enabled:

- clicking X hides to tray,
- app process continues,
- unlocked state remains only if stay-signed-in is enabled.

If disabled:

- close exits app and drops unlocked state.

### Quit

Tray `Quit Porkpie` always exits process and drops unlocked state.

### Lock Vault

Tray `Lock Vault` calls `AppState::lock()` or equivalent command into UI/app state.

## Required Security Behavior

- If vault is unlocked and `stay_signed_in` is false, close-to-tray should lock immediately or warn.
- If vault is unlocked and `stay_signed_in` is true, close-to-tray may keep it unlocked in memory.
- Tray menu must never display secret values.
- Tray notifications must not contain secret values.

## Required Tests / QA

Some tray behavior requires manual Windows QA.

Record in:

```text
docs/DESKTOP_WINDOWS_QA.md
```

Manual QA:

1. Enable minimize to tray.
2. Minimize window.
3. Confirm window disappears and tray icon remains.
4. Click tray Open.
5. Confirm window returns.
6. Enable close to tray.
7. Click X.
8. Confirm app hides instead of exits.
9. Click tray Open.
10. Confirm app returns.
11. Click tray Lock Vault.
12. Confirm vault locks.
13. Click tray Quit.
14. Confirm process exits.

## Acceptance Criteria

- Tray icon exists on Windows.
- Minimize-to-tray works.
- Close-to-tray works if enabled.
- Tray Open works.
- Tray Quit works.
- Explicit Lock works.
- Behavior respects Stay Signed In setting.

---

# Feature 3: Persist Settings

## Severity

P1.

## Problem

Settings currently live in `AppState` only unless another persistence layer exists. Stay-signed-in/tray/theme must survive app restart.

## Required Fix

Persist app settings to a non-secret settings file.

Suggested path:

```text
%APPDATA%\Porkpie\settings.json
```

or:

```text
%LOCALAPPDATA%\Porkpie\settings.json
```

This file may contain:

- theme,
- lock timeout,
- stay signed in preference,
- tray preferences,
- last selected vault ID/name.

It must not contain:

- master password,
- local secret key,
- vault key,
- item secrets,
- API keys,
- SSH private keys.

## Required Behavior

On app startup:

1. load settings file,
2. apply theme immediately,
3. decide startup screen based on vault existence and settings,
4. if settings missing/corrupt, use defaults and show nonfatal warning.

On settings change:

- save settings automatically,
- debounce writes if needed.

## Required Tests

- default settings serialize,
- saved theme reloads,
- stay-signed-in reloads,
- corrupt settings file falls back safely,
- settings file does not contain secret fields.

## Acceptance Criteria

- Theme persists.
- Stay-signed-in persists.
- Tray prefs persist.
- No secrets in settings file.

---

# Updated Desktop QA Checklist

Add these to `docs/DESKTOP_WINDOWS_QA.md`.

## Settings Light Theme

1. Open Settings.
2. Switch to Light.
3. Confirm all text readable.
4. Confirm no white-on-white.
5. Confirm About section readable.
6. Confirm buttons readable.
7. Confirm toast/error readable.

## Onboarding Visibility

1. Start with no vault.
2. Confirm onboarding appears.
3. Confirm permanent Onboarding sidebar button is absent or minimal.
4. Create vault.
5. Confirm Onboarding disappears.
6. Restart app.
7. Confirm app opens to Unlock/List flow, not Onboarding.

## Stay Signed In

1. Unlock vault.
2. Enable Stay signed in.
3. Minimize app.
4. Restore app.
5. Confirm still unlocked.
6. Close to tray if enabled.
7. Restore from tray.
8. Confirm still unlocked.
9. Quit app fully.
10. Reopen app.
11. Confirm unlock is required unless a deliberate persistent OS-protected unlock model is implemented.

## Tray

1. Enable minimize to tray.
2. Minimize.
3. Confirm tray icon exists.
4. Restore from tray.
5. Enable close to tray.
6. Click X.
7. Restore from tray.
8. Lock from tray.
9. Quit from tray.

---

# Definition of Done

This addendum is complete when:

- Settings page is readable in light theme.
- Onboarding only appears when no vault exists.
- Sidebar is contextual to app state.
- Stay signed in setting exists and behaves safely.
- Minimize-to-tray works.
- Close-to-tray works if enabled.
- Settings persist across restart.
- Manual Windows QA is recorded.
- Strict validation passes.

Expected user experience:

```text
Install Porkpie -> open app -> create vault once -> onboarding disappears -> use app normally -> stay signed in if desired -> minimize/close to tray like a real password manager.
```
