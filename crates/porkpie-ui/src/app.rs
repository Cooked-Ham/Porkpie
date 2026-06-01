use crate::pages::{
    ImportExportPage, ItemDetailPage, ItemListPage, OnboardingPage, PasswordGeneratorPage,
    SettingsPage, UnlockPage,
};
use crate::state::AppState;
#[cfg(not(target_arch = "wasm32"))]
use crate::state::Screen;
use crate::vault_store::VaultBackend;
use dioxus::prelude::*;

const APP_CSS: &str = r#"
:root {
  color-scheme: dark light;
  --bg: #101114;
  --surface: #181b20;
  --surface-2: #20242b;
  --text: #f3f6f8;
  --muted: #9aa7b4;
  --line: #313842;
  --accent: #4cc9a6;
  --accent-ink: #071612;
  --danger: #ff6b6b;
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--text); font-family: Inter, ui-sans-serif, system-ui, sans-serif; }
a { color: inherit; text-decoration: none; }
.app-shell { min-height: 100vh; display: grid; grid-template-columns: 260px minmax(0, 1fr); }
.sidebar { border-right: 1px solid var(--line); padding: 24px; background: #121419; position: sticky; top: 0; height: 100vh; }
.brand { font-size: 1.35rem; font-weight: 800; margin-bottom: 28px; }
.nav { display: grid; gap: 8px; }
.nav a, .btn, .icon-btn { min-height: 40px; border-radius: 8px; border: 1px solid var(--line); display: inline-flex; align-items: center; justify-content: center; padding: 0 14px; font-weight: 700; }
.nav a { justify-content: flex-start; color: var(--muted); background: transparent; }
.nav a:focus-visible, .btn:focus-visible, .icon-btn:focus-visible, input:focus-visible, select:focus-visible, textarea:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
.workspace { padding: 28px; display: grid; gap: 28px; max-width: 1180px; width: 100%; }
.screen { scroll-margin-top: 20px; }
.screen-header { margin-bottom: 16px; }
.split { display: flex; justify-content: space-between; gap: 12px; align-items: center; }
.eyebrow { color: var(--accent); text-transform: uppercase; letter-spacing: 0; font-size: .78rem; font-weight: 800; margin: 0 0 6px; }
h1, h2, p { margin-top: 0; }
h1 { font-size: clamp(1.65rem, 2.5vw, 2.35rem); margin-bottom: 8px; }
h2 { font-size: 1rem; margin-bottom: 6px; }
.muted { color: var(--muted); }
.panel { background: var(--surface); border: 1px solid var(--line); border-radius: 8px; padding: 20px; }
.form-grid { display: grid; gap: 16px; }
.two-col { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.full { grid-column: 1 / -1; }
.field { display: grid; gap: 8px; font-weight: 700; color: var(--muted); }
.input, .range { width: 100%; min-height: 42px; border-radius: 8px; border: 1px solid var(--line); background: var(--surface-2); color: var(--text); padding: 10px 12px; }
.textarea { min-height: 96px; resize: vertical; }
.actions { display: flex; gap: 10px; flex-wrap: wrap; align-items: center; }
.btn { cursor: pointer; background: var(--surface-2); color: var(--text); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-primary { background: var(--accent); color: var(--accent-ink); border-color: var(--accent); }
.btn-secondary { background: var(--surface-2); }
.btn-danger { background: transparent; color: var(--danger); border-color: var(--danger); }
.icon-btn { width: 42px; padding: 0; font-size: 1.3rem; background: var(--accent); color: var(--accent-ink); border-color: var(--accent); }
.inline-error { color: var(--danger); font-weight: 700; }
.list-panel { overflow-x: auto; }
.item-table { width: 100%; border-collapse: collapse; margin-top: 16px; min-width: 620px; }
.item-table th, .item-table td { border-bottom: 1px solid var(--line); padding: 12px; text-align: left; }
.item-table th { color: var(--muted); font-size: .8rem; text-transform: uppercase; letter-spacing: 0; }
.toggle-grid { display: grid; grid-template-columns: repeat(4, minmax(120px, 1fr)); gap: 10px; }
.toggle-grid label { background: var(--surface-2); border: 1px solid var(--line); border-radius: 8px; min-height: 42px; display: flex; align-items: center; gap: 8px; padding: 0 12px; }
.generated { display: block; overflow-wrap: anywhere; font: 700 1rem ui-monospace, SFMono-Regular, Consolas, monospace; background: var(--surface-2); border: 1px solid var(--line); border-radius: 8px; padding: 14px; }
.backup-row { display: flex; justify-content: space-between; gap: 16px; align-items: center; border-bottom: 1px solid var(--line); padding-bottom: 16px; }
.toast { border-left: 4px solid var(--accent); padding: 12px; background: var(--surface-2); color: var(--muted); }
.notice { border-left: 4px solid var(--muted); padding: 12px; background: var(--surface-2); color: var(--muted); }
.modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal { background: var(--surface); border: 1px solid var(--line); border-radius: 12px; padding: 24px; max-width: 540px; width: 100%; display: grid; gap: 16px; }
.empty-state { color: var(--muted); padding: 24px; text-align: center; border: 1px dashed var(--line); border-radius: 8px; }
.field-row { display: grid; grid-template-columns: 140px 1fr auto; gap: 12px; align-items: center; }
.field-row .field-label { color: var(--muted); font-weight: 700; }
@media (prefers-color-scheme: light) {
  :root { --bg: #f7f8fa; --surface: #ffffff; --surface-2: #eef1f4; --text: #171a1f; --muted: #526170; --line: #d9e0e7; --accent: #067a63; --accent-ink: #ffffff; }
  .sidebar { background: #ffffff; }
}
@media (max-width: 780px) {
  .app-shell { grid-template-columns: 1fr; }
  .sidebar { position: static; height: auto; border-right: 0; border-bottom: 1px solid var(--line); }
  .nav { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .workspace { padding: 18px; }
  .two-col, .toggle-grid { grid-template-columns: 1fr; }
  .split, .backup-row, .field-row { align-items: stretch; flex-direction: column; }
}
"#;

/// Database URL to use for the local vault store. Set via the
/// `PORKPIE_DATABASE_URL` environment variable. The web shell does
/// not have access to SQLite, so this helper only exists on the
/// desktop / native build.
#[cfg(not(target_arch = "wasm32"))]
fn database_url_from_env() -> Option<String> {
    std::env::var("PORKPIE_DATABASE_URL").ok()
}

/// Root Dioxus component shared by desktop and web shells.
pub fn App(cx: Scope) -> Element {
    let backend = use_ref(cx, || VaultBackend::Unavailable);
    let state = use_ref(cx, AppState::default);

    // The initial mount fires a future that loads vault summaries. On
    // non-WASM this is real. On WASM it is a no-op.
    let backend_for_init = backend.clone();
    let state_for_init = state.clone();
    use_future(cx, (), |_| {
        let backend = backend_for_init.clone();
        let state = state_for_init.clone();
        async move { initial_load(backend, state).await }
    });

    let state_for_render = state.clone();
    let backend_for_render = backend.clone();

    cx.render(rsx! {
        style { "{APP_CSS}" }
        div { class: "app-shell",
            aside { class: "sidebar",
                div { class: "brand", "Porkpie" }
                nav { class: "nav", "aria-label": "Primary",
                    a { href: "#onboarding", "Onboarding" }
                    a { href: "#unlock", "Unlock" }
                    a { href: "#items", "Items" }
                    a { href: "#generator", "Generator" }
                    a { href: "#backup", "Import/export" }
                    a { href: "#settings", "Settings" }
                }
            }
            main { class: "workspace",
                OnboardingPage { state: state_for_render.clone(), backend: backend_for_render.clone() }
                UnlockPage { state: state_for_render.clone(), backend: backend_for_render.clone() }
                ItemListPage { state: state_for_render.clone() }
                ItemDetailPage { state: state_for_render.clone() }
                PasswordGeneratorPage { state: state_for_render.clone() }
                ImportExportPage { state: state_for_render.clone(), backend: backend_for_render.clone() }
                SettingsPage { state: state_for_render.clone() }
            }
        }
    })
}

async fn initial_load(backend_ref: UseRef<VaultBackend>, state_ref: UseRef<AppState>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let url = database_url_from_env().unwrap_or_else(|| "sqlite:porkpie.db".to_string());
        let backend = match VaultBackend::connect_sqlite(&url).await {
            Ok(backend) => backend,
            Err(error) => {
                state_ref.with_mut(|state| {
                    state.error = Some(format!("Database error: {error}"));
                    state.screen = Screen::Onboarding;
                });
                return;
            }
        };
        backend_ref.set(backend.clone());
        match backend.list_vault_summaries().await {
            Ok(summaries) => {
                state_ref.with_mut(|state| {
                    state.vaults = summaries;
                    state.screen = if state.vaults.is_empty() {
                        Screen::Onboarding
                    } else {
                        Screen::Unlock
                    };
                });
            }
            Err(error) => {
                state_ref.with_mut(|state| {
                    state.error = Some(format!("Failed to list vaults: {error}"));
                    state.screen = Screen::Onboarding;
                });
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = backend_ref;
        let _ = state_ref;
    }
}
