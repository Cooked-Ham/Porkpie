use crate::components::{button::Button, form::FormSection};
use crate::state::{AppState, PasswordGeneratorState};
use crate::utils::validation::{has_password_character_set, validate_password_length};
use dioxus::prelude::*;
use porkpie_core::{generate_password, PasswordOptions};

/// Password generator controls wired to a real Porkpie password
/// generator. The generated password is held in state and never written
/// to disk or persisted across reloads.
pub fn PasswordGeneratorPage<'a>(cx: Scope<'a, PasswordGeneratorPageProps>) -> Element<'a> {
    let state = cx.props.state.clone();

    let length = use_state(cx, || state.with(|s| s.password_generator.length));
    let uppercase = use_state(cx, || state.with(|s| s.password_generator.uppercase));
    let lowercase = use_state(cx, || state.with(|s| s.password_generator.lowercase));
    let numbers = use_state(cx, || state.with(|s| s.password_generator.numbers));
    let symbols = use_state(cx, || state.with(|s| s.password_generator.symbols));
    let exclude_ambiguous = use_state(cx, || {
        state.with(|s| s.password_generator.exclude_ambiguous)
    });
    let error = use_state(cx, || None::<String>);

    let state_for_generate = state.clone();
    let length_for_click = *length.get();
    let length_for_click_handle = length.clone();
    let options_for_click = PasswordOptions {
        uppercase: *uppercase.get(),
        lowercase: *lowercase.get(),
        numbers: *numbers.get(),
        symbols: *symbols.get(),
        exclude_ambiguous: *exclude_ambiguous.get(),
        custom_chars: None,
    };
    let error_for_click = error.clone();

    let generate = move |_| {
        if let Err(validation_error) = validate_password_length(length_for_click) {
            error_for_click.set(Some(validation_error));
            return;
        }
        if !has_password_character_set(
            options_for_click.uppercase,
            options_for_click.lowercase,
            options_for_click.numbers,
            options_for_click.symbols,
        ) {
            error_for_click.set(Some("Enable at least one character set".to_string()));
            return;
        }
        match generate_password(length_for_click, &options_for_click) {
            Ok(password) => {
                state_for_generate.with_mut(|s| {
                    s.password_generator.length = length_for_click;
                    s.password_generator.uppercase = options_for_click.uppercase;
                    s.password_generator.lowercase = options_for_click.lowercase;
                    s.password_generator.numbers = options_for_click.numbers;
                    s.password_generator.symbols = options_for_click.symbols;
                    s.password_generator.exclude_ambiguous = options_for_click.exclude_ambiguous;
                    s.password_generator.generated_password = password.clone();
                    s.status = Some("Password generated".to_string());
                });
                error_for_click.set(None);
            }
            Err(error) => {
                error_for_click.set(Some(format!("Generator error: {error}")));
            }
        }
    };

    let state_for_copy = state.clone();
    let error_for_copy = error.clone();
    let copy = move |_| {
        let password = state_for_copy.with(|s| s.password_generator.generated_password.clone());
        if password.is_empty() {
            error_for_copy.set(Some("Generate a password first".to_string()));
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Err(error) = crate::utils::clipboard::copy_to_clipboard(&password) {
                error_for_copy.set(Some(format!("Clipboard error: {error}")));
            } else {
                state_for_copy.with_mut(|s| {
                    s.status = Some("Password copied to clipboard".to_string());
                });
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = password;
            error_for_copy.set(Some("Clipboard not available in this build".to_string()));
        }
    };

    let current_password = state.with(|s| s.password_generator.generated_password.clone());

    cx.render(rsx! {
        section { class: "screen", id: "generator",
            div { class: "screen-header",
                p { class: "eyebrow", "Tools" }
                h1 { "Password generator" }
                p { class: "muted", "Generates passwords from your local Porkpie random number generator. Nothing is uploaded." }
            }
            div { class: "panel form-grid",
                FormSection { title: "Length" }
                div { class: "field-row",
                    span { class: "field-label", "Characters" }
                    input {
                        class: "range",
                        r#type: "range",
                        min: "8",
                        max: "128",
                        value: "{*length.get()}",
                        oninput: move |event| {
                            if let Ok(value) = event.value.parse::<usize>() {
                                length_for_click_handle.set(value);
                            }
                        }
                    }
                    span { "{*length.get()}" }
                }
                div { class: "toggle-grid",
                    label { input {
                        r#type: "checkbox",
                        checked: "{*uppercase.get()}",
                        onchange: move |event| uppercase.set(event.value == "true")
                    } "Uppercase" }
                    label { input {
                        r#type: "checkbox",
                        checked: "{*lowercase.get()}",
                        onchange: move |event| lowercase.set(event.value == "true")
                    } "Lowercase" }
                    label { input {
                        r#type: "checkbox",
                        checked: "{*numbers.get()}",
                        onchange: move |event| numbers.set(event.value == "true")
                    } "Numbers" }
                    label { input {
                        r#type: "checkbox",
                        checked: "{*symbols.get()}",
                        onchange: move |event| symbols.set(event.value == "true")
                    } "Symbols" }
                }
                label { class: "field",
                    span { "Exclude ambiguous (0, O, 1, l, I, |)" }
                    input {
                        r#type: "checkbox",
                        checked: "{*exclude_ambiguous.get()}",
                        onchange: move |event| exclude_ambiguous.set(event.value == "true")
                    }
                }
                output { class: "generated", "{current_password}" }
                error.get().as_ref().map(|err| rsx! {
                    div { class: "inline-error", role: "alert", "{err}" }
                }),
                div { class: "actions",
                    Button { label: "Generate", variant: "btn-primary", on_click: generate }
                    Button { label: "Copy", variant: "btn-secondary", on_click: copy }
                }
                div { class: "notice",
                    "Clipboard writes go through the desktop arboard bridge. On the web shell, the system clipboard may be unavailable; use the form manually in that case."
                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
pub struct PasswordGeneratorPageProps {
    pub state: UseRef<AppState>,
}

// Re-export the state struct so unused warnings do not complain about it.
#[allow(dead_code)]
fn _use_state(_: &PasswordGeneratorState) {}
