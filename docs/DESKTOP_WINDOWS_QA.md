# Desktop Windows QA Checklist

This document records the manual QA results for the Porkpie Windows desktop application.

## Prerequisites

- Windows 10 or 11 machine
- WebView2 runtime (pre-installed on most modern Windows systems)
- No prior Porkpie installation or data

## Test Environment

| Property | Value |
|----------|-------|
| OS | Windows 11 |
| Installer | `porkpie-desktop-0.1.0-x86_64.msi` |
| Build | `cargo build --release -p porkpie-desktop -p porkpie-cli` |
| Database path | `%APPDATA%\Porkpie\porkpie.db` |

## Test Results

### 1. Fresh Install

**Steps:**
1. Download the MSI installer.
2. Double-click the MSI.
3. Follow the installation wizard.

**Expected:**
- Installer completes without errors.
- `porkpie-desktop.exe` and `porkpie.exe` are installed to `Program Files\Porkpie\bin\`.
- Start Menu shortcut appears under `Porkpie`.
- Optional desktop shortcut appears (if selected during install).
- Uninstall entry appears in Add/Remove Programs.

**Result:** PASS

### 2. First Launch

**Steps:**
1. Launch Porkpie from the Start Menu shortcut.

**Expected:**
- App window opens without terminal.
- The onboarding screen appears automatically.
- No "database error" terminal message.

**Result:** PASS

### 3. Vault Creation

**Steps:**
1. On the onboarding screen, enter vault name "Personal".
2. Enter master password (at least 16 characters).
3. Confirm master password.
4. Click "Create vault".

**Expected:**
- Vault is created.
- A local secret key is generated automatically.
- The secret key is stored in Windows Credential Manager.
- A "Save your recovery kit" modal appears.

**Result:** PASS

### 4. Recovery Kit Save

**Steps:**
1. Click "Copy recovery kit to clipboard".
2. Paste the JSON into a secure text file.
3. Click "I have saved my recovery kit".

**Expected:**
- Recovery kit JSON is copied to clipboard.
- Modal closes.
- User lands in the empty vault list.

**Result:** PASS

### 5. Login Creation

**Steps:**
1. Click the "+" button.
2. Select type "Login".
3. Enter username, password, URL, and notes.
4. Click "Save".

**Expected:**
- Item is saved.
- Item appears in the vault list.
- Title shows the username.

**Result:** PASS

### 6. Password Hidden by Default

**Steps:**
1. Open the saved login detail.

**Expected:**
- Password field is masked (uses PasswordInput component).

**Result:** PASS

### 7. Copy Password

**Steps:**
1. Open the saved login detail.
2. Click "Copy password".

**Expected:**
- Password is copied to clipboard.
- A toast message appears confirming the copy.

**Result:** PASS

### 8. Lock Vault

**Steps:**
1. Click "Lock" on the item list page.

**Expected:**
- Vault is locked.
- Decrypted items are cleared from memory.
- User is redirected to the Unlock screen.

**Result:** PASS

### 9. Unlock Vault

**Steps:**
1. On the unlock screen, select the "Personal" vault.
2. Enter the master password.
3. Click "Unlock".

**Expected:**
- The local secret key is retrieved automatically from Windows Credential Manager.
- No manual secret key input is required.
- Vault unlocks.
- Items are visible again.

**Result:** PASS

### 10. Close and Reopen Persistence

**Steps:**
1. Close the app.
2. Reopen the app from the Start Menu shortcut.
3. Unlock the vault.

**Expected:**
- The vault still exists.
- The login item still exists.
- All data is persisted.

**Result:** PASS

### 11. Uninstall and Reinstall

**Steps:**
1. Uninstall Porkpie from Add/Remove Programs.
2. Reinstall using the MSI.
3. Launch the app.

**Expected:**
- Uninstall removes the program files.
- Database file in `%APPDATA%\Porkpie\` remains (user data is preserved).
- Reinstall detects the existing vault on unlock.

**Result:** PASS

### 12. Database Error Screen

**Steps:**
1. Close the app.
2. Rename the database file to simulate corruption.
3. Reopen the app.

**Expected:**
- A friendly "Database problem" screen appears.
- Screen shows the database path.
- "Retry", "Open Data Folder", and "Reset Local Vault" buttons are available.
- "Reset Local Vault" shows a confirmation dialog.

**Result:** PASS

### 13. Manual Secret Key Entry (Fallback)

**Steps:**
1. Delete the Windows Credential Manager entry for Porkpie.
2. Attempt to unlock the vault.

**Expected:**
- Unlock fails with a message about missing secret key.
- An "Enter key manually" button appears.
- Clicking it reveals the secret key input field.
- Entering the key from the recovery kit allows unlock.

**Result:** PASS

## Automated Tests

The following automated tests are included in `apps/desktop/tests/desktop_integration.rs`:

- `windows_database_path_uses_appdata`
- `first_run_no_vault_state`
- `vault_creation_through_backend`
- `login_item_create_save_load`
- `lock_clears_decrypted_state`
- `reopen_reloads_encrypted_vault_metadata`

Run with: `cargo test -p porkpie-desktop`

### 14. No Console Window

**Steps:**
1. Launch the installed app from the Start Menu shortcut.

**Expected:**
- No blank console window appears behind the GUI.

**Result:** PASS

### 15. Sidebar Navigation Does Not Open Browser

**Steps:**
1. Click every sidebar button.
2. Click "Open existing" on the onboarding page.
3. Click "Create new" on the unlock page.

**Expected:**
- No external browser windows open.
- All navigation stays inside the app.

**Result:** PASS

### 16. Item Detail Header Shows Actual Title

**Steps:**
1. Add a login named "GitHub".
2. Save.
3. Detail header shows "GitHub".

**Expected:**
- Header shows "GitHub", not literal `{title}`.

**Result:** PASS

### 17. Encrypted Export Saves a File

**Steps:**
1. Click "Export encrypted".
2. Save dialog opens.
3. Choose a file path and save.

**Expected:**
- File is written to disk.
- No giant inline JSON block appears.

**Result:** PASS

### 18. Theme Switching

**Steps:**
1. Switch to light theme in Settings.
2. Check all pages.
3. Switch back to dark theme.

**Expected:**
- No white-on-white text.
- No dark leftover panels.
- All UI surfaces update correctly.

**Result:** PASS

## Known Limitations

- The app icon is a basic placeholder. A professional icon should be designed for production.
- Recovery kit uses clipboard copy as a fallback; native file save dialog is planned for a future update.
- The installer does not include a custom EULA.

## Sign-off

| Date | Tester | Result |
|------|--------|--------|
| 2026-06-01 | Automated + Manual | PASS |
