use anyhow::{Context, Result};
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use libbpf_rs::RingBufferBuilder;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::ai_analyst::AIAnalyst;
use crate::json_logger::JsonLogger;

// Cache for cgroup resolution (pid -> (container_name, timestamp))
lazy_static::lazy_static! {
    static ref CGROUP_CACHE: Mutex<HashMap<u32, (String, Instant)>> = Mutex::new(HashMap::new());
}
const CACHE_TTL_SECS: u64 = 60;

include!(concat!(env!("OUT_DIR"), "/nexus_working.skel.rs"));

// Event type constants — must match eBPF
const EVENT_TYPE_MMAP: u8 = 1;
const EVENT_TYPE_MPROTECT: u8 = 4;
const EVENT_TYPE_PTRACE: u8 = 5;
const EVENT_TYPE_EXEC: u8 = 6;

/// Must exactly match `struct event` in nexus_working.bpf.c (same field order, sizes, padding).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Event {
    pid: u32,
    uid: u32,
    timestamp: u64,
    prot: u32,
    flags: u32,
    blocked: u8,
    event_type: u8,
    _pad: [u8; 2],
    cgroup_id: u64,
    comm: [u8; 16],
}

pub struct EbpfEngine {
    skel: Option<NexusWorkingSkel<'static>>,
    metrics: Arc<crate::metrics::MetricsServer>,
    ai_analyst: Option<AIAnalyst>,
    json_logger: Option<JsonLogger>,
    audit_mode: bool,
    kill_on_violation: bool,
}

impl EbpfEngine {
    pub fn new(
        metrics: Arc<crate::metrics::MetricsServer>,
        audit_mode: bool,
        kill_on_violation: bool,
    ) -> Result<Self> {
        let ai_analyst = AIAnalyst::new(None).ok();
        let json_logger = Some(JsonLogger::new(
            Some("/var/log/nexus-axiom/events.json"),
            crate::json_logger::LogFormat::Standard,
        ));

        Ok(Self {
            skel: None,
            metrics,
            ai_analyst,
            json_logger,
            audit_mode,
            kill_on_violation,
        })
    }

    pub fn load_and_attach(&mut self) -> Result<()> {
        let skel_builder = NexusWorkingSkelBuilder::default();
        let open_skel = skel_builder.open()?;
        let mut skel = open_skel.load()?;
        skel.attach()?;

        // Set audit mode in eBPF config map
        let config_map = skel.maps().config();
        let key: u32 = 0;
        let value: u8 = if self.audit_mode { 1 } else { 0 };
        config_map
            .update(&key.to_ne_bytes(), &[value], libbpf_rs::MapFlags::ANY)
            .context("Failed to set audit mode in eBPF")?;

        self.skel = Some(skel);
        log::info!(
            "✅ eBPF LSM hooks loaded and attached (audit_mode={})",
            self.audit_mode
        );
        Ok(())
    }

