use dioxus::prelude::*;

/// Reusable password input with optional on_input wiring.
#[derive(Props)]
pub struct PasswordInputProps<'a> {
    pub label: &'a str,
    pub value: &'a str,
    #[props(optional)]
    pub on_input: Option<EventHandler<'a, String>>,
    #[props(optional)]
    pub auto_complete: Option<&'a str>,
}

pub fn PasswordInput<'a>(cx: Scope<'a, PasswordInputProps<'a>>) -> Element<'a> {
    let value = cx.props.value.to_string();
    let auto_complete = cx
        .props
        .auto_complete
        .unwrap_or("current-password")
        .to_string();
    let on_input = &cx.props.on_input;

    cx.render(rsx! {
        label { class: "field",
            span { "{cx.props.label}" }
            input {
                class: "input",
                r#type: "password",
                value: "{value}",
                autocomplete: "{auto_complete}",
                oninput: move |event| {
                    if let Some(handler) = on_input {
                        handler.call(event.value.clone());
                    }
                }
            }
        }
    })
}
