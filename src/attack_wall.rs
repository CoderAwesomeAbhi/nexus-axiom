// Live Attack Wall: Real-time public dashboard of blocked exploits

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedAttack {
    pub timestamp: u64,
    pub attack_type: String,
    pub process: String,
    pub severity: String,
    pub location: String,  // Anonymized: "US-East", "EU-West"
}

pub struct AttackWall {
    attacks: Arc<Mutex<Vec<BlockedAttack>>>,
    broadcast: broadcast::Sender<BlockedAttack>,
}

impl AttackWall {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            attacks: Arc::new(Mutex::new(Vec::new())),
            broadcast: tx,
        }
    }

    pub fn record_block(&self, attack: BlockedAttack) {
        let mut attacks = self.attacks.lock().unwrap();
        attacks.push(attack.clone());
        
        // Keep only last 1000
        if attacks.len() > 1000 {
            attacks.remove(0);
        }
        
        // Broadcast to WebSocket clients
        let _ = self.broadcast.send(attack);
    }

    pub fn get_recent(&self, limit: usize) -> Vec<BlockedAttack> {
        let attacks = self.attacks.lock().unwrap();
        attacks.iter().rev().take(limit).cloned().collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BlockedAttack> {
        self.broadcast.subscribe()
    }

    pub fn stats(&self) -> AttackStats {
        let attacks = self.attacks.lock().unwrap();
        let total = attacks.len();
        let last_hour = attacks.iter()
            .filter(|a| {
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                now - a.timestamp < 3600
            })
            .count();
        
        AttackStats {
            total_blocked: total,
            last_hour,
            uptime_seconds: 0,  // TODO: track uptime
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AttackStats {
    pub total_blocked: usize,
    pub last_hour: usize,
    pub uptime_seconds: u64,
}
