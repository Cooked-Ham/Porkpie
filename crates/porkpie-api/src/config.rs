use std::net::SocketAddr;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub api_key: String,
    pub cors_allowed_origins: Vec<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &self.database_url)
            .field("bind_addr", &self.bind_addr)
            .field("api_key", &"[redacted]")
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .finish()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("PORKPIE_SERVER_BIND must be a valid socket address")]
    InvalidBindAddr,
    #[error("PORKPIE_API_KEY environment variable is required but not set")]
    MissingApiKey,
    #[error("PORKPIE_API_KEY must be a non-placeholder secret at least 32 characters long")]
    WeakApiKey,
    #[error("PORKPIE_CORS_ALLOWED_ORIGINS contains an invalid origin URL: {0}")]
    InvalidCorsOrigin(String),
}

const PLACEHOLDER_KEYS: &[&str] = &[
    "replace-with-a-generated-secret",
    "change-me",
    "changeme",
    "dev",
    "test",
    "password",
    "secret",
    "porkpie",
];

const MIN_API_KEY_LEN: usize = 32;

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("PORKPIE_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "sqlite:data/porkpie.db".to_string());

        let bind_addr = std::env::var("PORKPIE_SERVER_BIND")
            .or_else(|_| std::env::var("API_PORT").map(|port| format!("0.0.0.0:{port}")))
            .unwrap_or_else(|_| "0.0.0.0:8000".to_string())
            .parse::<SocketAddr>()
            .map_err(|_| ConfigError::InvalidBindAddr)?;

        let api_key = std::env::var("PORKPIE_API_KEY")
            .or_else(|_| std::env::var("API_KEY"))
            .map_err(|_| ConfigError::MissingApiKey)?
            .trim()
            .to_string();
        if api_key.is_empty() {
            return Err(ConfigError::MissingApiKey);
        }
        if api_key.len() < MIN_API_KEY_LEN {
            return Err(ConfigError::WeakApiKey);
        }
        let api_key_lower = api_key.to_ascii_lowercase();
        if PLACEHOLDER_KEYS.iter().any(|p| api_key_lower == *p) {
            return Err(ConfigError::WeakApiKey);
        }

        let cors_allowed_origins = parse_cors_allowed_origins()?;

        Ok(Self {
            database_url,
            bind_addr,
            api_key,
            cors_allowed_origins,
        })
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.bind_addr
    }
}

fn parse_cors_allowed_origins() -> Result<Vec<String>, ConfigError> {
    let raw = std::env::var("PORKPIE_CORS_ALLOWED_ORIGINS")
        .or_else(|_| std::env::var("CORS_ALLOWED_ORIGINS"))
        .unwrap_or_else(|_| "https://app.porkpie.love".to_string());

    let mut origins = Vec::new();
    for origin in raw.split(',') {
        let trimmed = origin.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Err(e) = validate_origin(trimmed) {
            return Err(ConfigError::InvalidCorsOrigin(format!("{trimmed}: {e}")));
        }
        origins.push(trimmed.to_string());
    }

    if origins.is_empty() {
        origins.push("https://app.porkpie.love".to_string());
    }

    Ok(origins)
}

fn validate_origin(origin: &str) -> Result<(), &'static str> {
    if origin == "*" {
        return Err("wildcard '*' is not allowed; list explicit origins");
    }
    let url = url::Url::parse(origin).map_err(|_| "must be a valid URL with scheme and host")?;
    if !["http", "https"].contains(&url.scheme()) {
        return Err("origin scheme must be http or https");
    }
    if url.host().is_none() {
        return Err("origin must have a host");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn set_env(key: &str, value: &str) {
        std::env::set_var(key, value);
    }

    fn clear_api_key_env() {
        std::env::remove_var("PORKPIE_API_KEY");
        std::env::remove_var("API_KEY");
    }

    fn clear_bind_env() {
        std::env::remove_var("PORKPIE_SERVER_BIND");
        std::env::remove_var("API_PORT");
    }

    fn clear_cors_env() {
        std::env::remove_var("PORKPIE_CORS_ALLOWED_ORIGINS");
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    fn setup_valid() {
        clear_api_key_env();
        clear_bind_env();
        clear_cors_env();
        set_env(
            "PORKPIE_API_KEY",
            "a-very-long-generated-secret-key-that-is-safe-64chars!",
        );
        set_env("PORKPIE_SERVER_BIND", "127.0.0.1:8000");
    }

    #[test]
    fn missing_key_rejected() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_api_key_env();
        clear_bind_env();
        clear_cors_env();
        set_env("PORKPIE_SERVER_BIND", "127.0.0.1:8000");
        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::MissingApiKey)));
    }

    #[test]
    fn empty_key_rejected() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_api_key_env();
        clear_bind_env();
        clear_cors_env();
        set_env("PORKPIE_API_KEY", "");
        set_env("PORKPIE_SERVER_BIND", "127.0.0.1:8000");
        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::MissingApiKey)));
    }

    #[test]
    fn placeholder_replace_with_a_generated_secret_rejected() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        set_env("PORKPIE_API_KEY", "replace-with-a-generated-secret");
        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::WeakApiKey)));
    }

    #[test]
    fn placeholder_change_me_rejected() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        set_env("PORKPIE_API_KEY", "change-me");
        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::WeakApiKey)));
    }

    #[test]
    fn short_key_rejected() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        set_env("PORKPIE_API_KEY", "short");
        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::WeakApiKey)));
    }

    #[test]
    fn valid_long_key_accepted() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        let result = Config::from_env();
        assert!(result.is_ok());
    }

    #[test]
    fn config_debug_redacts_api_key() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        let config = Config::from_env().unwrap();
        let debug = format!("{:?}", config);
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains(&config.api_key));
    }

    #[test]
    fn cors_origin_wildcard_rejected() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        std::env::set_var("PORKPIE_CORS_ALLOWED_ORIGINS", "*");
        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::InvalidCorsOrigin(_))));
    }

    #[test]
    fn cors_origin_invalid_url_rejected() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        std::env::set_var("PORKPIE_CORS_ALLOWED_ORIGINS", "not-a-url");
        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::InvalidCorsOrigin(_))));
    }

    #[test]
    fn cors_origin_ftp_rejected() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        std::env::set_var("PORKPIE_CORS_ALLOWED_ORIGINS", "ftp://evil.com");
        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::InvalidCorsOrigin(_))));
    }

    #[test]
    fn cors_origin_multiple_valid_accepted() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        std::env::set_var(
            "PORKPIE_CORS_ALLOWED_ORIGINS",
            "https://app.porkpie.love, https://admin.porkpie.love",
        );
        let config = Config::from_env().unwrap();
        assert_eq!(config.cors_allowed_origins.len(), 2);
        assert!(config
            .cors_allowed_origins
            .contains(&"https://app.porkpie.love".to_string()));
        assert!(config
            .cors_allowed_origins
            .contains(&"https://admin.porkpie.love".to_string()));
    }

    #[test]
    fn cors_origin_empty_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        setup_valid();
        std::env::remove_var("PORKPIE_CORS_ALLOWED_ORIGINS");
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
        let config = Config::from_env().unwrap();
        assert_eq!(
            config.cors_allowed_origins,
            vec!["https://app.porkpie.love".to_string()]
        );
    }
}
