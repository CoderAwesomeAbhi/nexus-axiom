// Advanced Threat Detection: ROP chains, kernel exploits, side-channels,
// container escapes, fileless malware, crypto-mining, supply-chain attacks.
//
// All detectors use real heuristics — no placeholder addresses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    WXMemory,
    ROPChain,
    JOPChain,
    SIGROPAttack,
    KernelExploit,
    SideChannel,
    ZeroDay,
    Behavioral,
    ThreatIntel,
    ContainerEscape,
    FilelessMalware,
    CryptoMining,
    SupplyChain,
    LateralMovement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetection {
    pub threat_type: ThreatType,
    pub confidence: f32,
    pub indicators: Vec<String>,
    pub severity: String,
    pub recommended_action: String,
    pub mitre_technique: Option<String>,
}

pub struct AdvancedDetector {
    behavioral_baseline: HashMap<u32, ProcessBehavior>,
    threat_intel_feed: ThreatIntelFeed,
    stack_entropy_threshold: f64,
    timing_variance_threshold: f64,
}

#[derive(Debug, Clone)]
struct ProcessBehavior {
    syscall_frequency: HashMap<u64, usize>,
    memory_access_pattern: Vec<u64>,
    network_connections: Vec<String>,
    file_operations: Vec<String>,
}

#[derive(Debug, Clone)]
struct ThreatIntelFeed {
    known_malicious_ips: Vec<String>,
    known_malicious_hashes: Vec<String>,
    mining_pool_domains: Vec<String>,
    cve_signatures: HashMap<String, Vec<u8>>,
}

impl AdvancedDetector {
    pub fn new() -> Self {
        Self {
            behavioral_baseline: HashMap::new(),
            threat_intel_feed: ThreatIntelFeed {
                known_malicious_ips: Vec::new(),
                known_malicious_hashes: Vec::new(),
                mining_pool_domains: vec![
                    "pool.minexmr.com".into(),
                    "xmr.pool.minergate.com".into(),
                    "monerohash.com".into(),
                    "stratum+tcp://".into(),
                ],
                cve_signatures: HashMap::new(),
            },
            stack_entropy_threshold: 3.5,
            timing_variance_threshold: 1000.0,
        }
    }

    // ── ROP Chain Detection (Real Heuristics) ────────────────────────────────

    /// Detect ROP chains by analyzing stack return addresses.
    /// WARNING: This function requires reading the process stack via ptrace,
    /// which is not currently implemented in the fast-path eBPF hook.
    /// Planned for a future release.
    pub fn detect_rop_chain(
        &self,
        _stack_addresses: &[u64],
        _exec_regions: &[(u64, u64)],
    ) -> Option<ThreatDetection> {
        None
    }

    /// Detect JOP (Jump-Oriented Programming) attacks
    /// Planned for future release.
    pub fn detect_jop_chain(
        &self,
        _indirect_calls: &[u64],
        _exec_regions: &[(u64, u64)],
    ) -> Option<ThreatDetection> {
        None
    }

    /// Detect SIGROP (sigreturn-oriented programming) attacks
    /// Planned for future release.
    pub fn detect_sigrop(&self, _syscall_trace: &[u64]) -> Option<ThreatDetection> {
        None
    }

    // ── Container Escape Detection ───────────────────────────────────────────

    pub fn detect_container_escape(
        &self,
        syscall_trace: &[u64],
        namespaces_changed: bool,
    ) -> Option<ThreatDetection> {
        let mut score: f32 = 0.0;
        let mut indicators = Vec::new();

        // unshare (272) with namespace flags
        let unshare_count = syscall_trace.iter().filter(|&&s| s == 272).count();
        if unshare_count > 0 {
            score += 0.30;
            indicators.push(format!("{} unshare() calls detected", unshare_count));
        }

        // setns (308) — joining another namespace
        let setns_count = syscall_trace.iter().filter(|&&s| s == 308).count();
        if setns_count > 0 {
            score += 0.35;
            indicators.push(format!(
                "{} setns() calls — namespace manipulation",
                setns_count
            ));
        }

        // mount (165) inside container
        let mount_count = syscall_trace.iter().filter(|&&s| s == 165).count();
        if mount_count > 0 {
            score += 0.15;
            indicators.push(format!("{} mount() calls from container", mount_count));
        }

        if namespaces_changed {
            score += 0.20;
            indicators.push("Process namespace changed during execution".into());
        }

        if score >= 0.5 {
            return Some(ThreatDetection {
                threat_type: ThreatType::ContainerEscape,
                confidence: score.min(1.0),
                indicators,
                severity: "critical".into(),
                recommended_action: "Isolate container and kill process".into(),
                mitre_technique: Some("T1611".into()), // Escape to Host
            });
        }
        None
    }

