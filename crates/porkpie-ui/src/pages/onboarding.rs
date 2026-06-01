use crate::components::{button::Button, password_input::PasswordInput};
use dioxus::prelude::*;

/// Create-vault screen with master password validation messaging.
pub fn OnboardingPage(cx: Scope) -> Element {
    cx.render(rsx! {
        section { class: "screen", id: "onboarding",
            div { class: "screen-header",
                p { class: "eyebrow", "New vault" }
                h1 { "Create your Porkpie vault" }
                p { class: "muted", "Choose a master password with at least 16 characters." }
            }
            form { class: "panel form-grid",
                PasswordInput { label: "Master password", value: "" }
                PasswordInput { label: "Confirm master password", value: "" }
                div { class: "inline-error", role: "alert", "Passwords must be at least 16 characters and match." }
                div { class: "actions",
                    Button { label: "Create vault", variant: "btn-primary" }
                    a { class: "btn btn-secondary", href: "#unlock", "Open existing" }
                }
            }
        }
    })
}
