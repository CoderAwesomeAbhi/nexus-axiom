use anyhow::{Context, Result};
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use libbpf_rs::RingBufferBuilder;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/nexus_working.skel.rs"));

// Event type constants — must match eBPF
const EVENT_TYPE_MMAP: u8 = 1;
const EVENT_TYPE_MPROTECT: u8 = 4;

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
    ai_analyst: crate::ai_analyst::AIAnalyst,
}

impl EbpfEngine {
    pub fn new(metrics: Arc<crate::metrics::MetricsServer>) -> Result<Self> {
        Ok(Self {
            skel: None,
            metrics,
            ai_analyst: crate::ai_analyst::AIAnalyst::new(None),
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
        let ai_analyst = crate::ai_analyst::AIAnalyst::new(None);

        builder.add(maps.events(), move |data: &[u8]| {
            if data.len() < std::mem::size_of::<Event>() {
                return 0;
            }
            let event = unsafe { &*(data.as_ptr() as *const Event) };

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

            handle_event(event, &ai_analyst);
            0
        })?;

        let ringbuf = builder.build()?;

        log::info!("📡 Monitoring for security events...");

        while running.load(Ordering::SeqCst) {
            ringbuf.poll(Duration::from_millis(100))?;
        }

        Ok(())
    }
}

fn handle_event(event: &Event, ai_analyst: &crate::ai_analyst::AIAnalyst) {
    let comm = String::from_utf8_lossy(&event.comm)
        .trim_end_matches('\0')
        .to_string();

    if event.blocked == 1 {
        let event_label = match event.event_type {
            EVENT_TYPE_MMAP => "W^X mmap",
            EVENT_TYPE_MPROTECT => "W^X mprotect",
            _ => "unknown",
        };

        println!("\n{}", "═".repeat(70));
        println!("🚨 EXPLOIT ATTEMPT BLOCKED 🚨");
        println!("{}", "═".repeat(70));
        println!("  Process   : {} (PID: {})", comm, event.pid);
        println!("  Hook      : {}", event_label);
        println!("  prot=0x{:02x}  flags=0x{:02x}", event.prot, event.flags);
        println!("  Status    : ✅ BLOCKED AT KERNEL LEVEL");

        if let Ok(analysis) = ai_analyst.analyze_threat(
            event.pid,
            &comm,
            "W^X Memory Violation",
        ) {
            println!("\n  🤖 AI Analysis: {}", analysis);
        }

        match kill_process(event.pid) {
            Ok(_) => println!("  Action    : 💀 PROCESS TERMINATED"),
            Err(e) => println!("  Action    : ⚠️  Kill failed: {} (kernel block sufficient)", e),
        }
        println!("{}", "═".repeat(70));
    }
}

fn kill_process(pid: u32) -> Result<()> {
    signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
        .context("Failed to send SIGKILL")
}