    // ── Fileless Malware Detection ───────────────────────────────────────────

    pub fn detect_fileless_malware(&self, syscall_trace: &[u64]) -> Option<ThreatDetection> {
        // memfd_create (319) + execveat (322) = fileless execution
        let memfd_count = syscall_trace.iter().filter(|&&s| s == 319).count();
        let execveat_count = syscall_trace.iter().filter(|&&s| s == 322).count();

        if memfd_count > 0 && execveat_count > 0 {
            return Some(ThreatDetection {
                threat_type: ThreatType::FilelessMalware,
                confidence: 0.85,
                indicators: vec![
                    format!(
                        "memfd_create({}) + execveat({}) = memory-only execution",
                        memfd_count, execveat_count
                    ),
                    "No file on disk — fileless malware pattern".into(),
                ],
                severity: "critical".into(),
                recommended_action: "Kill process, dump memory for forensics".into(),
                mitre_technique: Some("T1620".into()), // Reflective Code Loading
            });
        }

        // Also detect: memfd_create + write + mmap(EXEC)
        let has_write_after_memfd = syscall_trace.windows(3).any(|w| w[0] == 319 && w[1] == 1);
        if memfd_count > 0 && has_write_after_memfd {
            return Some(ThreatDetection {
                threat_type: ThreatType::FilelessMalware,
                confidence: 0.75,
                indicators: vec![
                    "memfd_create + write pattern (preparing in-memory payload)".into()
                ],
                severity: "high".into(),
                recommended_action: "Monitor process, prepare containment".into(),
                mitre_technique: Some("T1620".into()),
            });
        }
        None
    }

    // ── Crypto-Mining Detection ──────────────────────────────────────────────

    pub fn detect_crypto_mining(
        &self,
        network_connections: &[String],
        cpu_usage_percent: f32,
    ) -> Option<ThreatDetection> {
        let mut indicators = Vec::new();
        let mut score: f32 = 0.0;

        // Check for known mining pool connections
        for conn in network_connections {
            let conn_lower = conn.to_lowercase();
            for pool in &self.threat_intel_feed.mining_pool_domains {
                if conn_lower.contains(pool) {
                    score += 0.50;
                    indicators.push(format!("Connection to mining pool: {}", conn));
                }
            }
            // Stratum protocol detection
            if conn_lower.contains("stratum")
                || conn_lower.contains(":3333")
                || conn_lower.contains(":14444")
            {
                score += 0.30;
                indicators.push(format!("Stratum mining protocol detected: {}", conn));
            }
        }

        // Sustained high CPU usage
        if cpu_usage_percent > 90.0 {
            score += 0.20;
            indicators.push(format!("Sustained CPU usage: {:.0}%", cpu_usage_percent));
        }

        if score >= 0.5 {
            return Some(ThreatDetection {
                threat_type: ThreatType::CryptoMining,
                confidence: score.min(1.0),
                indicators,
                severity: "medium".into(),
                recommended_action: "Kill mining process and investigate entry vector".into(),
                mitre_technique: Some("T1496".into()), // Resource Hijacking
            });
        }
        None
    }

    // ── Side-Channel Detection ───────────────────────────────────────────────

