// Multi-signal correlation: Detect attack chains across events.
// Supports time-decay scoring, cross-PID correlation, and MITRE ATT&CK mapping.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct Event {
    pub pid: u32,
    pub event_type: EventType,
    pub timestamp: SystemTime,
    pub uid: u32,
    pub comm: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    MmapWX,
    MprotectWX,
    Exec,
    Ptrace,
    NetworkConnect,
    FileAccess,
    Unshare,
    MemfdCreate,
    Mount,
    Sigreturn,
}

#[derive(Debug)]
pub struct AttackChain {
    pub events: Vec<Event>,
    pub severity: f64,
    pub pattern: AttackPattern,
    pub mitre_id: &'static str,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum AttackPattern {
    ShellcodeInjection,
    PrivilegeEscalation,
    ReverseShell,
    ContainerEscape,
    FilelessExecution,
    DataExfiltration,
    LateralMovement,
    MmapExecChain,
    PtraceInjection,
    Unknown,
}

impl AttackPattern {
    pub fn mitre_id(&self) -> &'static str {
        match self {
            Self::ShellcodeInjection => "T1055.012",
            Self::PrivilegeEscalation => "T1068",
            Self::ReverseShell => "T1059.004",
            Self::ContainerEscape => "T1611",
            Self::FilelessExecution => "T1620",
            Self::DataExfiltration => "T1041",
            Self::LateralMovement => "T1021",
            Self::MmapExecChain => "T1055",
            Self::PtraceInjection => "T1055.008",
            Self::Unknown => "TA0001",
        }
    }
}

// ── Pattern definitions ──────────────────────────────────────────────────────

struct PatternDef {
    sequence: Vec<EventType>,
    pattern: AttackPattern,
    base_severity: f64,
    window: Duration,
    description: &'static str,
}

pub struct CorrelationEngine {
    event_buffer: HashMap<u32, VecDeque<Event>>,
    cross_pid_buffer: VecDeque<Event>,
    patterns: Vec<PatternDef>,
    max_window: Duration,
}