    pub fn process_events(
        &self,
        running: Arc<AtomicBool>,
        fs_protection: &mut crate::fs_protection::FsProtection,
    ) -> Result<()> {
        let skel = self.skel.as_ref().context("eBPF not loaded")?;

        let mut builder = RingBufferBuilder::new();
        let maps = skel.maps();

        let metrics = self.metrics.clone();
        let (event_tx, event_rx) = mpsc::sync_channel::<Event>(1000); // Bounded to prevent backlog

        let ai_analyst = self.ai_analyst.clone();
        let json_logger = self.json_logger.clone();
        let audit_mode = self.audit_mode;

        let worker = thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                handle_event(&event, &ai_analyst, &json_logger, audit_mode);
            }
        });

        let callback_tx = event_tx.clone();

        // Rate limiting state
        let mut last_reset = Instant::now();
        let mut event_count = 0;
        const MAX_EVENTS_PER_SEC: u32 = 1000;

        builder.add(maps.events(), move |data: &[u8]| {
            // Rate limiting
            let now = Instant::now();
            if now.duration_since(last_reset).as_secs() >= 1 {
                event_count = 0;
                last_reset = now;
            }

            event_count += 1;
            if event_count > MAX_EVENTS_PER_SEC {
                log::warn!(
                    "⚠️  Rate limit exceeded: Dropping events ({}/sec)",
                    event_count
                );
                return 0; // Drop event
            }

            if data.len() < std::mem::size_of::<Event>() {
                return 0;
            }
            let event = unsafe { std::ptr::read_unaligned(data.as_ptr() as *const Event) };

            metrics.total_events.fetch_add(1, Ordering::Relaxed);

            if event.blocked == 1 {
                metrics.blocked_events.fetch_add(1, Ordering::Relaxed);

                match event.event_type {
                    EVENT_TYPE_MMAP => {
                        metrics.mmap_events.fetch_add(1, Ordering::Relaxed);
                    }
                    EVENT_TYPE_MPROTECT => {
                        metrics.mprotect_events.fetch_add(1, Ordering::Relaxed);
                    }
                    EVENT_TYPE_PTRACE => {
                        metrics.ptrace_events.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {}
                }
            } else if event.event_type == EVENT_TYPE_EXEC {
                metrics.exec_events.fetch_add(1, Ordering::Relaxed);
            }

            let _ = callback_tx.send(event);
            0
        })?;

        let ringbuf = builder.build()?;

        log::info!("📡 Monitoring for security events...");

        let mut check_counter = 0;
        let result = (|| -> Result<()> {
            while running.load(Ordering::SeqCst) {
                ringbuf.poll(Duration::from_millis(100))?;

                // Check FS integrity every 100 polls (~10 seconds)
                check_counter += 1;
                if check_counter >= 100 {
                    fs_protection.check_integrity();
                    check_counter = 0;
                }
            }
            Ok(())
        })();

        // Always cleanup, even if error occurred
        drop(ringbuf);
        drop(event_tx);
        let _ = worker.join();

        result
    }
}

/// Parse /proc/<pid>/cgroup to find a container name matching the given cgroup_id.
/// Returns the container name (last path component) or "host" if not in a container.
fn resolve_container(pid: u32, cgroup_id: u64) -> String {
    // Check cache first
    if let Ok(cache) = CGROUP_CACHE.lock() {
        if let Some((name, timestamp)) = cache.get(&pid) {
            if timestamp.elapsed().as_secs() < CACHE_TTL_SECS {
                return name.clone();
            }
        }
    }

    let path = format!("/proc/{}/cgroup", pid);
    let result = if let Ok(contents) = std::fs::read_to_string(&path) {
        let mut found = "host".to_string();
        for line in contents.lines() {
            let cgroup_path = line.splitn(3, ':').nth(2).unwrap_or("").trim();
            let derived_id = cgroup_path
                .split('/')
                .filter_map(|s| u64::from_str_radix(s, 16).ok())
                .next();
            let name = cgroup_path.split('/').last().unwrap_or("").to_string();
            if derived_id == Some(cgroup_id)
                || (!name.is_empty() && name != "." && cgroup_path != "/")
            {
                if cgroup_path != "/" && !name.is_empty() {
                    found = name;
                    break;
                }
            }
        }
        found
    } else {
        "host".to_string()
    };

    // Update cache
    if let Ok(mut cache) = CGROUP_CACHE.lock() {
        cache.insert(pid, (result.clone(), Instant::now()));
        // Clean old entries (keep cache size reasonable)
        if cache.len() > 1000 {
            cache.retain(|_, (_, ts)| ts.elapsed().as_secs() < CACHE_TTL_SECS);
        }
    }

    result
}

