//! Web launch configuration helpers.
//!
//! Encapsulates reading the web launch configuration from the
//! process environment and converting it into a `dioxus_web::Config`.

/// Browser-only configuration. Reads the host element id from
/// `PORKPIE_WEB_ROOT`. Everything else is delegated to
/// `dioxus_web::Config::default()`.
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub root_name: String,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            root_name: "main".to_string(),
        }
    }
}

impl LaunchConfig {
    /// Build a launch config from the process environment (only
    /// meaningful in the WASM build; ignored on the host target when
    /// running `cargo check` on the web crate).
    pub fn load() -> Self {
        let mut config = Self::default();
        if let Ok(value) = std::env::var("PORKPIE_WEB_ROOT") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                config.root_name = trimmed.to_string();
            }
        }
        config
    }

    /// Convert into a Dioxus web `Config`.
    pub fn into_dioxus_config(self) -> dioxus_web::Config {
        dioxus_web::Config::default().rootname(self.root_name)
    }
}