impl CorrelationEngine {
    pub fn new() -> Self {
        let patterns = vec![
            PatternDef {
                sequence: vec![EventType::MmapWX, EventType::Exec],
                pattern: AttackPattern::ShellcodeInjection,
                base_severity: 0.95, window: Duration::from_secs(10),
                description: "W^X mmap followed by exec — shellcode injection",
            },
            PatternDef {
                sequence: vec![EventType::Ptrace, EventType::MmapWX],
                pattern: AttackPattern::PrivilegeEscalation,
                base_severity: 0.90, window: Duration::from_secs(10),
                description: "ptrace followed by W^X mmap — privilege escalation",
            },
            PatternDef {
                sequence: vec![EventType::Exec, EventType::NetworkConnect],
                pattern: AttackPattern::ReverseShell,
                base_severity: 0.85, window: Duration::from_secs(5),
                description: "exec followed by network connect — reverse shell",
            },
            PatternDef {
                sequence: vec![EventType::Unshare, EventType::Mount, EventType::Exec],
                pattern: AttackPattern::ContainerEscape,
                base_severity: 0.95, window: Duration::from_secs(15),
                description: "unshare + mount + exec — container escape",
            },
            PatternDef {
                sequence: vec![EventType::MemfdCreate, EventType::Exec],
                pattern: AttackPattern::FilelessExecution,
                base_severity: 0.90, window: Duration::from_secs(10),
                description: "memfd_create + exec — fileless malware",
            },
            PatternDef {
                sequence: vec![EventType::FileAccess, EventType::NetworkConnect],
                pattern: AttackPattern::DataExfiltration,
                base_severity: 0.75, window: Duration::from_secs(30),
                description: "sensitive file access followed by network connect",
            },
            PatternDef {
                sequence: vec![EventType::MmapWX, EventType::MprotectWX, EventType::Exec],
                pattern: AttackPattern::MmapExecChain,
                base_severity: 0.92, window: Duration::from_secs(10),
                description: "W^X mmap + mprotect + exec — multi-stage injection",
            },
            PatternDef {
                sequence: vec![EventType::Ptrace, EventType::MmapWX, EventType::Exec],
                pattern: AttackPattern::PtraceInjection,
                base_severity: 0.93, window: Duration::from_secs(10),
                description: "ptrace + W^X mmap + exec — process injection",
            },
            PatternDef {
                sequence: vec![EventType::Sigreturn, EventType::Exec],
                pattern: AttackPattern::PrivilegeEscalation,
                base_severity: 0.90, window: Duration::from_secs(5),
                description: "sigreturn + exec — SIGROP attack",
            },
            PatternDef {
                sequence: vec![EventType::MprotectWX, EventType::MprotectWX, EventType::Exec],
                pattern: AttackPattern::ShellcodeInjection,
                base_severity: 0.88, window: Duration::from_secs(10),
                description: "repeated mprotect W^X + exec — staged injection",
            },
            PatternDef {
                sequence: vec![EventType::Unshare, EventType::Exec],
                pattern: AttackPattern::ContainerEscape,
                base_severity: 0.80, window: Duration::from_secs(10),
                description: "unshare + exec — namespace escape attempt",
            },
            PatternDef {
                sequence: vec![EventType::Exec, EventType::Exec, EventType::NetworkConnect],
                pattern: AttackPattern::LateralMovement,
                base_severity: 0.70, window: Duration::from_secs(15),
                description: "chained exec + network — lateral movement",
            },
            PatternDef {
                sequence: vec![EventType::MemfdCreate, EventType::MmapWX],
                pattern: AttackPattern::FilelessExecution,
                base_severity: 0.85, window: Duration::from_secs(10),
                description: "memfd_create + W^X mmap — preparing fileless payload",
            },
            PatternDef {
                sequence: vec![EventType::Ptrace, EventType::Ptrace, EventType::MmapWX],
                pattern: AttackPattern::PtraceInjection,
                base_severity: 0.90, window: Duration::from_secs(10),
                description: "repeated ptrace + W^X — aggressive injection",
            },
            PatternDef {
                sequence: vec![EventType::Mount, EventType::FileAccess, EventType::Exec],
                pattern: AttackPattern::ContainerEscape,
                base_severity: 0.85, window: Duration::from_secs(15),
                description: "mount + file access + exec — breakout via mount",
            },
        ];

        Self {
            event_buffer: HashMap::new(),
            cross_pid_buffer: VecDeque::with_capacity(1000),
            patterns,
            max_window: Duration::from_secs(30),
        }
    }

    pub fn add_event(&mut self, event: Event) -> Option<AttackChain> {
        let pid = event.pid;

        // Global memory cleanup: remove old events across ALL PIDs
        // This prevents memory leaks for PIDs that have exited
        let cutoff = event.timestamp - self.max_window;
        self.event_buffer.retain(|_, buffer| {
            buffer.retain(|e| e.timestamp >= cutoff);
            !buffer.is_empty()
        });

        // Add to per-PID buffer
        let buffer = self.event_buffer.entry(pid).or_insert_with(VecDeque::new);
        buffer.push_back(event.clone());

        // Add to cross-PID buffer
        self.cross_pid_buffer.push_back(event.clone());
        while self.cross_pid_buffer.len() > 1000 {
            self.cross_pid_buffer.pop_front();
        }

        // Check per-PID patterns
        if let Some(chain) = self.detect_pattern(pid) {
            return Some(chain);
        }

        // Check cross-PID patterns (coordinated attacks)
        self.detect_cross_pid_pattern()
    }

    fn detect_pattern(&self, pid: u32) -> Option<AttackChain> {
        let buffer = self.event_buffer.get(&pid)?;

        for pattern_def in &self.patterns {
            if self.has_sequence_in_window(buffer, &pattern_def.sequence, pattern_def.window) {
                let severity = self.apply_time_decay(buffer, pattern_def.base_severity);
                return Some(AttackChain {
                    events: buffer.iter().cloned().collect(),
                    severity,
                    pattern: pattern_def.pattern.clone(),
                    mitre_id: pattern_def.pattern.mitre_id(),
                    description: pattern_def.description.to_string(),
                });
            }
        }
        None
    }

