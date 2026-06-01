use dioxus::prelude::*;

/// Modal shell used for confirmations.
#[derive(Props, PartialEq)]
pub struct ModalProps<'a> {
    pub title: &'a str,
    pub message: &'a str,
}

pub fn Modal<'a>(cx: Scope<'a, ModalProps<'a>>) -> Element<'a> {
    cx.render(rsx! {
        div { class: "modal-backdrop",
            div { class: "modal",
                h2 { "{cx.props.title}" }
                p { "{cx.props.message}" }
            }
        }
    })
}
