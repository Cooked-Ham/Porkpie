use std::net::SocketAddr;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub api_key: String,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &self.database_url)
            .field("bind_addr", &self.bind_addr)
            .field("api_key", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("PORKPIE_SERVER_BIND must be a valid socket address")]
    InvalidBindAddr,
    #[error("PORKPIE_API_KEY environment variable is required but not set")]
    MissingApiKey,
}

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

        Ok(Self {
            database_url,
            bind_addr,
            api_key,
        })
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.bind_addr
    }
}
