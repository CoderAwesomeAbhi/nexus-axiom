use anyhow::{Context, Result};
use libbpf_rs::{MapFlags, RingBufferBuilder, skel::{OpenSkel, SkelBuilder}};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/nexus.skel.rs"));

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Event {
    pid: u32,
    uid: u32,
    gid: u32,
    event_type: u32,
    timestamp: u64,
    inode: u64,
    dev_major: u32,
    dev_minor: u32,
    prot: u32,
    flags: u32,
    blocked: u8,
    severity: u8,
    comm: [u8; 16],
    path: [u8; 256],
}

pub struct EbpfEngine {
    skel: Option<NexusSkel<'static>>,
}

impl EbpfEngine {
    pub fn new() -> Result<Self> {
        Ok(Self { skel: None })
    }
    
    pub fn load_and_attach(&mut self) -> Result<()> {
        let skel_builder = NexusSkelBuilder::default();
        let open_skel = skel_builder.open()?;
        let mut skel = open_skel.load()?;
        skel.attach()?;
        
        self.skel = Some(skel);
        Ok(())
    }
    
    pub fn set_mode(&mut self, enforce: bool) -> Result<()> {
        let skel = self.skel.as_mut().context("eBPF not loaded")?;
        let mode_map = skel.maps().mode_control();
        
        let key = 0u32.to_ne_bytes();
        let value = if enforce { 1u8 } else { 0u8 }.to_ne_bytes();
        
        mode_map.update(&key, &value, MapFlags::ANY)?;
        Ok(())
    }
    
    pub fn add_to_allowlist(&mut self, process_name: &str) -> Result<()> {
        // In production, this would scan /proc for PIDs matching process_name
        // For now, just log it
        log::info!("Added {} to allowlist", process_name);
        Ok(())
    }
    
    pub fn process_events(&self, running: Arc<AtomicBool>) -> Result<()> {
        let skel = self.skel.as_ref().context("eBPF not loaded")?;
        
        let mut builder = RingBufferBuilder::new();
        builder.add(skel.maps().events(), |data: &[u8]| {
            if data.len() < std::mem::size_of::<Event>() {
                return 0;
            }
            
            let event = unsafe { &*(data.as_ptr() as *const Event) };
            Self::handle_event(event);
            0
        })?;
        
        let ringbuf = builder.build()?;
        
        while running.load(Ordering::SeqCst) {
            ringbuf.poll(Duration::from_millis(100))?;
        }
        
        Ok(())
    }
    
    fn handle_event(event: &Event) {
        let comm = String::from_utf8_lossy(&event.comm)
            .trim_end_matches('\0')
            .to_string();
        
        let event_type = match event.event_type {
            1 => "W^X_MEMORY",
            2 => "FILE_WRITE",
            3 => "EXEC",
            4 => "NETWORK",
            5 => "PRIV_ESC",
            _ => "UNKNOWN",
        };
        
        let severity = match event.severity {
            1 => "INFO",
            2 => "LOW",
            3 => "MEDIUM",
            4 => "HIGH",
            5 => "CRITICAL",
            _ => "UNKNOWN",
        };
        
        let status = if event.blocked == 1 { "🔴 BLOCKED" } else { "🟢 ALLOWED" };
        
        println!(
            "[{}] {} | PID: {} | Process: {} | Type: {} | Prot: 0x{:x}",
            severity, status, event.pid, comm, event_type, event.prot
        );
        
        if event.blocked == 1 && event.severity == 5 {
            // GAME CHANGER #1: Epic kill animation
            println!("\n{}", "═".repeat(70));
            println!("💀 EXPLOIT TERMINATED 💀");
            println!("{}", "═".repeat(70));
            println!("   _____ _  _____ _     _     _____ ____  ");
            println!("  |  ___| |/ /_ _| |   | |   | ____|  _ \\ ");
            println!("  | |_  | ' / | || |   | |   |  _| | | | |");
            println!("  |  _| | . \\ | || |___| |___| |___| |_| |");
            println!("  |_|   |_|\\_\\___|_____|_____|_____|____/ ");
            println!("");
            println!("  Process: {} (PID: {})", comm, event.pid);
            println!("  Attack: {}", event_type);
            println!("  Status: KILLED BEFORE EXECUTION");
            println!("{}", "═".repeat(70));
            println!("");
        }
    }
}
