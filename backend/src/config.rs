//! Application configuration loaded from environment variables.

use std::net::IpAddr;
use std::str::FromStr;

/// Application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL connection URL.
    pub database_url: String,
    /// Optional Redis connection URL.
    pub redis_url: Option<String>,
    /// Bind address for the HTTP server.
    pub host: IpAddr,
    /// Port for the HTTP server.
    pub port: u16,
}

impl Config {
    /// Load configuration from the environment with sensible defaults for local dev.
    ///
    /// - `DATABASE_URL` defaults to `postgres://appuser:apppassword@localhost:5432/appdb`
    /// - `REDIS_URL` is optional.
    /// - `HOST` defaults to `0.0.0.0`, `PORT` defaults to `8000`.
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://appuser:apppassword@localhost:5432/appdb".to_string());
        let redis_url = std::env::var("REDIS_URL").ok();
        let host = std::env::var("HOST")
            .ok()
            .as_deref()
            .and_then(|s| IpAddr::from_str(s).ok())
            .unwrap_or_else(|| {
                IpAddr::from_str("0.0.0.0").unwrap_or(std::net::Ipv4Addr::UNSPECIFIED.into())
            });
        let port = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8000);
        Self {
            database_url,
            redis_url,
            host,
            port,
        }
    }

    /// Socket address the server should bind to.
    pub fn bind_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::new(self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_bind_addr_uses_host_and_port() {
        let config = Config {
            database_url: "postgres://localhost:5432/test".to_string(),
            redis_url: None,
            host: IpAddr::from_str("127.0.0.1").unwrap_or(std::net::Ipv4Addr::LOCALHOST.into()),
            port: 9000,
        };
        assert_eq!(
            config.bind_addr(),
            std::net::SocketAddr::new(config.host, 9000)
        );
    }

    #[test]
    fn config_defaults_when_env_missing() {
        // `from_env` should not panic when env vars are absent; it uses defaults.
        // The test only asserts it returns a parser config without panicking.
        let config = Config::from_env();
        assert!(!config.database_url.is_empty());
        assert_eq!(config.port, 8000);
    }
}
