use dioxus::prelude::*;

/// Modal shell used for confirmations. The caller decides whether to render
/// it at all; this component only provides the visual structure.
#[derive(Props)]
pub struct ModalProps<'a> {
    pub title: &'a str,
    pub message: &'a str,
    #[props(optional)]
    pub confirm_label: Option<&'a str>,
    #[props(optional)]
    pub cancel_label: Option<&'a str>,
    #[props(optional)]
    pub on_confirm: Option<EventHandler<'a, MouseEvent>>,
    #[props(optional)]
    pub on_cancel: Option<EventHandler<'a, MouseEvent>>,
    #[props(optional)]
    pub danger: Option<bool>,
}

pub fn Modal<'a>(cx: Scope<'a, ModalProps<'a>>) -> Element<'a> {
    let confirm_label = cx.props.confirm_label.unwrap_or("Confirm").to_string();
    let cancel_label = cx.props.cancel_label.unwrap_or("Cancel").to_string();
    let on_confirm = &cx.props.on_confirm;
    let on_cancel = &cx.props.on_cancel;
    let danger = cx.props.danger.unwrap_or(false);
    let confirm_class = if danger {
        "btn btn-danger"
    } else {
        "btn btn-primary"
    };
    let title = cx.props.title.to_string();
    let message = cx.props.message.to_string();

    cx.render(rsx! {
        div { class: "modal-backdrop",
            div { class: "modal",
                h2 { "{title}" }
                p { class: "muted", "{message}" }
                div { class: "actions",
                    button {
                        class: "btn btn-secondary",
                        r#type: "button",
                        onclick: move |event| {
                            if let Some(handler) = on_cancel {
                                handler.call(event);
                            }
                        },
                        "{cancel_label}"
                    }
                    button {
                        class: "{confirm_class}",
                        r#type: "button",
                        onclick: move |event| {
                            if let Some(handler) = on_confirm {
                                handler.call(event);
                            }
                        },
                        "{confirm_label}"
                    }
                }
            }
        }
    })
}