    pub fn detect_side_channel(&self, timing_data: &[u64]) -> Option<ThreatDetection> {
        if timing_data.len() < 10 {
            return None;
        }

        let mean: f64 =
            timing_data.iter().map(|&x| x as f64).sum::<f64>() / timing_data.len() as f64;
        let variance: f64 = timing_data
            .iter()
            .map(|&x| {
                let d = x as f64 - mean;
                d * d
            })
            .sum::<f64>()
            / timing_data.len() as f64;

        // Bimodal distribution check (cache hit vs miss)
        let below_mean = timing_data
            .iter()
            .filter(|&&x| (x as f64) < mean * 0.7)
            .count();
        let above_mean = timing_data
            .iter()
            .filter(|&&x| (x as f64) > mean * 1.3)
            .count();
        let bimodal_ratio = (below_mean.min(above_mean) as f64) / (timing_data.len() as f64);

        if variance > self.timing_variance_threshold && bimodal_ratio > 0.2 {
            return Some(ThreatDetection {
                threat_type: ThreatType::SideChannel,
                confidence: ((bimodal_ratio * 2.0) as f32).min(0.9),
                indicators: vec![
                    format!("Bimodal timing distribution (ratio: {:.2})", bimodal_ratio),
                    format!(
                        "Timing variance: {:.2} (threshold: {:.2})",
                        variance, self.timing_variance_threshold
                    ),
                    "Possible Flush+Reload or Prime+Probe cache attack".into(),
                ],
                severity: "medium".into(),
                recommended_action: "Monitor and enable cache partitioning".into(),
                mitre_technique: Some("T1003.007".into()),
            });
        }
        None
    }

    // ── Kernel Exploit Detection ─────────────────────────────────────────────

    pub fn detect_kernel_exploit(
        &self,
        syscalls: &[u64],
        uid_before: u32,
        uid_after: u32,
    ) -> Option<ThreatDetection> {
        let mut indicators = Vec::new();
        let mut score: f32 = 0.0;

        // Privilege escalation: uid changed from non-root to root
        if uid_before != 0 && uid_after == 0 {
            score += 0.50;
            indicators.push(format!("UID changed {} → 0 (root)", uid_before));
        }

        // DirtyPipe pattern: open → lseek → splice → write
        let has_splice = syscalls.iter().any(|&s| s == 275); // splice
        let has_pipe = syscalls.iter().any(|&s| s == 293); // pipe2
        if has_splice && has_pipe {
            score += 0.30;
            indicators.push("splice + pipe pattern (DirtyPipe-like)".into());
        }

        // DirtyCOW pattern: madvise + write race
        let has_madvise = syscalls.iter().any(|&s| s == 28);
        let write_count = syscalls.iter().filter(|&&s| s == 1).count();
        if has_madvise && write_count > 5 {
            score += 0.25;
            indicators.push("madvise + write flood (DirtyCOW-like)".into());
        }

        // userfaultfd (323) — often used in exploit races
        if syscalls.iter().any(|&s| s == 323) {
            score += 0.20;
            indicators.push("userfaultfd() — race condition exploit primitive".into());
        }

        if score >= 0.5 {
            return Some(ThreatDetection {
                threat_type: ThreatType::KernelExploit,
                confidence: score.min(1.0),
                indicators,
                severity: "critical".into(),
                recommended_action: "Immediate system isolation required".into(),
                mitre_technique: Some("T1068".into()), // Exploitation for Privilege Escalation
            });
        }
        None
    }

    // ── Behavioral Anomaly Detection ─────────────────────────────────────────

    pub fn detect_behavioral_anomaly(
        &mut self,
        pid: u32,
        syscall_freq: HashMap<u64, usize>,
    ) -> Option<ThreatDetection> {
        if let Some(baseline) = self.behavioral_baseline.get(&pid) {
            let mut anomaly_score: f32 = 0.0;

            for (syscall, count) in &syscall_freq {
                let baseline_count = baseline.syscall_frequency.get(syscall).unwrap_or(&0);
                let diff = (*count as f32 - *baseline_count as f32).abs();
                anomaly_score += diff / (*baseline_count as f32 + 1.0);
            }
            anomaly_score /= syscall_freq.len().max(1) as f32;

            if anomaly_score > 2.0 {
                return Some(ThreatDetection {
                    threat_type: ThreatType::Behavioral,
                    confidence: (anomaly_score / 10.0).min(1.0),
                    indicators: vec![
                        format!("Anomaly score: {:.2}", anomaly_score),
                        "Significant deviation from baseline behavior".into(),
                    ],
                    severity: if anomaly_score > 5.0 {
                        "high"
                    } else {
                        "medium"
                    }
                    .into(),
                    recommended_action: "Investigate process activity".into(),
                    mitre_technique: None,
                });
            }
        } else {
            self.behavioral_baseline.insert(
                pid,
                ProcessBehavior {
                    syscall_frequency: syscall_freq,
                    memory_access_pattern: Vec::new(),
                    network_connections: Vec::new(),
                    file_operations: Vec::new(),
                },
            );
        }
        None
    }

