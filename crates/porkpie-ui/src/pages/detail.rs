use crate::components::{
    button::Button, modal::Modal, password_input::PasswordInput, text_input::TextInput,
};
use dioxus::prelude::*;

/// Item detail and edit surface.
pub fn ItemDetailPage(cx: Scope) -> Element {
    cx.render(rsx! {
        section { class: "screen", id: "detail",
            div { class: "screen-header split",
                div {
                    p { class: "eyebrow", "Login" }
                    h1 { "Item detail" }
                }
                a { class: "btn btn-secondary", href: "#items", "Back" }
            }
            form { class: "panel form-grid two-col",
                TextInput { label: "Title", value: "Example login", placeholder: "Item title" }
                TextInput { label: "Username", value: "user@example.com", placeholder: "Username" }
                PasswordInput { label: "Password", value: "hidden" }
                TextInput { label: "URL", value: "https://example.com", placeholder: "Website" }
                label { class: "field full",
                    span { "Notes" }
                    textarea { class: "input textarea", "Recovery notes and metadata" }
                }
                div { class: "actions full",
                    Button { label: "Edit", variant: "btn-secondary" }
                    Button { label: "Save", variant: "btn-primary" }
                    Button { label: "Cancel", variant: "btn-secondary" }
                    Button { label: "Copy secret", variant: "btn-secondary" }
                    Button { label: "Delete", variant: "btn-danger" }
                }
            }
            Modal { title: "Delete item", message: "Confirm before permanently removing this item." }
        }
    })
}
