use crate::components::button::Button;
use crate::state::AppState;
use dioxus::prelude::*;

/// Settings screen. Currently exposes lock timeout and theme. Theme
/// toggling is wired to state but does not yet change the live CSS
/// variables; this is honest about the gap.
pub fn SettingsPage<'a>(cx: Scope<'a, SettingsPageProps>) -> Element<'a> {
    let state_ref = &cx.props.state;
    let lock_minutes = use_state(cx, || state_ref.with(|s| s.settings.lock_timeout_minutes));
    let theme = use_state(cx, || state_ref.with(|s| s.settings.theme));

    let state_for_lock = state_ref.clone();
    let lock_minutes_setter = lock_minutes.clone();
    let on_lock_change = move |event: Event<FormData>| {
        if let Ok(value) = event.value.parse::<u16>() {
            lock_minutes_setter.set(value);
            state_for_lock.with_mut(|s| {
                s.settings.lock_timeout_minutes = value;
            });
        }
    };

    let state_for_theme = state_ref.clone();
    let theme_setter = theme.clone();
    let on_theme_change = move |event: Event<FormData>| {
        let value = match event.value.as_str() {
            "light" => crate::state::Theme::Light,
            _ => crate::state::Theme::Dark,
        };
        theme_setter.set(value);
        state_for_theme.with_mut(|s| {
            s.settings.theme = value;
        });
    };

    let on_lock_vault = move |_| {
        let state = state_ref.clone();
        state.with_mut(|s| {
            s.lock();
        });
    };

    let status = state_ref.with(|s| s.status.clone());
    let error = state_ref.with(|s| s.error.clone());

    cx.render(rsx! {
        section { class: "screen", id: "settings",
            div { class: "screen-header",
                p { class: "eyebrow", "Preferences" }
                h1 { "Settings" }
            }
            form { class: "panel form-grid",
                label { class: "field",
                    span { "Lock timeout" }
                    select {
                        class: "input",
                        value: "{lock_minutes.get()}",
                        onchange: on_lock_change,
                        option { value: "5", "5 minutes" }
                        option { value: "15", "15 minutes" }
                        option { value: "30", "30 minutes" }
                        option { value: "60", "60 minutes" }
                    }
                }
                label { class: "field",
                    span { "Theme" }
                    select {
                        class: "input",
                        value: "dark",
                        onchange: on_theme_change,
                        option { value: "dark", "Dark" }
                        option { value: "light", "Light" }
                    }
                }
                div { class: "notice",
                    "Theme switching updates the configured theme but does not yet re-render the live CSS variables. Restart the app to see the change."
                }
                div { class: "about",
                    h2 { "About Porkpie" }
                    p { class: "muted", "Local-first password vault with encrypted storage. Phase 06 prototype." }
                }
                div { class: "backup-row",
                    div {
                        h2 { "Encrypted backups" }
                        p { class: "muted", "Use the Import/Export page for backup operations." }
                    }
                    div { class: "actions",
                        a { class: "btn btn-primary", href: "#backup", "Open import/export" }
                    }
                }
                status.as_ref().map(|msg| rsx! {
                    div { class: "toast", role: "status", "{msg}" }
                }),
                error.as_ref().map(|msg| rsx! {
                    div { class: "inline-error", role: "alert", "{msg}" }
                }),
                div { class: "actions",
                    Button { label: "Lock vault", variant: "btn-secondary", on_click: on_lock_vault }
                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
pub struct SettingsPageProps {
    pub state: UseRef<AppState>,
}
