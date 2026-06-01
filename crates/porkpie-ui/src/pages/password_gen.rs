use crate::components::button::Button;
use dioxus::prelude::*;
use porkpie_core::{generate_password, PasswordOptions};

/// Password generator controls.
pub fn PasswordGeneratorPage(cx: Scope) -> Element {
    let generated = generate_password(24, &PasswordOptions::default())
        .unwrap_or_else(|error| format!("Generator error: {error}"));

    cx.render(rsx! {
        section { class: "screen", id: "generator",
            div { class: "screen-header",
                p { class: "eyebrow", "Tools" }
                h1 { "Password generator" }
            }
            div { class: "panel form-grid",
                label { class: "field",
                    span { "Length" }
                    input { class: "range", r#type: "range", min: "8", max: "128", value: "24" }
                }
                div { class: "toggle-grid",
                    label { input { r#type: "checkbox", checked: "true" } "Uppercase" }
                    label { input { r#type: "checkbox", checked: "true" } "Lowercase" }
                    label { input { r#type: "checkbox", checked: "true" } "Numbers" }
                    label { input { r#type: "checkbox", checked: "true" } "Symbols" }
                }
                output { class: "generated", "{generated}" }
                div { class: "actions",
                    Button { label: "Generate", variant: "btn-primary" }
                    Button { label: "Copy", variant: "btn-secondary" }
                }
            }
        }
    })
}