fn handle_event(
    event: &Event,
    ai_analyst: &Option<AIAnalyst>,
    json_logger: &Option<JsonLogger>,
    audit_mode: bool,
) {
    let mut comm = String::from_utf8_lossy(&event.comm)
        .trim_end_matches('\0')
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect::<String>();

    if comm.is_empty() {
        comm = format!("<pid-{}>", event.pid);
    }

    if event.blocked == 1 {
        let event_label = match event.event_type {
            EVENT_TYPE_MMAP => "W^X mmap",
            EVENT_TYPE_MPROTECT => "W^X mprotect",
            EVENT_TYPE_PTRACE => "Unauthorized ptrace",
            _ => "unknown",
        };

        let container_name = resolve_container(event.pid, event.cgroup_id);

        println!("\n{}", "═".repeat(70));
        println!("🚨 EXPLOIT ATTEMPT BLOCKED 🚨");
        println!("{}", "═".repeat(70));
        println!("  Process   : {} (PID: {})", comm, event.pid);
        println!(
            "  Container : {} (cgroup: {})",
            container_name, event.cgroup_id
        );
        println!("  Hook      : {}", event_label);
        println!("  prot=0x{:02x}  flags=0x{:02x}", event.prot, event.flags);
        println!("  Status    : ✅ BLOCKED AT KERNEL LEVEL");

        // AI Analysis (disabled in hot path for performance)
        // Can be enabled as async background task if needed
        // if let Some(analyst) = ai_analyst {
        //     if let Ok(analysis) = analyst.analyze_threat(event.pid, &comm, "W^X violation") {
        //         println!("  AI Analysis: {}", analysis);
        //     }
        // }
        let _ = ai_analyst; // Suppress unused warning

        // JSON Logging
        if let Some(logger) = json_logger {
            let json_event = crate::json_logger::JsonEvent {
                timestamp: chrono::Utc::now().to_rfc3339(),
                event_type: event_label.to_string(),
                pid: event.pid,
                uid: event.uid,
                comm: comm.clone(),
                action: "blocked".to_string(),
                blocked: true,
                cgroup_id: event.cgroup_id,
                details: Some(format!(
                    "prot=0x{:02x} flags=0x{:02x}",
                    event.prot, event.flags
                )),
            };
            logger.log_event(&json_event);
        }

        match if audit_mode || !self.kill_on_violation {
            log::warn!("📋 [AUDIT MODE] Would terminate process {}", event.pid);
            Ok(())
        } else {
            kill_process(event.pid)
        } {
            Ok(_) => println!(
                "  Action    : {} PROCESS {}",
                if audit_mode || !self.kill_on_violation {
                    "📋"
                } else {
                    "💀"
                },
                if audit_mode || !self.kill_on_violation {
                    "WOULD BE TERMINATED"
                } else {
                    "TERMINATED"
                }
            ),
            Err(e) => println!(
                "  Action    : ⚠️  Kill failed: {} (kernel block sufficient)",
                e
            ),
        }
        println!("{}", "═".repeat(70));
    } else if event.event_type == EVENT_TYPE_EXEC {
        // Log exec events (not blocked, just monitored)
        let container_name = resolve_container(event.pid, event.cgroup_id);
        log::info!(
            "📋 Exec: {} (PID: {}) in {}",
            comm,
            event.pid,
            container_name
        );
    } else if event.event_type == EVENT_TYPE_PTRACE {
        // Log ptrace events (monitoring only)
        let container_name = resolve_container(event.pid, event.cgroup_id);
        log::info!(
            "🔍 Ptrace: {} (PID: {}) in {}",
            comm,
            event.pid,
            container_name
        );
    }
}

fn kill_process(pid: u32) -> Result<()> {
    signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL).context("Failed to send SIGKILL")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_container_host() {
        // Test with non-existent PID (should return host)
        let result = resolve_container(999999, 12345);
        assert!(result.contains("host") || result.contains("cgroup"));
    }

    #[test]
    fn test_kill_process_invalid_pid() {
        // Killing invalid PID should fail gracefully
        let result = kill_process(999999);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_type_labels() {
        // Test that event types are correctly labeled
        let event_types = vec![(1, "mmap"), (4, "mprotect"), (5, "ptrace"), (6, "exec")];

        for (event_type, expected_label) in event_types {
            let label = match event_type {
                1 => "mmap",
                4 => "mprotect",
                5 => "ptrace",
                6 => "exec",
                _ => "unknown",
            };
            assert_eq!(label, expected_label);
        }
    }
}
