// Replay Engine: Test policies against captured events

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEngine {
    pub events: Vec<CapturedEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedEvent {
    pub pid: u32,
    pub uid: u32,
    pub comm: String,
    pub event_type: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct ReplayResult {
    pub total_events: usize,
    pub would_block: usize,
    pub would_allow: usize,
    pub differences: Vec<Difference>,
}

#[derive(Debug, Serialize)]
pub struct Difference {
    pub event: CapturedEvent,
    pub old_action: String,
    pub new_action: String,
}

impl ReplayEngine {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let data = fs::read_to_string(path)?;
        let events = serde_json::from_str(&data)?;
        Ok(Self { events })
    }

    pub fn replay<F>(&self, policy_fn: F) -> ReplayResult
    where
        F: Fn(&CapturedEvent) -> bool,
    {
        let mut would_block = 0;
        let mut would_allow = 0;
        let mut differences = Vec::new();

        for event in &self.events {
            let blocked = policy_fn(event);
            if blocked {
                would_block += 1;
            } else {
                would_allow += 1;
            }
        }

        ReplayResult {
            total_events: self.events.len(),
            would_block,
            would_allow,
            differences,
        }
    }
}
