use anyhow::{Context, Result};
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use libbpf_rs::RingBufferBuilder;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::ai_analyst::AIAnalyst;
use crate::json_logger::JsonLogger;

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
}

impl EbpfEngine {
    pub fn new(metrics: Arc<crate::metrics::MetricsServer>) -> Result<Self> {
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
        })
    }

    pub fn load_and_attach(&mut self) -> Result<()> {
        let skel_builder = NexusWorkingSkelBuilder::default();
        let open_skel = skel_builder.open()?;
        let mut skel = open_skel.load()?;
        skel.attach()?;

        self.skel = Some(skel);
        log::info!("✅ eBPF LSM hooks loaded and attached");
        Ok(())
    }

    pub fn process_events(&self, running: Arc<AtomicBool>) -> Result<()> {
        let skel = self.skel.as_ref().context("eBPF not loaded")?;

        let mut builder = RingBufferBuilder::new();
        let maps = skel.maps();

        let metrics = self.metrics.clone();
        let (event_tx, event_rx) = mpsc::channel::<Event>();

        let ai_analyst = self.ai_analyst.clone();
        let json_logger = self.json_logger.clone();

        let worker = thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                handle_event(&event, &ai_analyst, &json_logger);
            }
        });

        let callback_tx = event_tx.clone();

        builder.add(maps.events(), move |data: &[u8]| {
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
                    _ => {}
                }
            }

            let _ = callback_tx.send(event);
            0
        })?;

        let ringbuf = builder.build()?;

        log::info!("📡 Monitoring for security events...");

        while running.load(Ordering::SeqCst) {
            ringbuf.poll(Duration::from_millis(100))?;
        }

        drop(ringbuf);
        drop(event_tx);
        let _ = worker.join();

        Ok(())
    }
}

/// Parse /proc/<pid>/cgroup to find a container name matching the given cgroup_id.
/// Returns the container name (last path component) or "host" if not in a container.
fn resolve_container(pid: u32, cgroup_id: u64) -> String {
    let path = format!("/proc/{}/cgroup", pid);
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return "host".to_string();
    };
    for line in contents.lines() {
        // Each line: <hierarchy>:<controllers>:<cgroup-path>
        let cgroup_path = line.splitn(3, ':').nth(2).unwrap_or("").trim();
        // Derive a numeric id from the path to match against cgroup_id
        let derived_id = cgroup_path
            .split('/')
            .filter_map(|s| u64::from_str_radix(s, 16).ok())
            .next();
        let name = cgroup_path.split('/').last().unwrap_or("").to_string();
        if derived_id == Some(cgroup_id) || (!name.is_empty() && name != "." && cgroup_path != "/") {
            if cgroup_path != "/" && !name.is_empty() {
                return name;
            }
        }
    }
    "host".to_string()
}

fn handle_event(event: &Event, ai_analyst: &Option<AIAnalyst>, json_logger: &Option<JsonLogger>) {
    let comm = String::from_utf8_lossy(&event.comm)
        .trim_end_matches('\0')
        .to_string();

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
        println!("  Container : {} (cgroup: {})", container_name, event.cgroup_id);
        println!("  Hook      : {}", event_label);
        println!("  prot=0x{:02x}  flags=0x{:02x}", event.prot, event.flags);
        println!("  Status    : ✅ BLOCKED AT KERNEL LEVEL");

        // AI Analysis
        if let Some(analyst) = ai_analyst {
            if let Ok(analysis) = analyst.analyze_threat(&comm, event.pid) {
                println!("  AI Analysis: {}", analysis);
            }
        }

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
                details: Some(format!("prot=0x{:02x} flags=0x{:02x}", event.prot, event.flags)),
            };
            logger.log_event(&json_event);
        }

        match kill_process(event.pid) {
            Ok(_) => println!("  Action    : 💀 PROCESS TERMINATED"),
            Err(e) => println!(
                "  Action    : ⚠️  Kill failed: {} (kernel block sufficient)",
                e
            ),
        }
        println!("{}", "═".repeat(70));
    } else if event.event_type == EVENT_TYPE_EXEC {
        // Log exec events (not blocked, just monitored)
        let container_name = resolve_container(event.pid, event.cgroup_id);
        log::info!("📋 Exec: {} (PID: {}) in {}", comm, event.pid, container_name);
    }
}

fn kill_process(pid: u32) -> Result<()> {
    signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL).context("Failed to send SIGKILL")
}

fn resolve_container(pid: u32, cgroup_id: u64) -> String {
    let cgroup_path = format!("/proc/{}/cgroup", pid);
    if let Ok(content) = std::fs::read_to_string(&cgroup_path) {
        for line in content.lines() {
            if line.contains("docker") || line.contains("containerd") || line.contains("crio") {
                if let Some(name) = line.split('/').last() {
                    return name.trim().to_string();
                }
            }
        }
    }
    format!("host (cgroup: {})", cgroup_id)
}
