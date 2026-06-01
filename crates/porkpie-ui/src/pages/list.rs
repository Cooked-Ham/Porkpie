use crate::components::{button::Button, item_list::ItemList, text_input::TextInput};
use crate::state::ItemSummary;
use dioxus::prelude::*;

/// Searchable vault item list.
#[derive(Props, PartialEq)]
pub struct ItemListPageProps {
    pub items: Vec<ItemSummary>,
}

pub fn ItemListPage(cx: Scope<ItemListPageProps>) -> Element {
    cx.render(rsx! {
        section { class: "screen", id: "items",
            div { class: "screen-header split",
                div {
                    p { class: "eyebrow", "Unlocked" }
                    h1 { "Items" }
                }
                div { class: "actions",
                    a { class: "icon-btn", href: "#detail", title: "Add item", "+" }
                    Button { label: "Lock", variant: "btn-secondary" }
                }
            }
            div { class: "panel list-panel",
                TextInput { label: "Search", value: "", placeholder: "Filter by title or type" }
                ItemList { items: cx.props.items.clone() }
            }
        }
    })
}
