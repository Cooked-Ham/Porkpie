use dioxus::prelude::*;

/// Simple form section wrapper.
#[derive(Props, PartialEq)]
pub struct FormSectionProps<'a> {
    pub title: &'a str,
}

pub fn FormSection<'a>(cx: Scope<'a, FormSectionProps<'a>>) -> Element<'a> {
    cx.render(rsx! {
        section { class: "form-section",
            h2 { "{cx.props.title}" }
        }
    })
}