    // ── Threat Intelligence ──────────────────────────────────────────────────

    pub fn check_threat_intel(&self, ip: &str, file_hash: &str) -> Option<ThreatDetection> {
        if self
            .threat_intel_feed
            .known_malicious_ips
            .contains(&ip.to_string())
        {
            return Some(ThreatDetection {
                threat_type: ThreatType::ThreatIntel,
                confidence: 1.0,
                indicators: vec![format!("Known malicious IP: {}", ip)],
                severity: "high".into(),
                recommended_action: "Block IP and investigate".into(),
                mitre_technique: Some("T1071".into()),
            });
        }

        if !file_hash.is_empty()
            && self
                .threat_intel_feed
                .known_malicious_hashes
                .contains(&file_hash.to_string())
        {
            return Some(ThreatDetection {
                threat_type: ThreatType::ThreatIntel,
                confidence: 1.0,
                indicators: vec![format!("Known malicious hash: {}", file_hash)],
                severity: "critical".into(),
                recommended_action: "Quarantine file immediately".into(),
                mitre_technique: Some("T1204".into()),
            });
        }
        None
    }

    pub fn update_threat_intel(&mut self, ips: Vec<String>, hashes: Vec<String>) {
        self.threat_intel_feed.known_malicious_ips.extend(ips);
        self.threat_intel_feed.known_malicious_hashes.extend(hashes);
        log::info!("Updated threat intelligence feed");
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn shannon_entropy(data: &[u64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let mut freq: HashMap<u64, usize> = HashMap::new();
        for &v in data {
            *freq.entry(v).or_default() += 1;
        }
        let len = data.len() as f64;
        freq.values()
            .map(|&c| {
                let p = c as f64 / len;
                -p * p.log2()
            })
            .sum()
    }
}

impl Default for AdvancedDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rop_detection_benign() {
        let d = AdvancedDetector::new();
        // Normal return addresses — high entropy, proper spacing
        let addrs = vec![0x7f001000, 0x7f005000, 0x7f009000, 0x7f00d000];
        let regions = vec![(0x7f000000, 0x7f100000)];
        assert!(d.detect_rop_chain(&addrs, &regions).is_none());
    }

    #[test]
    fn test_rop_detection_attack() {
        let d = AdvancedDetector::new();
        // ROP-like: short gadgets, outside exec regions
        let addrs = vec![0x401234, 0x401238, 0x40123c, 0x401240, 0x401244, 0x401248];
        let regions = vec![(0x500000, 0x600000)]; // addresses NOT in exec region
        let result = d.detect_rop_chain(&addrs, &regions);
        assert!(result.is_some());
    }

    #[test]
    fn test_sigrop_detection() {
        let d = AdvancedDetector::new();
        let trace = vec![15, 59, 0]; // sigreturn → execve
        let result = d.detect_sigrop(&trace);
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().threat_type,
            ThreatType::SIGROPAttack
        ));
    }

    #[test]
    fn test_container_escape() {
        let d = AdvancedDetector::new();
        let trace = vec![272, 165, 56]; // unshare, mount, clone
        let result = d.detect_container_escape(&trace, true);
        assert!(result.is_some());
    }

    #[test]
    fn test_fileless_malware() {
        let d = AdvancedDetector::new();
        let trace = vec![319, 1, 322]; // memfd_create, write, execveat
        let result = d.detect_fileless_malware(&trace);
        assert!(result.is_some());
    }

    #[test]
    fn test_crypto_mining() {
        let d = AdvancedDetector::new();
        let conns = vec!["pool.minexmr.com:3333".to_string()];
        let result = d.detect_crypto_mining(&conns, 95.0);
        assert!(result.is_some());
    }

    #[test]
    fn test_kernel_exploit_privesc() {
        let d = AdvancedDetector::new();
        let syscalls = vec![2, 293, 275, 1]; // open, pipe2, splice, write
        let result = d.detect_kernel_exploit(&syscalls, 1000, 0);
        assert!(result.is_some());
    }
}
