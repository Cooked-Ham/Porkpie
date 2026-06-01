use crate::components::{button::Button, item_list::ItemList, text_input::TextInput};
use crate::state::{AppState, Screen};
use crate::vault_store::ItemSummary;
use dioxus::prelude::*;

/// Searchable vault item list backed by real decrypted data from the
/// currently unlocked vault. The list only ever shows metadata, never
/// decrypted field values.
pub fn ItemListPage<'a>(cx: Scope<'a, ItemListPageProps>) -> Element<'a> {
    let state_ref = &cx.props.state;

    let search_value = state_ref.with(|s| s.search_query.clone());
    let items = state_ref.with(|s| s.filtered_items());
    let vault_name = state_ref.with(|s| s.current_vault.as_ref().map(|v| v.name.clone()));
    let is_unlocked = state_ref.with(|s| s.current_vault.is_some());
    let status = state_ref.with(|s| s.status.clone());
    let error = state_ref.with(|s| s.error.clone());

    let on_select = move |item: ItemSummary| {
        let state = state_ref.clone();
        state.with_mut(|s| {
            s.current_item = None;
            s.screen = Screen::Detail(item.id);
        });
    };

    let on_lock = move |_| {
        let state = state_ref.clone();
        state.with_mut(|s| {
            s.lock();
        });
    };

    let on_add = move |_| {
        let state = state_ref.clone();
        state.with_mut(|s| {
            s.current_item = None;
            s.screen = Screen::NewItem;
        });
    };

    let on_refresh = move |_| {
        let state = state_ref.clone();
        cx.spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let handle_opt = state.with(|s| s.unlocked_handle.as_ref().cloned());
                let Some(handle) = handle_opt else {
                    state.with_mut(|s| {
                        s.error = Some("Unlock a vault before refreshing".to_string());
                    });
                    return;
                };
                match handle.list_items().await {
                    Ok(items) => {
                        state.with_mut(|s| {
                            s.items = items;
                            s.error = None;
                            s.status = Some("Items refreshed".to_string());
                        });
                    }
                    Err(error) => {
                        state.with_mut(|s| {
                            s.error = Some(format!("Refresh failed: {error}"));
                        });
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = state;
            }
        });
    };

    cx.render(rsx! {
        section { class: "screen", id: "items",
            div { class: "screen-header split",
                div {
                    p { class: "eyebrow", "Unlocked" }
                    h1 { "Items" }
                    p { class: "muted",
                        if let Some(name) = vault_name.clone() {
                            format!("Vault: {name}")
                        } else {
                            "No vault unlocked".to_string()
                        }
                    }
                }
                div { class: "actions",
                    button {
                        class: "icon-btn",
                        r#type: "button",
                        title: "Add item",
                        onclick: on_add,
                        "+"
                    }
                    Button { label: "Refresh", variant: "btn-secondary", on_click: on_refresh }
                    Button { label: "Lock", variant: "btn-secondary", on_click: on_lock }
                }
            }
            div { class: "panel list-panel",
                TextInput {
                    label: "Search",
                    value: "{search_value}",
                    placeholder: "Filter by title or type",
                    on_input: move |value: String| {
                        let state = state_ref.clone();
                        state.with_mut(|s| s.search_query = value);
                    }
                }
                if is_unlocked {
                    if items.is_empty() {
                        rsx! {
                            div { class: "empty-state",
                                "No items yet. Use the + button to add your first item."
                            }
                        }
                    } else {
                        rsx! {
                            ItemList { items: items, on_select: on_select }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "empty-state",
                            "Unlock a vault to see its items."
                        }
                    }
                }
                status.as_ref().map(|msg| rsx! {
                    div { class: "toast", role: "status", "{msg}" }
                }),
                error.as_ref().map(|msg| rsx! {
                    div { class: "inline-error", role: "alert", "{msg}" }
                })
            }
        }
    })
}

#[derive(Props, PartialEq)]
pub struct ItemListPageProps {
    pub state: UseRef<AppState>,
}
