# Porkpie Desktop Product Gate

## Goal

Porkpie must become a double-clickable Windows desktop app with a complete first-run vault setup and first-login flow.

No new backend/security/SSH features until this gate passes.

## Required User Flow

On a clean Windows machine:

1. Install Porkpie using an `.exe` or `.msi`.
2. Launch Porkpie from Start Menu or desktop shortcut.
3. First-run onboarding appears automatically.
4. User creates a new vault.
5. User chooses a master password.
6. App generates local secret key.
7. App stores local secret key in Windows Credential Manager.
8. App displays recovery kit and forces user to save it.
9. User lands in empty vault.
10. User clicks "Add Login."
11. User enters title, username, password, URL, and notes.
12. User saves login.
13. Login appears in vault list.
14. User opens login detail.
15. Password is hidden by default.
16. User clicks Copy Password.
17. User locks vault.
18. User unlocks vault.
19. Login still exists.
20. User closes app.
21. User reopens app by double-clicking.
22. User unlocks vault.
23. Login still exists.

## Hard Rules

- No terminal required.
- No manual env vars.
- No cargo commands.
- No CLI-only setup.
- No local secret key copy/paste during normal onboarding.
- No fake/demo data in normal mode.
- No "coming soon" buttons in the main path.
- No Electron.
- No React.
- No TypeScript/Vite frontend.
- Keep Dioxus/Rust desktop.

## Installer Requirements

Build a Windows installer that includes:

- `porkpie-desktop.exe`
- `porkpie.exe` CLI if needed internally
- Start Menu shortcut
- optional desktop shortcut
- uninstall support
- app icon
- version metadata
- default app data directory setup

## Recommended packaging options to evaluate:

- WiX Toolset MSI
- NSIS installer
- cargo-wix
- cargo-bundle if suitable

Pick one and implement it. Do not stop at "packaging later."

## App Data Requirements

The desktop app must use a normal Windows app data path:

%APPDATA%\Porkpie\porkpie.db

or:

%LOCALAPPDATA%\Porkpie\porkpie.db

Document the choice.

The app must create the directory automatically.

GUI Error Handling

No raw database panic nonsense.

If DB open fails, show a friendly error screen with:

database path
sanitized error message
"Retry"
"Open Data Folder"
"Reset Local Vault" behind confirmation
Manual QA Required

Record results in:

docs/DESKTOP_WINDOWS_QA.md

Include:

fresh install
first launch
vault creation
recovery kit save
login creation
lock/unlock
close/reopen persistence
uninstall/reinstall behavior
screenshot list or text transcript
Automated Tests

Add tests for:

desktop DB path construction on Windows
first-run no-vault state
vault creation through backend
login item create/save/load
lock clears decrypted state
reopen reloads encrypted vault metadata
Definition of Done

This gate passes only when a non-developer can:

Install Porkpie → double-click icon → create vault → add login → close/reopen → unlock → see login

without opening a terminal.


## My actual recommendation

Stop treating SSH agent as the flagship for a minute.

The flagship should be:

Porkpie opens.
Porkpie creates a vault.
Porkpie stores a login.
Porkpie survives restart.