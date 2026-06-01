use dioxus::prelude::*;

/// Backup import/export screen.
pub fn ImportExportPage(cx: Scope) -> Element {
    let status = use_state(cx, || "Ready".to_string());
    let import_status = status.clone();
    let export_status = status.clone();

    cx.render(rsx! {
        section { class: "screen", id: "backup",
            div { class: "screen-header",
                p { class: "eyebrow", "Backup" }
                h1 { "Import and export" }
            }
            div { class: "panel form-grid",
                div { class: "backup-row",
                    div {
                        h2 { "Export encrypted backup" }
                        p { class: "muted", "Save an encrypted copy of this vault." }
                    }
                    button {
                        class: "btn btn-primary",
                        r#type: "button",
                        onclick: move |_| export_status.set("Preparing encrypted backup export.".to_string()),
                        "Export"
                    }
                }
                div { class: "backup-row",
                    div {
                        h2 { "Import backup" }
                        p { class: "muted", "Load a Porkpie backup file." }
                    }
                    label { class: "btn btn-secondary file-btn",
                        input {
                            r#type: "file",
                            accept: ".json.enc,.csv",
                            hidden: "true",
                            onchange: move |_| import_status.set("Import file selected; validating encrypted data.".to_string())
                        }
                        "Choose file"
                    }
                }
                div { class: "toast", role: "status", "{status}" }
            }
        }
    })
}
