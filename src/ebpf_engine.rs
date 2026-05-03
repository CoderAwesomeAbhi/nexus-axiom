use anyhow::{Context, Result};
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use libbpf_rs::RingBufferBuilder;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/nexus_real.skel.rs"));

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
    skel: Option<NexusRealSkel<'static>>,
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
        let skel_builder = NexusRealSkelBuilder::default();
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
        builder.add(maps.events(), |data: &[u8]| {
            if data.len() < std::mem::size_of::<Event>() {
                return 0;
            }

            let event = unsafe { &*(data.as_ptr() as *const Event) };

            // Increment total events metric
            self.metrics.total_events.fetch_add(1, Ordering::Relaxed);

            self.handle_event(event);
            0
        })?;

        let ringbuf = builder.build()?;

        log::info!("📡 Monitoring for security events...");
        log::info!("🔪 Userspace blocking enabled (works on WSL2!)");

        while running.load(Ordering::SeqCst) {
            ringbuf.poll(Duration::from_millis(100))?;
        }

        Ok(())
    }

    fn handle_event(&self, event: &Event) {
        let comm = String::from_utf8_lossy(&event.comm)
            .trim_end_matches('\0')
            .to_string();

        if event.blocked == 1 {
            self.metrics.blocked_events.fetch_add(1, Ordering::Relaxed);

            println!("\n{}", "═".repeat(70));
            println!("🚨 VULNERABILITY OR EXPLOIT ATTEMPT BLOCKED 🚨");
            println!("{}", "═".repeat(70));
            println!("  Process: {} (PID: {})", comm, event.pid);
            println!("  Reason: Suspicious Memory Allocation (W^X) or File Access");
            println!("  Status: ✅ BLOCKED AT KERNEL LEVEL");

            // Get AI Analysis
            if let Ok(analysis) = self.ai_analyst.analyze_threat(
                event.pid,
                &comm,
                "W^X Memory Violation / Critical File Access",
            ) {
                println!("\n  🤖 AI Threat Analysis:");
                println!("  {}", analysis);
            }

            // The kernel already blocked the operation, but we can also kill the process as an extra measure
            match Self::kill_process(event.pid) {
                Ok(_) => {
                    println!("\n  Action: 💀 PROCESS TERMINATED");
                    println!("{}", "═".repeat(70));
                }
                Err(e) => {
                    println!("\n  Action: ⚠️  Process termination failed: {}", e);
                    println!("  (Process may have already exited or kernel block was sufficient)");
                    println!("{}", "═".repeat(70));
                }
            }
        }
    }

    fn kill_process(pid: u32) -> Result<()> {
        let pid = Pid::from_raw(pid as i32);

        // Send SIGKILL
        signal::kill(pid, Signal::SIGKILL).context("Failed to send SIGKILL")?;

        log::info!("🔪 Sent SIGKILL to PID {}", pid);
        Ok(())
    }
}
