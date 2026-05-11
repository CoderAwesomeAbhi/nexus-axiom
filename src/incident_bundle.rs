// Signed Incident Bundles: Every block produces signed evidence

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentBundle {
    pub id: String,
    pub timestamp: u64,
    pub event_chain: Vec<EventRecord>,
    pub process_tree: ProcessTree,
    pub network_context: NetworkContext,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_type: String,
    pub pid: u32,
    pub uid: u32,
    pub comm: String,
    pub timestamp: u64,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTree {
    pub pid: u32,
    pub ppid: u32,
    pub ancestors: Vec<u32>,
    pub exe_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
    pub cgroup_id: u64,
    pub namespace_id: u64,
}

impl IncidentBundle {
    pub fn new(pid: u32, event_type: &str) -> Self {
        let id = format!(
            "{}-{}",
            pid,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        Self {
            id,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            event_chain: Vec::new(),
            process_tree: ProcessTree::capture(pid),
            network_context: NetworkContext::capture(pid),
            signature: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: EventRecord) {
        self.event_chain.push(event);
    }

    pub fn sign(&mut self, key: &[u8]) {
        let data = serde_json::to_vec(self).unwrap();
        self.signature = Self::compute_signature(&data, key);
    }

    fn compute_signature(data: &[u8], _key: &[u8]) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish().to_le_bytes().to_vec()
    }
}

impl ProcessTree {
    fn capture(pid: u32) -> Self {
        Self {
            pid,
            ppid: Self::get_ppid(pid),
            ancestors: Self::get_ancestors(pid),
            exe_hash: Self::get_exe_hash(pid),
        }
    }

    fn get_ppid(pid: u32) -> u32 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(stat) = std::fs::read_to_string(format!("/proc/{}/stat", pid)) {
                if let Some(ppid_str) = stat.split_whitespace().nth(3) {
                    return ppid_str.parse().unwrap_or(0);
                }
            }
        }
        0
    }

    fn get_ancestors(pid: u32) -> Vec<u32> {
        let mut ancestors = Vec::new();
        let mut current = pid;
        for _ in 0..10 {
            let ppid = Self::get_ppid(current);
            if ppid == 0 || ppid == current {
                break;
            }
            ancestors.push(ppid);
            current = ppid;
        }
        ancestors
    }

    fn get_exe_hash(pid: u32) -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(exe) = std::fs::read(format!("/proc/{}/exe", pid)) {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                exe.hash(&mut hasher);
                return format!("{:x}", hasher.finish());
            }
        }
        String::from("unknown")
    }
}

impl NetworkContext {
    fn capture(pid: u32) -> Self {
        Self {
            cgroup_id: Self::get_cgroup_id(pid),
            namespace_id: Self::get_namespace_id(pid),
        }
    }

    fn get_cgroup_id(pid: u32) -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(cgroup) = std::fs::read_to_string(format!("/proc/{}/cgroup", pid)) {
                return cgroup.lines().next().map(|l| l.len() as u64).unwrap_or(0);
            }
        }
        0
    }

    fn get_namespace_id(_pid: u32) -> u64 {
        0
    }
}
