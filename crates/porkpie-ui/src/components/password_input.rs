use dioxus::prelude::*;

/// Password input component.
#[derive(Props, PartialEq)]
pub struct PasswordInputProps<'a> {
    pub label: &'a str,
    pub value: &'a str,
}

pub fn PasswordInput<'a>(cx: Scope<'a, PasswordInputProps<'a>>) -> Element<'a> {
    cx.render(rsx! {
        label { class: "field",
            span { "{cx.props.label}" }
            input {
                class: "input",
                r#type: "password",
                value: "{cx.props.value}",
                autocomplete: "current-password"
            }
        }
    })
}
