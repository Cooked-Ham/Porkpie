use crate::state::ItemSummary;
use dioxus::prelude::*;

/// Render a table of item summaries.
#[derive(Props, PartialEq)]
pub struct ItemListProps {
    pub items: Vec<ItemSummary>,
}

pub fn ItemList(cx: Scope<ItemListProps>) -> Element {
    cx.render(rsx! {
        table { class: "item-table",
            thead {
                tr {
                    th { "Type" }
                    th { "Title" }
                    th { "Created" }
                    th { "Updated" }
                }
            }
            tbody {
                cx.props.items.iter().map(|item| rsx! {
                    tr { key: "{item.id}",
                        td { "{item.item_type}" }
                        td { "{item.title}" }
                        td { "{item.created_at.to_millis()}" }
                        td { "{item.updated_at.to_millis()}" }
                    }
                })
            }
        }
    })
}
