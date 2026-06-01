/// Copy text to the clipboard.
#[cfg(not(target_arch = "wasm32"))]
pub fn copy_to_clipboard(value: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
    clipboard
        .set_text(value.to_string())
        .map_err(|error| error.to_string())
}

/// Copy text to the clipboard.
#[cfg(target_arch = "wasm32")]
pub fn copy_to_clipboard(_value: &str) -> Result<(), String> {
    Err("Use the web shell clipboard bridge for browser clipboard writes".to_string())
}
