use crate::components::{button::Button, password_input::PasswordInput, text_input::TextInput};
use dioxus::prelude::*;

/// Unlock screen for an existing vault.
pub fn UnlockPage(cx: Scope) -> Element {
    cx.render(rsx! {
        section { class: "screen", id: "unlock",
            div { class: "screen-header",
                p { class: "eyebrow", "Locked" }
                h1 { "Unlock vault" }
            }
            form { class: "panel form-grid",
                TextInput { label: "Vault ID", value: "", placeholder: "Known vault ID" }
                PasswordInput { label: "Master password", value: "" }
                div { class: "inline-error", role: "alert", "Wrong password or vault not found." }
                div { class: "actions",
                    Button { label: "Unlock", variant: "btn-primary" }
                    a { class: "btn btn-secondary", href: "#onboarding", "Create new" }
                }
            }
        }
    })
}
