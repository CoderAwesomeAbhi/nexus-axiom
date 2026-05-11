use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub network: NetworkConfig,
    #[serde(default)]
    pub integrations: IntegrationsConfig,
    #[serde(default)]
    pub ml: MlConfig,
    #[serde(default)]
    pub ha: HaConfig,
    #[serde(default)]
    pub compliance: ComplianceConfig,
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
    #[serde(default)]
    pub rotation: LogRotationConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogRotationConfig {
    pub max_size_mb: u64,
    pub max_files: usize,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 100,
            max_files: 10,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub blocked_ips: Vec<String>,
    pub blocked_ports: Vec<u16>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct IntegrationsConfig {
    pub slack_webhook: Option<String>,
    pub pagerduty_key: Option<String>,
    pub datadog_api_key: Option<String>,
    pub splunk_hec_url: Option<String>,
    pub splunk_hec_token: Option<String>,
    pub syslog_host: Option<String>,
    pub syslog_port: Option<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MlConfig {
    pub enabled: bool,
    pub prediction_interval_ms: u64,
    pub threat_threshold: f32,
}

impl Default for MlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prediction_interval_ms: 500,
            threat_threshold: 0.65,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct HaConfig {
    pub enabled: bool,
    pub heartbeat_interval_secs: u64,
    pub failover_timeout_secs: u64,
    pub health_port: u16,
}

impl Default for HaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            heartbeat_interval_secs: 5,
            failover_timeout_secs: 15,
            health_port: 9091,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ComplianceConfig {
    pub enabled: bool,
    pub frameworks: Vec<String>,
    pub report_interval_hours: u64,
    pub report_path: String,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            frameworks: vec!["SOC2".into(), "ISO27001".into(), "NIST-CSF".into()],
            report_interval_hours: 24,
            report_path: "/var/lib/nexus-axiom/compliance/".into(),
        }
    }
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
                rotation: LogRotationConfig::default(),
            },
            network: NetworkConfig {
                blocked_ips: vec![],
                blocked_ports: vec![],
            },
            integrations: IntegrationsConfig::default(),
            ml: MlConfig::default(),
            ha: HaConfig::default(),
            compliance: ComplianceConfig::default(),
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
        assert!(config.ml.enabled);
        assert!(!config.ha.enabled);
    }

    #[test]
    fn test_config_values() {
        let config = Config::default();
        assert!(config.server.dashboard_port > 0);
        assert!(config.server.metrics_port > 0);
        assert!(!config.security.mode.is_empty());
    }

    #[test]
    fn test_ml_config_defaults() {
        let ml = MlConfig::default();
        assert!(ml.enabled);
        assert_eq!(ml.prediction_interval_ms, 500);
        assert!((ml.threat_threshold - 0.65).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compliance_config_defaults() {
        let c = ComplianceConfig::default();
        assert!(!c.enabled);
        assert!(!c.frameworks.is_empty());
    }
}
