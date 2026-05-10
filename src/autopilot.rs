// Audit→Enforce Autopilot: Learn normal behavior, generate policies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorProfile {
    pub process_patterns: HashMap<String, ProcessBehavior>,
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessBehavior {
    pub comm: String,
    pub mmap_wx_count: u64,
    pub exec_count: u64,
    pub file_writes: Vec<String>,
    pub confidence: f64,
}

impl BehaviorProfile {
    pub fn new() -> Self {
        Self {
            process_patterns: HashMap::new(),
            confidence_threshold: 0.95,
        }
    }

    pub fn learn(&mut self, comm: &str, event_type: &str) {
        let behavior = self.process_patterns
            .entry(comm.to_string())
            .or_insert(ProcessBehavior {
                comm: comm.to_string(),
                mmap_wx_count: 0,
                exec_count: 0,
                file_writes: Vec::new(),
                confidence: 0.0,
            });

        match event_type {
            "mmap_wx" => behavior.mmap_wx_count += 1,
            "exec" => behavior.exec_count += 1,
            _ => {}
        }

        // Update confidence based on observation count
        let total_events = behavior.mmap_wx_count + behavior.exec_count;
        behavior.confidence = (total_events as f64 / 100.0).min(1.0);
    }

    pub fn generate_policy(&self) -> String {
        let mut policy = String::from("# Auto-generated policy from audit mode\n\n");
        
        for (comm, behavior) in &self.process_patterns {
            if behavior.confidence >= self.confidence_threshold {
                if behavior.mmap_wx_count == 0 {
                    policy.push_str(&format!("workload:{} can mmap wx=deny  # confidence: {:.2}\n", 
                        comm, behavior.confidence));
                } else {
                    policy.push_str(&format!("workload:{} can mmap wx=allow  # JIT detected, confidence: {:.2}\n", 
                        comm, behavior.confidence));
                }
            }
        }
        
        policy
    }

    pub fn is_anomaly(&self, comm: &str, event_type: &str) -> bool {
        if let Some(behavior) = self.process_patterns.get(comm) {
            if behavior.confidence < self.confidence_threshold {
                return false; // Still learning
            }
            
            match event_type {
                "mmap_wx" => behavior.mmap_wx_count == 0,
                _ => false,
            }
        } else {
            true // Unknown process
        }
    }
}
