//! Desktop-only settings persistence.
//!
//! Reads and writes `settings.json` to the platform data directory so
//! preferences survive app restarts. On WASM this module is empty; the
//! web shell stores preferences in `localStorage` separately if needed.

use crate::state::SettingsState;
use std::path::{Path, PathBuf};

/// Error type for settings I/O failures.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Resolve the platform-specific data directory for Porkpie.
fn platform_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return Some(PathBuf::from(appdata).join("Porkpie"));
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                return Some(
                    PathBuf::from(home)
                        .join("Library")
                        .join("Application Support")
                        .join("Porkpie"),
                );
            }
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            if !xdg.trim().is_empty() {
                return Some(PathBuf::from(xdg).join("porkpie"));
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                return Some(
                    PathBuf::from(home)
                        .join(".local")
                        .join("share")
                        .join("porkpie"),
                );
            }
        }
    }
    None
}

/// Default path to the settings JSON file.
pub fn default_settings_path() -> PathBuf {
    platform_data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("settings.json")
}

/// Load settings from the given path. If the file does not exist or is
/// malformed, return `SettingsState::default()`.
pub fn load_or_default(path: &Path) -> SettingsState {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => SettingsState::default(),
    }
}

/// Save settings to the given path, creating parent directories if needed.
pub fn save_settings(path: &Path, settings: &SettingsState) -> Result<(), SettingsError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn roundtrip_settings() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let settings = SettingsState {
            lock_timeout_minutes: 15,
            theme: crate::state::Theme::Light,
            stay_signed_in: true,
            minimize_to_tray: true,
            close_to_tray: false,
        };
        save_settings(tmp.path(), &settings).unwrap();
        let loaded = load_or_default(tmp.path());
        assert_eq!(loaded, settings);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let path = std::env::temp_dir().join("porkpie-settings-missing.json");
        let _ = std::fs::remove_file(&path);
        let loaded = load_or_default(&path);
        assert_eq!(loaded, SettingsState::default());
    }

    #[test]
    fn malformed_file_returns_defaults() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "not json").unwrap();
        let loaded = load_or_default(tmp.path());
        assert_eq!(loaded, SettingsState::default());
    }
}
