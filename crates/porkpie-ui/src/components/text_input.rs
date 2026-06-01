use dioxus::prelude::*;

/// Reusable text input component with optional on_input wiring.
#[derive(Props)]
pub struct TextInputProps<'a> {
    pub label: &'a str,
    pub value: &'a str,
    #[props(optional)]
    pub placeholder: Option<&'a str>,
    #[props(optional)]
    pub on_input: Option<EventHandler<'a, String>>,
    #[props(optional)]
    pub input_type: Option<&'a str>,
    #[props(optional)]
    pub auto_complete: Option<&'a str>,
}

pub fn TextInput<'a>(cx: Scope<'a, TextInputProps<'a>>) -> Element<'a> {
    let placeholder = cx.props.placeholder.unwrap_or_default().to_string();
    let input_type = cx.props.input_type.unwrap_or("text").to_string();
    let auto_complete = cx.props.auto_complete.unwrap_or("off").to_string();
    let value = cx.props.value.to_string();
    let on_input = &cx.props.on_input;

    cx.render(rsx! {
        label { class: "field",
            span { "{cx.props.label}" }
            input {
                class: "input",
                r#type: "{input_type}",
                value: "{value}",
                placeholder: "{placeholder}",
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
