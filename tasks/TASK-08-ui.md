---
task_id: 08-ui
task_name: User Interface (Dioxus)
sequence: 8
dependencies_complete: [01-workspace, 03-types, 04-crypto, 05-vault-core, 06-storage]
estimated_duration: 4-5 hours
difficulty: High
blockers_resolved: none
can_parallelize: false
---

# Task 8: User Interface (Dioxus)

## 🎯 Objective

Implement `porkpie-ui` crate with Dioxus-based desktop/web UI. Share vault/crypto/store code with CLI.

## ✅ Acceptance Criteria

**Screens Implemented**
- [ ] Onboarding screen (create vault)
- [ ] Unlock screen (password entry)
- [ ] Item list screen (searchable)
- [ ] Item detail screen (view/edit)
- [ ] Password generator screen
- [ ] Import/export screen
- [ ] Settings screen

**Onboarding Screen**
- [ ] Welcome message
- [ ] Master password input (password field, not visible)
- [ ] Confirm password field
- [ ] Create button
- [ ] Validation: min 16 chars, match confirmation

**Unlock Screen**
- [ ] Vault ID input (or dropdown of known vaults)
- [ ] Master password input
- [ ] Unlock button
- [ ] Error message on wrong password

**Item List Screen**
- [ ] Display all items in table/list
- [ ] Columns: Type, Title, Created Date
- [ ] Search/filter by title or type
- [ ] Click item → detail view
- [ ] Add button (+ icon)
- [ ] Logout button (lock vault)

**Item Detail Screen**
- [ ] Show all fields for item type
- [ ] Edit button (toggle edit mode)
- [ ] Save/Cancel buttons (edit mode)
- [ ] Delete button (with confirmation)
- [ ] Copy-to-clipboard for secret fields
- [ ] Back button to list

**Password Generator Screen**
- [ ] Length slider (8-128)
- [ ] Toggles: Uppercase, Lowercase, Numbers, Symbols
- [ ] Generate button
- [ ] Display generated password
- [ ] Copy button

**Import/Export Screen**
- [ ] Export button → Save encrypted backup
- [ ] Import button → Load backup file
- [ ] File dialog (native OS picker)
- [ ] Success/error messages

**Settings Screen**
- [ ] Logout button (lock vault)
- [ ] Lock timeout (5, 15, 30 min)
- [ ] Theme (light/dark)
- [ ] About section

**State Management**
- [ ] App-level state: current vault, unlock status
- [ ] Session tracking (locked/unlocked)
- [ ] Lock on timeout (configurable)
- [ ] Lock on window close

**Styling**
- [ ] Clean, modern design
- [ ] Dark mode support
- [ ] Responsive layout
- [ ] Keyboard shortcuts (Tab, Enter, Escape)

**Error Handling**
- [ ] Validation errors shown inline
- [ ] Network errors (if syncing) shown in toast
- [ ] Graceful error recovery

**Tests**
- [ ] Component rendering tests
- [ ] State management tests
- [ ] Navigation tests
- [ ] Form validation tests

**Code Quality**
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes (zero warnings)
- [ ] All components documented

## 📋 Output Specification

### File Structure

```
crates/porkpie-ui/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # App entry point
│   ├── app.rs                  # Root app component
│   ├── state.rs                # Global app state
│   ├── components/
│   │   ├── mod.rs
│   │   ├── button.rs           # Reusable button
│   │   ├── text_input.rs       # Reusable text input
│   │   ├── password_input.rs   # Password field
│   │   ├── item_list.rs        # Item list display
│   │   ├── form.rs             # Generic form
│   │   └── modal.rs            # Modal dialog
│   ├── pages/
│   │   ├── mod.rs
│   │   ├── onboarding.rs       # Create vault
│   │   ├── unlock.rs           # Unlock vault
│   │   ├── list.rs             # Item list
│   │   ├── detail.rs           # Item detail
│   │   ├── password_gen.rs     # Password generator
│   │   ├── import_export.rs    # Backup UI
│   │   └── settings.rs         # Settings
│   └── utils/
│       ├── mod.rs
│       ├── validation.rs       # Form validation
│       └── clipboard.rs        # Copy to clipboard
└── tests/
    ├── components.rs           # Component tests
    └── navigation.rs           # Navigation tests
```

### Cargo.toml