    fn detect_cross_pid_pattern(&self) -> Option<AttackChain> {
        // Look for coordinated attacks: multiple PIDs doing suspicious things in a short window
        let now = SystemTime::now();
        let window = Duration::from_secs(10);
        let cutoff = now - window;

        let recent: Vec<&Event> = self.cross_pid_buffer.iter()
            .filter(|e| e.timestamp >= cutoff)
            .collect();

        // Count unique PIDs with W^X events
        let wx_pids: std::collections::HashSet<u32> = recent.iter()
            .filter(|e| matches!(e.event_type, EventType::MmapWX | EventType::MprotectWX))
            .map(|e| e.pid)
            .collect();

        if wx_pids.len() >= 3 {
            return Some(AttackChain {
                events: recent.iter().map(|e| (*e).clone()).collect(),
                severity: 0.90,
                pattern: AttackPattern::Unknown,
                mitre_id: "TA0001",
                description: format!("Coordinated W^X attack across {} processes", wx_pids.len()),
            });
        }
        None
    }

    fn has_sequence_in_window(&self, buffer: &VecDeque<Event>, pattern: &[EventType], window: Duration) -> bool {
        if buffer.len() < pattern.len() { return false; }

        let types: Vec<&EventType> = buffer.iter()
            .filter(|e| {
                if let Some(last) = buffer.back() {
                    last.timestamp.duration_since(e.timestamp).unwrap_or(Duration::ZERO) <= window
                } else { false }
            })
            .map(|e| &e.event_type)
            .collect();

        // Subsequence match (not necessarily contiguous)
        let mut pattern_idx = 0;
        for t in &types {
            if pattern_idx < pattern.len() && **t == pattern[pattern_idx] {
                pattern_idx += 1;
            }
        }
        pattern_idx == pattern.len()
    }

    /// Apply time-decay: older events contribute less to severity.
    fn apply_time_decay(&self, buffer: &VecDeque<Event>, base_severity: f64) -> f64 {
        if buffer.len() < 2 { return base_severity; }
        if let (Some(first), Some(last)) = (buffer.front(), buffer.back()) {
            let span = last.timestamp.duration_since(first.timestamp).unwrap_or(Duration::ZERO);
            let decay = 1.0 - (span.as_secs_f64() / 30.0).min(0.3); // max 30% decay over 30s
            return base_severity * decay;
        }
        base_severity
    }

    /// Get active chain count (for metrics).
    pub fn active_chain_count(&self) -> usize {
        self.event_buffer.len()
    }

    /// Clean up state for exited PIDs.
    pub fn remove_pid(&mut self, pid: u32) {
        self.event_buffer.remove(&pid);
    }
}

impl Default for CorrelationEngine { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(pid: u32, et: EventType) -> Event {
        Event { pid, event_type: et, timestamp: SystemTime::now(), uid: 0, comm: "test".into() }
    }

    #[test]
    fn test_shellcode_injection_chain() {
        let mut engine = CorrelationEngine::new();
        let _ = engine.add_event(make_event(100, EventType::MmapWX));
        let result = engine.add_event(make_event(100, EventType::Exec));
        assert!(result.is_some());
        let chain = result.unwrap();
        assert!(matches!(chain.pattern, AttackPattern::ShellcodeInjection));
        assert!(chain.severity > 0.5);
    }

    #[test]
    fn test_container_escape_chain() {
        let mut engine = CorrelationEngine::new();
        let _ = engine.add_event(make_event(200, EventType::Unshare));
        let _ = engine.add_event(make_event(200, EventType::Mount));
        let result = engine.add_event(make_event(200, EventType::Exec));
        assert!(result.is_some());
        assert!(matches!(result.unwrap().pattern, AttackPattern::ContainerEscape));
    }

    #[test]
    fn test_no_false_positive_single_event() {
        let mut engine = CorrelationEngine::new();
        let result = engine.add_event(make_event(300, EventType::Exec));
        assert!(result.is_none());
    }
}
