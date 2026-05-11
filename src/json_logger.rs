#![allow(dead_code)]
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ml_confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitre_technique: Option<String>,
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
    Ocsf,
    Cef,
}

/// JSON Logger writes structured security events to a file or stdout.
/// Supports log rotation by size.
#[derive(Clone)]
pub struct JsonLogger {
    file: Option<Arc<Mutex<std::fs::File>>>,
    file_path: Option<PathBuf>,
    stdout_mode: bool,
    format: LogFormat,
    max_size_bytes: u64,
    max_files: usize,
    bytes_written: Arc<Mutex<u64>>,
}

impl JsonLogger {
    pub fn new(path: Option<&str>, format: LogFormat) -> Self {
        Self::with_rotation(path, format, 100 * 1024 * 1024, 10) // 100MB, 10 files
    }

    pub fn with_rotation(
        path: Option<&str>,
        format: LogFormat,
        max_size_bytes: u64,
        max_files: usize,
    ) -> Self {
        match path {
            Some(p) => {
                if let Some(parent) = Path::new(p).parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        log::warn!("⚠️  Failed to create log directory: {}", e);
                    }
                }

                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)
                    .ok()
                    .map(|f| Arc::new(Mutex::new(f)));

                let current_size = fs::metadata(p).map(|m| m.len()).unwrap_or(0);

                if file.is_some() {
                    log::info!(
                        "📝 JSON logging enabled: {} (rotation: {}MB × {})",
                        p,
                        max_size_bytes / (1024 * 1024),
                        max_files
                    );
                } else {
                    log::warn!(
                        "⚠️  Could not open JSON log file: {}, falling back to stdout",
                        p
                    );
                }

                Self {
                    file,
                    file_path: Some(PathBuf::from(p)),
                    stdout_mode: false,
                    format,
                    max_size_bytes,
                    max_files,
                    bytes_written: Arc::new(Mutex::new(current_size)),
                }
            }
            None => Self {
                file: None,
                file_path: None,
                stdout_mode: true,
                format,
                max_size_bytes,
                max_files,
                bytes_written: Arc::new(Mutex::new(0)),
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
            LogFormat::Datadog => serde_json::to_string(event).ok(),
            LogFormat::Ocsf => Some(Self::format_ocsf(event)),
            LogFormat::Cef => Some(Self::format_cef(event)),
        };

        if let Some(json) = json {
            if self.stdout_mode {
                println!("{}", json);
                return;
            }

            if let Some(ref file_mutex) = self.file {
                // Check if rotation needed
                self.maybe_rotate();

                if let Ok(mut f) = file_mutex.lock() {
                    let bytes = json.len() as u64 + 1; // +1 for newline
                    let _ = writeln!(f, "{}", json);
                    if let Ok(mut written) = self.bytes_written.lock() {
                        *written += bytes;
                    }
                }
            }
        }
    }

    fn maybe_rotate(&self) {
        let should_rotate = self
            .bytes_written
            .lock()
            .map(|w| *w >= self.max_size_bytes)
            .unwrap_or(false);

        if !should_rotate {
            return;
        }

        if let Some(ref path) = self.file_path {
            // Rotate: .1 → .2, .0 → .1, current → .0
            for i in (1..self.max_files).rev() {
                let old = path.with_extension(format!("json.{}", i - 1));
                let new = path.with_extension(format!("json.{}", i));
                if old.exists() {
                    let _ = fs::rename(&old, &new);
                }
            }

            let rotated = path.with_extension("json.0");
            if let Some(ref file_mutex) = self.file {
                if let Ok(f) = file_mutex.lock() {
                    drop(f); // Release lock before rename
                }
            }
            let _ = fs::rename(path, &rotated);

            // Re-create the file
            if let Ok(new_file) = OpenOptions::new().create(true).append(true).open(path) {
                if let Some(ref file_mutex) = self.file {
                    if let Ok(mut f) = file_mutex.lock() {
                        *f = new_file;
                    }
                }
            }

            if let Ok(mut written) = self.bytes_written.lock() {
                *written = 0;
            }

            // Delete old rotated files beyond max_files
            let oldest = path.with_extension(format!("json.{}", self.max_files));
            if oldest.exists() {
                let _ = fs::remove_file(&oldest);
            }

            log::info!("📝 Log rotated: {}", path.display());
        }
    }

    /// Format as OCSF (Open Cybersecurity Schema Framework)
    fn format_ocsf(event: &JsonEvent) -> String {
        let ocsf = serde_json::json!({
            "class_uid": 2001,
            "category_uid": 2,
            "severity_id": if event.blocked { 4 } else { 2 },
            "type_uid": 200101,
            "time": event.timestamp,
            "message": format!("{}: {}", event.event_type, event.action),
            "finding_info": {
                "title": format!("{} from {}", event.event_type, event.comm),
                "uid": event.correlation_id,
            },
            "process": { "pid": event.pid, "name": event.comm, "uid": event.uid },
            "metadata": {
                "product": { "name": "Nexus Axiom", "vendor_name": "Nexus Axiom" },
                "version": "1.1.0"
            }
        });
        serde_json::to_string(&ocsf).unwrap_or_default()
    }

    /// Format as CEF (Common Event Format)
    fn format_cef(event: &JsonEvent) -> String {
        let severity = if event.blocked { "8" } else { "3" };
        format!(
            "CEF:0|NexusAxiom|Security|1.0|{}|{}|{}|pid={} uid={} comm={} blocked={} cgroup={}",
            event.event_type,
            event.action,
            severity,
            event.pid,
            event.uid,
            event.comm,
            event.blocked,
            event.cgroup_id
        )
    }

    /// Map a numeric event type to a human-readable string.
    pub fn event_type_str(event_type: u8) -> &'static str {
        match event_type {
            1 => "W^X_MMAP",
            2 => "EXEC_BLOCK",
            3 => "FILE_ACCESS",
            4 => "W^X_MPROTECT",
            5 => "PTRACE",
            6 => "EXEC",
            7 => "ROP_CHAIN",
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
            correlation_id: self.correlation_id.clone(),
            ml_confidence: self.ml_confidence,
            mitre_technique: self.mitre_technique.clone(),
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
            correlation_id: None,
            ml_confidence: Some(0.92),
            mitre_technique: Some("T1055".into()),
        };
        assert_eq!(event.pid, 1234);
        assert!(event.blocked);
    }

    #[test]
    fn test_json_logger_stdout() {
        let logger = JsonLogger::new(None, LogFormat::Standard);
        assert!(logger.stdout_mode);
    }

    #[test]
    fn test_cef_format() {
        let event = JsonEvent {
            timestamp: "2026-01-01T00:00:00Z".into(),
            event_type: "W^X_MMAP".into(),
            pid: 42,
            uid: 0,
            comm: "test".into(),
            action: "blocked".into(),
            blocked: true,
            cgroup_id: 1,
            details: None,
            correlation_id: None,
            ml_confidence: None,
            mitre_technique: None,
        };
        let cef = JsonLogger::format_cef(&event);
        assert!(cef.starts_with("CEF:0|NexusAxiom"));
        assert!(cef.contains("pid=42"));
    }

    #[test]
    fn test_event_type_str() {
        assert_eq!(JsonLogger::event_type_str(1), "W^X_MMAP");
        assert_eq!(JsonLogger::event_type_str(7), "ROP_CHAIN");
        assert_eq!(JsonLogger::event_type_str(255), "UNKNOWN");
    }
}
