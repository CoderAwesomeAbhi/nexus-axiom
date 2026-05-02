#![allow(dead_code)]

//! Temporal Emulation Graph: Defeating Point-in-Time Evasion
//!
//! Advanced malware evades AI by spreading its behavior across multiple processes
//! and hiding malicious intent across isolated operations.
//! 
//! This module tracks process ancestry and operation sequences, feeding the LLM
//! a continuous behavioral graph instead of a single isolated event.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::ProcessEvent;

const MAX_HISTORY_PER_PROCESS: usize = 20;

/// A node in the execution graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessNode {
    pub pid: u32,
    pub ppid: u32,
    pub process_name: String,
    pub spawn_time: u64,
    /// Chronological list of critical events
    pub events: VecDeque<String>,
}

pub struct TemporalGraph {
    /// pid -> ProcessNode
    nodes: Arc<RwLock<HashMap<u32, ProcessNode>>>,
}

impl TemporalGraph {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Ingest a new event and update the graph
    pub async fn ingest(&self, event: &ProcessEvent, ppid: u32) {
        let mut graph = self.nodes.write().await;

        let node = graph.entry(event.pid).or_insert_with(|| ProcessNode {
            pid: event.pid,
            ppid,
            process_name: event.process_name.clone(),
            spawn_time: event.timestamp,
            events: VecDeque::with_capacity(MAX_HISTORY_PER_PROCESS),
        });

        // Condense the event for the graph trace
        let trace = match event.operation {
            crate::OperationType::FileWrite => format!("WRITE {}", event.target_path.as_deref().unwrap_or("unknown")),
            crate::OperationType::FileMmap => format!("MMAP(prot={:?}) {}", event.mmap_prot, event.target_path.as_deref().unwrap_or("anon")),
            crate::OperationType::SocketConnect => format!("CONNECT {}", event.target_addr.as_deref().unwrap_or("unknown")),
            crate::OperationType::ProcessExec => format!("EXEC {}", event.target_path.as_deref().unwrap_or("unknown")),
            _ => format!("{:?}", event.operation),
        };

        if node.events.len() >= MAX_HISTORY_PER_PROCESS {
            node.events.pop_front();
        }
        node.events.push_back(trace);
    }

    /// Extract the temporal context for a given PID to feed to the LLM
    pub async fn extract_context(&self, pid: u32) -> TemporalContext {
        let graph = self.nodes.read().await;
        
        let mut ancestry = Vec::new();
        let mut current_pid = pid;

        // Walk up the tree (max 3 levels to avoid massive payloads)
        for _ in 0..3 {
            if let Some(node) = graph.get(&current_pid) {
                ancestry.push(node.clone());
                current_pid = node.ppid;
                if current_pid == 0 || current_pid == 1 {
                    break;
                }
            } else {
                break;
            }
        }

        TemporalContext {
            target_pid: pid,
            ancestry,
        }
    }
}

/// The structured graph sequence sent to the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    pub target_pid: u32,
    pub ancestry: Vec<ProcessNode>,
}
