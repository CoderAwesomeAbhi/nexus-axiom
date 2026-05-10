use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub struct TelemetryData {
    pub blocks: u64,
    pub cve_id: Option<String>,
    pub country: String,
    pub timestamp: i64,
}

pub struct Telemetry {
    enabled: Arc<AtomicBool>,
    endpoint: String,
    blocks: Arc<AtomicU64>,
}

impl Telemetry {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(enabled)),
            endpoint: "https://stats.nexus-axiom.dev/report".to_string(),
            blocks: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_block(&self, cve: Option<&str>) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        self.blocks.fetch_add(1, Ordering::Relaxed);
        let _ = self.send_async(cve);
    }

    fn send_async(&self, cve: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let data = TelemetryData {
            blocks: 1,
            cve_id: cve.map(|s| s.to_string()),
            country: "XX".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        let client = reqwest::blocking::Client::new();
        let _ = client.post(&self.endpoint).json(&data).send();
        Ok(())
    }
}
