use dioxus::prelude::*;

/// Text input component.
#[derive(Props, PartialEq)]
pub struct TextInputProps<'a> {
    pub label: &'a str,
    pub value: &'a str,
    #[props(optional)]
    pub placeholder: Option<&'a str>,
}

pub fn TextInput<'a>(cx: Scope<'a, TextInputProps<'a>>) -> Element<'a> {
    let placeholder = match cx.props.placeholder {
        Some(placeholder) => placeholder,
        None => "",
    };
    cx.render(rsx! {
        label { class: "field",
            span { "{cx.props.label}" }
            input {
                class: "input",
                r#type: "text",
                value: "{cx.props.value}",
                placeholder: "{placeholder}",
                autocomplete: "off"
            }
        }
    })
}
