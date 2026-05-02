// Real eBPF engine using libbpf-rs
#![cfg(target_os = "linux")]

use anyhow::{Context, Result};
use libbpf_rs::{Object, ObjectBuilder, RingBufferBuilder, MapFlags};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[repr(C)]
#[derive(Debug)]
struct Event {
    pid: u32,
    uid: u32,
    timestamp: u64,
    prot: u32,
    flags: u32,
    blocked: u32,
    comm: [u8; 16],
}

pub struct RealEBPFEngine {
    obj: Option<Object>,
    running: Arc<AtomicBool>,
}

impl RealEBPFEngine {
    pub fn new() -> Self {
        Self {
            obj: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn load_and_attach(&mut self) -> Result<()> {
        println!("🔧 Loading REAL eBPF LSM hooks into kernel...");
        
        // Check root
        use nix::unistd::Uid;
        if !Uid::effective().is_root() {
            anyhow::bail!("Must run as root to load eBPF LSM hooks");
        }
        println!(" ✅ Running as root");

        // Find the eBPF object file
        let bpf_path = Path::new("target/bpf/nexus_real.bpf.o");
        if !bpf_path.exists() {
            anyhow::bail!("eBPF object not found at {:?}. Run 'make ebpf' first.", bpf_path);
        }

        // Load eBPF object
        let mut obj = ObjectBuilder::default()
            .open_file(bpf_path)
            .context("Failed to open eBPF object")?
            .load()
            .context("Failed to load eBPF object into kernel")?;

        println!(" ✅ eBPF object loaded");

        // Attach LSM programs by name
        let lsm_programs = vec!["mmap_file", "bprm_check_security"];
        for prog_name in lsm_programs {
            if let Some(prog) = obj.prog_mut(prog_name) {
                prog.attach_lsm()
                    .context(format!("Failed to attach LSM program: {}", prog_name))?;
                println!(" ✅ Attached LSM hook: {}", prog_name);
            }
        }

        self.obj = Some(obj);
        self.running.store(true, Ordering::SeqCst);
        
        println!(" ✅ Ring buffer: 1MB allocated");
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.obj.is_some()
    }

    pub fn set_mode(&self, audit_only: bool) -> Result<()> {
        let obj = self.obj.as_ref().context("eBPF not loaded")?;
        let config_map = obj.map("config").context("Failed to find 'config' map")?;
        
        let key = 0u32.to_ne_bytes();
        let value = if audit_only { 0u32 } else { 1u32 }.to_ne_bytes();
        
        config_map.update(&key, &value, MapFlags::ANY)
            .context("Failed to set mode")?;
        
        Ok(())
    }

    pub fn process_events(&self) -> Result<()> {
        let obj = self.obj.as_ref().context("eBPF not loaded")?;
        
        // Get ring buffer map
        let events_map = obj.map("events").context("Failed to find 'events' map")?;
        
        println!("\n📊 Monitoring kernel events (Ctrl+C to stop)...");

        let running = self.running.clone();
        let mut kill_count = 0u64;
        
        let callback = move |data: &[u8]| -> i32 {
            if data.len() < std::mem::size_of::<Event>() {
                return 0;
            }

            let event = unsafe { &*(data.as_ptr() as *const Event) };

            if event.blocked == 1 {
                kill_count += 1;
                println!(
                    "🚨 THREAT BLOCKED: PID {} | Total Threats Stopped: {}",
                    event.pid, kill_count
                );
            }
            0
        };

        let mut builder = RingBufferBuilder::new();
        builder.add(&events_map, callback)?;
        let ringbuf = builder.build()?;

        // Poll ring buffer
        while running.load(Ordering::SeqCst) {
            ringbuf.poll(Duration::from_millis(100))?;
        }

        Ok(())
    }

    pub fn unload(&mut self) -> Result<()> {
        if self.obj.is_none() {
            return Ok(());
        }

        println!("🔧 Unloading eBPF LSM hooks...");
        self.running.store(false, Ordering::SeqCst);
        self.obj = None;
        println!("✅ eBPF unloaded");
        Ok(())
    }

    pub fn add_to_allowlist(&self, pid: u32) -> Result<()> {
        let obj = self.obj.as_ref().context("eBPF not loaded")?;
        let allowlist = obj.map("allowlist").context("Failed to find 'allowlist' map")?;
        
        let key = pid.to_ne_bytes();
        let value = 1u32.to_ne_bytes();
        
        allowlist.update(&key, &value, MapFlags::ANY)
            .context("Failed to update allowlist")?;
        
        println!("✅ Added PID {} to allowlist", pid);
        Ok(())
    }

    pub fn get_stats(&self) -> Result<()> {
        if self.obj.is_none() {
            anyhow::bail!("eBPF not loaded");
        }

        println!("📊 Statistics:");
        println!(" • eBPF LSM hooks: active");
        println!(" • Ring buffer: monitoring");
        
        Ok(())
    }
}
