use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub network: NetworkConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub dashboard_port: u16,
    pub metrics_port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub mode: String,
    pub kill_on_violation: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub blocked_ips: Vec<String>,
    pub blocked_ports: Vec<u16>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_str = fs::read_to_string("config.toml")
            .or_else(|_| fs::read_to_string("/etc/nexus-axiom/config.toml"))?;
        Ok(toml::from_str(&config_str)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                dashboard_port: 8080,
                metrics_port: 9090,
            },
            security: SecurityConfig {
                mode: "enforce".to_string(),
                kill_on_violation: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "text".to_string(),
            },
            network: NetworkConfig {
                blocked_ips: vec![],
                blocked_ports: vec![],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.dashboard_port, 8080);
        assert_eq!(config.server.metrics_port, 9090);
        assert_eq!(config.security.mode, "enforce");
        assert!(config.security.kill_on_violation);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_values() {
        let config = Config::default();
        assert!(config.server.dashboard_port > 0);
        assert!(config.server.metrics_port > 0);
        assert!(!config.security.mode.is_empty());
    }
}
