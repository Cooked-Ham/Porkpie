use dioxus::prelude::*;

/// Reusable button component.
#[derive(Props, PartialEq)]
pub struct ButtonProps<'a> {
    pub label: &'a str,
    #[props(optional)]
    pub variant: Option<&'a str>,
}

pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element<'a> {
    let class_name = format!("btn {}", cx.props.variant.unwrap_or("btn-primary"));
    cx.render(rsx! {
        button { class: "{class_name}", r#type: "button", "{cx.props.label}" }
    })
}
