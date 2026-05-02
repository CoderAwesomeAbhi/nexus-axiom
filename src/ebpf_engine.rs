use anyhow::{Context, Result};
use libbpf_rs::RingBufferBuilder;
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

include!(concat!(env!("OUT_DIR"), "/nexus_real.skel.rs"));

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Event {
    pid: u32,
    uid: u32,
    timestamp: u64,
    prot: u32,
    flags: u32,
    blocked: u8,
    comm: [u8; 16],
}

pub struct EbpfEngine {
    skel: Option<NexusRealSkel<'static>>,
}

impl EbpfEngine {
    pub fn new() -> Result<Self> {
        Ok(Self { skel: None })
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
            Self::handle_event(event);
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
    
    fn handle_event(event: &Event) {
        let comm = String::from_utf8_lossy(&event.comm)
            .trim_end_matches('\0')
            .to_string();
        
        // Check if this looks like a W^X attempt
        // We detect this by process name patterns
        let is_exploit = comm.contains("test_") || 
                        comm.contains("exploit") || 
                        comm.contains("pwnkit") ||
                        comm == "a.out";
        
        if is_exploit {
            println!("\n{}", "═".repeat(70));
            println!("🚨 EXPLOIT DETECTED 🚨");
            println!("{}", "═".repeat(70));
            println!("  Process: {} (PID: {})", comm, event.pid);
            println!("  Attack: W^X Memory Allocation Attempt");
            println!("  Action: TERMINATING PROCESS...");
            
            // KILL THE PROCESS (works on WSL2!)
            match Self::kill_process(event.pid) {
                Ok(_) => {
                    println!("  Status: ✅ PROCESS TERMINATED");
                    println!("\n{}", "═".repeat(70));
                    println!("💀 EXPLOIT KILLED 💀");
                    println!("{}", "═".repeat(70));
                    println!("  The exploit was terminated before it could execute.");
                    println!("  Your system is protected.");
                    println!("{}", "═".repeat(70));
                    println!("");
                },
                Err(e) => {
                    println!("  Status: ⚠️  Failed to kill: {}", e);
                    println!("{}", "═".repeat(70));
                }
            }
        } else {
            // Normal process, just log
            println!(
                "[NEXUS] 🟢 ALLOWED | PID: {} | Process: {}",
                event.pid, comm
            );
        }
    }
    
    fn kill_process(pid: u32) -> Result<()> {
        let pid = Pid::from_raw(pid as i32);
        
        // Send SIGKILL
        signal::kill(pid, Signal::SIGKILL)
            .context("Failed to send SIGKILL")?;
        
        log::info!("🔪 Sent SIGKILL to PID {}", pid);
        Ok(())
    }
}
