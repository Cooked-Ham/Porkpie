use std::net::SocketAddr;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    pub database_url: String,
    pub api_port: u16,
    pub api_key: String,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &self.database_url)
            .field("api_port", &self.api_port)
            .field("api_key", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("API_PORT must be a valid u16 port")]
    InvalidPort,
    #[error("API_KEY environment variable is required but not set")]
    MissingApiKey,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/porkpie.db".to_string());
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidPort)?;
        let api_key = std::env::var("API_KEY")
            .map_err(|_| ConfigError::MissingApiKey)?
            .trim()
            .to_string();
        if api_key.is_empty() {
            return Err(ConfigError::MissingApiKey);
        }

        Ok(Self {
            database_url,
            api_port,
            api_key,
        })
    }

    pub fn listen_addr(&self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], self.api_port))
    }
}
