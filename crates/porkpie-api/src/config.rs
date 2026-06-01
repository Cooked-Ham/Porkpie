use std::net::SocketAddr;
use thiserror::Error;

/// Runtime configuration for the API server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub database_url: String,
    pub api_port: u16,
    pub api_key: String,
}

/// Configuration parsing errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("API_PORT must be a valid u16 port")]
    InvalidPort,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/porkpie.db".to_string());
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidPort)?;
        let api_key =
            std::env::var("API_KEY").unwrap_or_else(|_| "dev-key-change-in-production".to_string());

        Ok(Self {
            database_url,
            api_port,
            api_key,
        })
    }

    /// Return the server listen address.
    pub fn listen_addr(&self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], self.api_port))
    }
}