```toml
[package]
name = "porkpie-ui"
version = "0.1.0"
edition = "2021"

[dependencies]
porkpie-types = { path = "../porkpie-types" }
porkpie-core = { path = "../porkpie-core" }
porkpie-store = { path = "../porkpie-store" }
dioxus = { version = "0.4", features = ["web", "router"] }
dioxus-router = "0.4"
dioxus-hooks = "0.4"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"

[dev-dependencies]
```

### Example: App Root (`src/app.rs`)

```rust
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(RootLayout)]
        #[route("/")]
        Onboarding {},
        #[route("/unlock")]
        Unlock {},
        #[route("/list")]
        List {},
        #[route("/item/:id")]
        ItemDetail { id: String },
        #[route("/password-gen")]
        PasswordGen {},
        #[route("/settings")]
        Settings {},
}

fn RootLayout() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}

pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
```

### Example: Unlock Page (`src/pages/unlock.rs`)

```rust
use dioxus::prelude::*;

pub fn UnlockPage() -> Element {
    let mut vault_id = use_state(|| String::new());
    let mut password = use_state(|| String::new());
    let mut error = use_state(|| String::new());

    let mut handle_unlock = move |_| {
        // Load vault from SQLite
        // Call vault.unlock(password)
        // On success: navigate to /list
        // On error: show error message
        spawn(async move {
            match porkpie_store::load_vault(&vault_id).await {
                Ok(_) => {
                    // TODO: Persist session
                    // TODO: Navigate to /list
                },
                Err(e) => error.set(format!("Unlock failed: {}", e)),
            }
        });
    };

    rsx! {
        div { class: "container",
            h1 { "🔓 Unlock Vault" }
            
            input {
                class: "form-input",
                value: "{vault_id}",
                placeholder: "Vault ID",
                oninput: move |evt| vault_id.set(evt.value())
            }
            
            input {
                class: "form-input",
                input_type: "password",
                value: "{password}",
                placeholder: "Master password",
                oninput: move |evt| password.set(evt.value())
            }
            
            if !error.is_empty() {
                div { class: "error", "{error}" }
            }
            
            button {
                onclick: handle_unlock,
                "Unlock"
            }
        }
    }
}
```

## 🔗 References

- **Vault Core:** Task 5 (porkpie-core)
- **Storage:** Task 6 (porkpie-store)
- **Dioxus Docs:** https://dioxuslabs.com/

## ✔️ Success Verification

```bash
# Build (web target)
cargo build --package porkpie-ui --target wasm32-unknown-unknown

# Tests
cargo test --package porkpie-ui

# Lint
cargo clippy --package porkpie-ui -- -D warnings

# Format
cargo fmt --package porkpie-ui --check
```

## 🚨 If Blocked...

| Problem | Solution |
|---------|----------|
| "Dioxus is new" | Dioxus = React-like for Rust. Use `rsx!` macro for JSX-like syntax. |
| "State management complex" | Use `use_state()` hook for component state. Use global state for app-level data. |
| "Routing confusing" | Use `Router::<Route>` and enum `#[derive(Routable)]` for pages. |
| "Form validation tedious" | Create reusable `TextInput`, `PasswordInput` components with built-in validation. |
| "Password field showing text" | Use `input_type: "password"` for password masking. |

## 🔒 STRICT TYPECHECK REQUIREMENTS

**Type safety is non-negotiable.** Rust's type system is your first line of defense.

- ✓ **All type errors must compile** — `cargo build` must succeed with zero type errors
- ✓ **No `unsafe` blocks without justification** — Document why in code comment
- ✓ **No unchecked casts** — Use `as` only where necessary (document reasoning)
- ✓ **No `unwrap()` on external input** — Use `.map_err()` or `?` operator
- ✓ **No `todo!()` or `unimplemented!()` in production code** — Only in stubs
- ✓ **Compiler warnings are failures** — `cargo clippy` must have zero warnings
- ✓ **Type inference must be clear** — Add explicit types where ambiguous
- ✓ **Trait bounds must be explicit** — Don't hide requirements

**Verification command:**
```bash
cargo check --workspace
cargo build --workspace
```

**If ANY type error appears, stop and fix it. Type errors = broken code.**

## 📌 What Comes Next

**Task 9: Sync Protocol and HTTP API**

Next agent will implement sync. UI will call sync API to push/pull changes.

---

**Status:** Ready for agent assignment
