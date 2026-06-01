use crate::state::ItemSummary;
use dioxus::prelude::*;

/// Render a table of item summaries.
#[derive(Props)]
pub struct ItemListProps<'a> {
    pub items: Vec<ItemSummary>,
    #[props(optional)]
    pub on_select: Option<EventHandler<'a, ItemSummary>>,
}

pub fn ItemList<'a>(cx: Scope<'a, ItemListProps<'a>>) -> Element<'a> {
    let on_select = &cx.props.on_select;

    cx.render(rsx! {
        table { class: "item-table",
            thead {
                tr {
                    th { "Type" }
                    th { "Title" }
                    th { "Created" }
                    th { "Updated" }
                    th { "" }
                }
            }
            tbody {
                cx.props.items.iter().map(|item| {
                    let item = item.clone();
                    let item_for_handler = item.clone();
                    let handler = on_select;
                    rsx! {
                        tr { key: "{item.id}",
                            td { "{item.item_type}" }
                            td { "{item.title}" }
                            td { "{item.created_at.to_millis()}" }
                            td { "{item.updated_at.to_millis()}" }
                            td {
                                button {
                                    class: "btn btn-secondary",
                                    r#type: "button",
                                    onclick: move |_| {
                                        if let Some(handler) = handler {
                                            handler.call(item_for_handler.clone());
                                        }
                                    },
                                    "Open"
                                }
                            }
                        }
                    }
                })
            }
        }
    })
}
