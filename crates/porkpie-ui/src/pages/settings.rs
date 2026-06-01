use crate::components::button::Button;
use crate::state::{AppState, SettingsState};
use dioxus::prelude::*;

/// Settings screen. Exposes lock timeout, theme, and desktop integration
/// toggles. Changes are saved to disk automatically on desktop.
pub fn SettingsPage<'a>(cx: Scope<'a, SettingsPageProps>) -> Element<'a> {
    let state_ref = &cx.props.state;
    let lock_minutes = use_state(cx, || state_ref.with(|s| s.settings.lock_timeout_minutes));
    let theme = use_state(cx, || state_ref.with(|s| s.settings.theme));
    let stay_signed_in = use_state(cx, || state_ref.with(|s| s.settings.stay_signed_in));
    let minimize_to_tray = use_state(cx, || state_ref.with(|s| s.settings.minimize_to_tray));
    let close_to_tray = use_state(cx, || state_ref.with(|s| s.settings.close_to_tray));

    let state_for_lock = state_ref.clone();
    let lock_minutes_setter = lock_minutes.clone();
    let on_lock_change = move |event: Event<FormData>| {
        if let Ok(value) = event.value.parse::<u16>() {
            lock_minutes_setter.set(value);
            let mut settings = state_for_lock.with(|s| s.settings.clone());
            settings.lock_timeout_minutes = value;
            state_for_lock.with_mut(|s| {
                s.settings.lock_timeout_minutes = value;
            });
            save_settings_async(settings);
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
        let mut settings = state_for_theme.with(|s| s.settings.clone());
        settings.theme = value;
        state_for_theme.with_mut(|s| {
            s.settings.theme = value;
        });
        save_settings_async(settings);
    };

    let state_for_stay = state_ref.clone();
    let stay_setter = stay_signed_in.clone();
    let on_stay_change = move |event: Event<FormData>| {
        let value = event.value == "on";
        stay_setter.set(value);
        let mut settings = state_for_stay.with(|s| s.settings.clone());
        settings.stay_signed_in = value;
        state_for_stay.with_mut(|s| {
            s.settings.stay_signed_in = value;
        });
        save_settings_async(settings);
    };

    let state_for_minimize = state_ref.clone();
    let minimize_setter = minimize_to_tray.clone();
    let on_minimize_change = move |event: Event<FormData>| {
        let value = event.value == "on";
        minimize_setter.set(value);
        let mut settings = state_for_minimize.with(|s| s.settings.clone());
        settings.minimize_to_tray = value;
        state_for_minimize.with_mut(|s| {
            s.settings.minimize_to_tray = value;
        });
        save_settings_async(settings);
    };

    let state_for_close = state_ref.clone();
    let close_setter = close_to_tray.clone();
    let on_close_change = move |event: Event<FormData>| {
        let value = event.value == "on";
        close_setter.set(value);
        let mut settings = state_for_close.with(|s| s.settings.clone());
        settings.close_to_tray = value;
        state_for_close.with_mut(|s| {
            s.settings.close_to_tray = value;
        });
        save_settings_async(settings);
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
                        value: "{theme}",
                        onchange: on_theme_change,
                        option { value: "dark", "Dark" }
                        option { value: "light", "Light" }
                    }
                }
                label { class: "field checkbox-field",
                    input {
                        r#type: "checkbox",
                        checked: "{stay_signed_in}",
                        onchange: on_stay_change,
                    }
                    span { "Stay signed in" }
                }
                label { class: "field checkbox-field",
                    input {
                        r#type: "checkbox",
                        checked: "{minimize_to_tray}",
                        onchange: on_minimize_change,
                    }
                    span { "Minimize to tray" }
                }
                label { class: "field checkbox-field",
                    input {
                        r#type: "checkbox",
                        checked: "{close_to_tray}",
                        onchange: on_close_change,
                    }
                    span { "Close to tray" }
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
                        button {
                            class: "btn btn-primary",
                            r#type: "button",
                            onclick: move |_| {
                                state_ref.with_mut(|s| s.screen = crate::state::Screen::ImportExport);
                            },
                            "Open import/export"
                        }
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

fn save_settings_async(settings: SettingsState) {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let path = crate::settings_store::default_settings_path();
        let _ = crate::settings_store::save_settings(&path, &settings);
    });
}

#[derive(Props, PartialEq)]
pub struct SettingsPageProps {
    pub state: UseRef<AppState>,
}
