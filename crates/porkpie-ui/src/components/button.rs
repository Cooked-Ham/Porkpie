use dioxus::prelude::*;

/// Reusable button component.
#[derive(Props)]
pub struct ButtonProps<'a> {
    pub label: &'a str,
    #[props(optional)]
    pub variant: Option<&'a str>,
    #[props(optional)]
    pub on_click: Option<EventHandler<'a, MouseEvent>>,
    #[props(optional)]
    pub disabled: Option<bool>,
}

pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element<'a> {
    let class_name = format!("btn {}", cx.props.variant.unwrap_or("btn-primary"));
    let on_click = &cx.props.on_click;
    let disabled = cx.props.disabled.unwrap_or(false);
    let label = cx.props.label.to_string();

    cx.render(rsx! {
        button {
            class: "{class_name}",
            r#type: "button",
            disabled: "{disabled}",
            onclick: move |event| {
                if let Some(handler) = on_click {
                    handler.call(event);
                }
            },
            "{label}"
        }
    })
}
