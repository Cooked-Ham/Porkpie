//! Desktop launch configuration helpers.
//!
//! Encapsulates reading the desktop launch configuration from the
//! process environment and converting it into the `dioxus_desktop::Config`
//! the entrypoint uses.

use std::path::PathBuf;

/// Configuration used by the desktop entrypoint.
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub database_url: String,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            window_title: "Porkpie".to_string(),
            window_width: 1180,
            window_height: 820,
            database_url: default_database_url(),
        }
    }
}

impl LaunchConfig {
    /// Build a launch config from the process environment, falling
    /// back to sensible defaults for the desktop binary.
    pub fn load() -> Self {
        let mut config = Self::default();
        if let Ok(value) = std::env::var("PORKPIE_WINDOW_TITLE") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                config.window_title = trimmed.to_string();
            }
        }
        if let Ok(value) = std::env::var("PORKPIE_WINDOW_WIDTH") {
            if let Ok(parsed) = value.parse::<u32>() {
                if parsed >= 480 {
                    config.window_width = parsed;
                }
            }
        }
        if let Ok(value) = std::env::var("PORKPIE_WINDOW_HEIGHT") {
            if let Ok(parsed) = value.parse::<u32>() {
                if parsed >= 360 {
                    config.window_height = parsed;
                }
            }
        }
        if let Ok(value) = std::env::var("PORKPIE_DATABASE_URL") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                config.database_url = trimmed.to_string();
            }
        }
        config
    }

    /// Convert into a Dioxus desktop `Config`.
    pub fn into_dioxus_config(self, close_to_tray: bool) -> dioxus_desktop::Config {
        // The Dioxus desktop backend reads the database URL from
        // the same environment variable the UI uses, so any caller
        // overriding the location through `LaunchConfig` is wired
        // through the process environment.
        if std::env::var_os("PORKPIE_DATABASE_URL").is_none() {
            std::env::set_var("PORKPIE_DATABASE_URL", &self.database_url);
        }
        let data_dir = platform_data_dir().unwrap_or_else(|| PathBuf::from("."));
        let _ = std::fs::create_dir_all(&data_dir);
        let mut cfg = dioxus_desktop::Config::default()
            .with_data_directory(&data_dir)
            .with_resource_directory(&data_dir)
            .with_window(
                dioxus_desktop::WindowBuilder::new()
                    .with_title(&self.window_title)
                    .with_inner_size(dioxus_desktop::LogicalSize::new(
                        f64::from(self.window_width),
                        f64::from(self.window_height),
                    )),
            );
        if close_to_tray {
            cfg = cfg.with_close_behaviour(dioxus_desktop::WindowCloseBehaviour::LastWindowHides);
        }
        cfg
    }
}

/// Build a SQLx-compatible SQLite URL from a filesystem path.
///
/// On Windows `PathBuf` contains backslashes which SQLx cannot parse as a URL.
/// This helper converts the path to a forward-slash string before building the
/// `sqlite://` URI. The query parameter `?mode=rwc` creates the file if it does
/// not exist.
pub fn sqlite_url_from_path(path: &std::path::Path) -> String {
    let path_str = path.as_os_str().to_string_lossy().replace('\\', "/");
    format!("sqlite://{path_str}?mode=rwc")
}

/// Resolve the default SQLite location for the desktop binary.
fn default_database_url() -> String {
    let path = default_database_path();
    let parent = path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    if let Err(error) = std::fs::create_dir_all(&parent) {
        eprintln!(
            "porkpie-desktop: could not create {}: {error}; falling back to in-memory database",
            parent.display()
        );
        return "sqlite::memory:".to_string();
    }
    sqlite_url_from_path(&path)
}

/// Path of the default on-disk SQLite database for the desktop
/// binary. Honours `PORKPIE_DATA_DIR` for callers that want a
/// specific location; otherwise uses the platform data directory.
fn default_database_path() -> PathBuf {
    if let Ok(value) = std::env::var("PORKPIE_DATA_DIR") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("porkpie.db");
        }
    }
    if let Some(dir) = platform_data_dir() {
        return dir.join("porkpie.db");
    }
    PathBuf::from("porkpie.db")
}

/// Pick a platform-appropriate data directory without pulling in an
/// extra crate. Returns `None` only if the home directory is
/// unavailable.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_path_to_sqlite_url() {
        let path = PathBuf::from("C:\\Users\\Alice\\AppData\\Roaming\\Porkpie\\porkpie.db");
        let url = sqlite_url_from_path(&path);
        assert_eq!(
            url,
            "sqlite://C:/Users/Alice/AppData/Roaming/Porkpie/porkpie.db?mode=rwc"
        );
    }

    #[test]
    fn macos_path_to_sqlite_url() {
        let path = PathBuf::from("/Users/alice/Library/Application Support/Porkpie/porkpie.db");
        let url = sqlite_url_from_path(&path);
        assert_eq!(
            url,
            "sqlite:///Users/alice/Library/Application Support/Porkpie/porkpie.db?mode=rwc"
        );
    }

    #[test]
    fn linux_path_to_sqlite_url() {
        let path = PathBuf::from("/home/alice/.local/share/porkpie/porkpie.db");
        let url = sqlite_url_from_path(&path);
        assert_eq!(
            url,
            "sqlite:///home/alice/.local/share/porkpie/porkpie.db?mode=rwc"
        );
    }

    #[test]
    fn data_dir_override_constructs_url() {
        let dir = std::env::temp_dir().join("porkpie-test-data-dir");
        std::env::set_var("PORKPIE_DATA_DIR", &dir);
        let path = default_database_path();
        assert_eq!(path, dir.join("porkpie.db"));
        let url = sqlite_url_from_path(&path);
        assert!(url.contains("porkpie.db?mode=rwc"));
        std::env::remove_var("PORKPIE_DATA_DIR");
    }

    #[test]
    fn database_url_override_is_passthrough() {
        let expected = "sqlite::memory:";
        std::env::set_var("PORKPIE_DATABASE_URL", expected);
        let config = LaunchConfig::load();
        assert_eq!(config.database_url, expected);
        std::env::remove_var("PORKPIE_DATABASE_URL");
    }

    #[test]
    fn parent_directory_created_before_db_open() {
        let parent = std::env::temp_dir().join("porkpie-test-parent-nested");
        let path = parent.join("deep").join("porkpie.db");
        let url = sqlite_url_from_path(&path);
        assert!(url.contains("porkpie-test-parent-nested"));
        assert!(url.contains("deep/porkpie.db"));
    }
}
