//! System tray integration for the desktop app.
//!
//! Creates a tray icon with a context menu (Open, Lock, Quit) before the
//! Dioxus event loop starts and forwards menu events to the UI through a
//! global command queue polled by a timer in the root component.

use porkpie_ui::tray::{TrayCommand, TRAY_COMMANDS};

/// Build the tray icon and spawn a background thread that polls menu events.
///
/// Must be called on the main thread before `dioxus_desktop::launch_cfg` because
/// `TrayIcon` is not `Send` and the tao event loop runs on the same thread.
pub fn create_tray_icon() -> Result<tray_icon::TrayIcon, Box<dyn std::error::Error>> {
    let menu = tray_icon::menu::Menu::new();
    let open_item = tray_icon::menu::MenuItem::new("Open", true, None);
    let lock_item = tray_icon::menu::MenuItem::new("Lock", true, None);
    let quit_item = tray_icon::menu::MenuItem::new("Quit", true, None);

    menu.append(&open_item)?;
    menu.append(&lock_item)?;
    menu.append(&tray_icon::menu::PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let icon = build_icon()?;

    let tray = tray_icon::TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Porkpie")
        .with_icon(icon)
        .build()?;

    let commands = TRAY_COMMANDS.clone();
    let open_id = open_item.id().clone();
    let lock_id = lock_item.id().clone();
    let quit_id = quit_item.id().clone();

    std::thread::spawn(move || {
        let receiver = tray_icon::menu::MenuEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                let cmd = if event.id == open_id {
                    Some(TrayCommand::ShowWindow)
                } else if event.id == lock_id {
                    Some(TrayCommand::LockVault)
                } else if event.id == quit_id {
                    Some(TrayCommand::Quit)
                } else {
                    None
                };

                if let Some(cmd) = cmd {
                    let mut queue = commands.lock().unwrap();
                    queue.push(cmd);
                }

                if event.id == quit_id {
                    break;
                }
            }
        }
    });

    Ok(tray)
}

/// Build a simple 32×32 RGBA icon (solid accent colour matching the UI).
fn build_icon() -> Result<tray_icon::Icon, tray_icon::BadIcon> {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            rgba[i] = 0x4c; // R
            rgba[i + 1] = 0xc9; // G
            rgba[i + 2] = 0xa6; // B
            rgba[i + 3] = 0xff; // A
        }
    }
    tray_icon::Icon::from_rgba(rgba, width, height)
}
