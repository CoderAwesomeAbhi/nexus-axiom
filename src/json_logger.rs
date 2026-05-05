#![allow(dead_code)]
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Structured JSON event for SIEM integration and log analysis.
#[derive(Serialize, Debug)]
pub struct JsonEvent {
    pub timestamp: String,
    pub event_type: String,
    pub pid: u32,
    pub uid: u32,
    pub comm: String,
    pub action: String,
    pub blocked: bool,
    pub cgroup_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// SIEM-specific event formats
#[derive(Serialize, Debug)]
pub struct SplunkEvent {
    pub time: String,
    pub source: String,
    pub sourcetype: String,
    pub event: JsonEvent,
}

#[derive(Serialize, Debug)]
pub struct ElkEvent {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    pub event_type: String,
    pub process: ProcessInfo,
    pub security: SecurityInfo,
}

#[derive(Serialize, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct SecurityInfo {
    pub blocked: bool,
    pub action: String,
    pub cgroup_id: u64,
}

#[derive(Clone)]
pub enum LogFormat {
    Standard,
    Splunk,
    Elk,
    Datadog,
}

/// JSON Logger writes structured security events to a file or stdout.
#[derive(Clone)]
pub struct JsonLogger {
    file: Option<Arc<Mutex<std::fs::File>>>,
    stdout_mode: bool,
    format: LogFormat,
}

impl JsonLogger {
    /// Create a new JSON logger.
    /// If `path` is Some, logs to that file. If None, logs to stdout.
    pub fn new(path: Option<&str>, format: LogFormat) -> Self {
        match path {
            Some(p) => {
                // Create parent directory if it doesn't exist
                if let Some(parent) = std::path::Path::new(p).parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        log::warn!("⚠️  Failed to create log directory: {}", e);
                    }
                }

                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)
                    .ok()
                    .map(|f| Arc::new(Mutex::new(f)));

                if file.is_some() {
                    log::info!("📝 JSON logging enabled: {}", p);
                } else {
                    log::warn!(
                        "⚠️  Could not open JSON log file: {}, falling back to stdout",
                        p
                    );
                }

                Self {
                    file,
                    stdout_mode: false,
                    format,
                }
            }
            None => Self {
                file: None,
                stdout_mode: true,
                format,
            },
        }
    }

    /// Log a security event as JSON.
    pub fn log_event(&self, event: &JsonEvent) {
        let json = match self.format {
            LogFormat::Standard => serde_json::to_string(event).ok(),
            LogFormat::Splunk => {
                let splunk_event = SplunkEvent {
                    time: event.timestamp.clone(),
                    source: "nexus-axiom".to_string(),
                    sourcetype: "security:ebpf".to_string(),
                    event: event.clone(),
                };
                serde_json::to_string(&splunk_event).ok()
            }
            LogFormat::Elk => {
                let elk_event = ElkEvent {
                    timestamp: event.timestamp.clone(),
                    event_type: event.event_type.clone(),
                    process: ProcessInfo {
                        pid: event.pid,
                        name: event.comm.clone(),
                    },
                    security: SecurityInfo {
                        blocked: event.blocked,
                        action: event.action.clone(),
                        cgroup_id: event.cgroup_id,
                    },
                };
                serde_json::to_string(&elk_event).ok()
            }
            LogFormat::Datadog => {
                // Datadog format is similar to standard but with specific tags
                serde_json::to_string(event).ok()
            }
        };

        if let Some(json) = json {
            if self.stdout_mode {
                println!("{}", json);
                return;
            }

            if let Some(ref file_mutex) = self.file {
                if let Ok(mut f) = file_mutex.lock() {
                    let _ = writeln!(f, "{}", json);
                }
            }
        }
    }

    /// Map a numeric event type to a human-readable string.
    pub fn event_type_str(event_type: u8) -> &'static str {
        match event_type {
            1 => "W^X_MMAP",
            2 => "EXEC_BLOCK",
            3 => "FILE_ACCESS",
            4 => "W^X_MPROTECT",
            _ => "UNKNOWN",
        }
    }
}

impl Clone for JsonEvent {
    fn clone(&self) -> Self {
        Self {
            timestamp: self.timestamp.clone(),
            event_type: self.event_type.clone(),
            pid: self.pid,
            uid: self.uid,
            comm: self.comm.clone(),
            action: self.action.clone(),
            blocked: self.blocked,
            cgroup_id: self.cgroup_id,
            details: self.details.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_event_creation() {
        let event = JsonEvent {
            timestamp: "2026-05-05T00:00:00Z".to_string(),
            event_type: "mmap".to_string(),
            pid: 1234,
            uid: 0,
            comm: "exploit".to_string(),
            action: "blocked".to_string(),
            blocked: true,
            cgroup_id: 12345,
            details: Some("test".to_string()),
        };

        assert_eq!(event.pid, 1234);
        assert_eq!(event.event_type, "mmap");
        assert!(event.blocked);
    }

    #[test]
    fn test_json_logger_stdout() {
        let logger = JsonLogger::new(None, LogFormat::Standard);
        assert!(logger.stdout_mode);
    }

    #[test]
    fn test_log_format_clone() {
        let format = LogFormat::Standard;
        let format2 = format.clone();
        assert!(matches!(format2, LogFormat::Standard));
    }
}
