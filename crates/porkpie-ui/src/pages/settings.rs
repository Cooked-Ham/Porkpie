use crate::components::button::Button;
use dioxus::prelude::*;

/// User settings screen.
pub fn SettingsPage(cx: Scope) -> Element {
    cx.render(rsx! {
        section { class: "screen", id: "settings",
            div { class: "screen-header",
                p { class: "eyebrow", "Preferences" }
                h1 { "Settings" }
            }
            form { class: "panel form-grid",
                label { class: "field",
                    span { "Lock timeout" }
                    select { class: "input",
                        option { value: "5", "5 minutes" }
                        option { value: "15", "15 minutes" }
                        option { value: "30", selected: "true", "30 minutes" }
                    }
                }
                label { class: "field",
                    span { "Theme" }
                    select { class: "input",
                        option { value: "dark", selected: "true", "Dark" }
                        option { value: "light", "Light" }
                    }
                }
                div { class: "about",
                    h2 { "About Porkpie" }
                    p { class: "muted", "Local-first password vault with encrypted storage." }
                }
                div { class: "backup-row",
                    div {
                        h2 { "Encrypted backups" }
                        p { class: "muted", "Export or import vault backups from settings." }
                    }
                    div { class: "actions",
                        Button { label: "Export", variant: "btn-primary" }
                        label { class: "btn btn-secondary file-btn",
                            input { r#type: "file", accept: ".json.enc,.csv", hidden: "true" }
                            "Import"
                        }
                    }
                }
                div { class: "toast", role: "status", "Backup progress and results appear here." }
                div { class: "actions",
                    Button { label: "Lock vault", variant: "btn-secondary" }
                }
            }
        }
    })
}
