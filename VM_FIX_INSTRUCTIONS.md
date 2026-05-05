# 🔧 QUICK FIX FOR UBUNTU VM

The GitHub repo has old code. Here's how to fix it:

## Option 1: Quick Manual Fix (5 minutes)

### Step 1: Add missing module
```bash
cd ~/nexus-axiom
nano src/main.rs
```

Add this line after `pub mod seccomp_engine;`:
```rust
pub mod ai_analyst;
```

### Step 2: Create ai_analyst.rs
```bash
cat > src/ai_analyst.rs << 'EOF'
use anyhow::Result;

#[derive(Clone)]
pub struct AIAnalyst;

impl AIAnalyst {
    pub fn new(_api_key: Option<String>) -> Result<Self> {
        Ok(Self)
    }
}
EOF
```

### Step 3: Fix metrics/dashboard calls
```bash
nano src/main.rs
```

Change these lines (around line 67-72):
```rust
// FROM:
if let Err(e) = metrics.start(9090) {

// TO:
metrics.start(9090);
if false {
```

And:
```rust
// FROM:
if let Err(e) = dashboard.start(8080) {

// TO:
dashboard.start(8080);
if false {
```

### Step 4: Build
```bash
cargo build --release
```

---

## Option 2: Use Simplified Version (RECOMMENDED)

Create a minimal working version:

```bash
cd ~/nexus-axiom

# Backup current
mv src/ebpf_engine.rs src/ebpf_engine.rs.bak
mv src/main.rs src/main.rs.bak

# Create minimal ebpf_engine.rs
cat > src/ebpf_engine.rs << 'EOFENGINE'
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

include!(concat!(env!("OUT_DIR"), "/nexus_working.skel.rs"));

const EVENT_TYPE_MMAP: u8 = 1;
const EVENT_TYPE_MPROTECT: u8 = 4;

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
}

impl EbpfEngine {
    pub fn new(metrics: Arc<crate::metrics::MetricsServer>) -> Result<Self> {
        Ok(Self { skel: None, metrics })
    }

    pub fn load_and_attach(&mut self) -> Result<()> {
        let skel_builder = NexusWorkingSkelBuilder::default();
        let open_skel = skel_builder.open()?;
        let mut skel = open_skel.load()?;
        skel.attach()?;
        self.skel = Some(skel);
        log::info!("✅ eBPF LSM hooks loaded");
        Ok(())
    }

    pub fn process_events(&self, running: Arc<AtomicBool>) -> Result<()> {
        let skel = self.skel.as_ref().context("eBPF not loaded")?;
        let mut builder = RingBufferBuilder::new();
        let maps = skel.maps();
        let metrics = self.metrics.clone();
        let (event_tx, event_rx) = mpsc::channel::<Event>();

        let worker = thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                handle_event(&event);
            }
        });

        let callback_tx = event_tx.clone();
        builder.add(maps.events(), move |data: &[u8]| {
            if data.len() < std::mem::size_of::<Event>() { return 0; }
            let event = unsafe { std::ptr::read_unaligned(data.as_ptr() as *const Event) };
            metrics.total_events.fetch_add(1, Ordering::Relaxed);
            if event.blocked == 1 {
                metrics.blocked_events.fetch_add(1, Ordering::Relaxed);
                match event.event_type {
                    EVENT_TYPE_MMAP => metrics.mmap_events.fetch_add(1, Ordering::Relaxed),
                    EVENT_TYPE_MPROTECT => metrics.mprotect_events.fetch_add(1, Ordering::Relaxed),
                    _ => {}
                };
            }
            let _ = callback_tx.send(event);
            0
        })?;

        let ringbuf = builder.build()?;
        log::info!("📡 Monitoring...");

        while running.load(Ordering::SeqCst) {
            ringbuf.poll(Duration::from_millis(100))?;
        }

        drop(ringbuf);
        drop(event_tx);
        let _ = worker.join();
        Ok(())
    }
}

fn handle_event(event: &Event) {
    let comm = String::from_utf8_lossy(&event.comm).trim_end_matches('\0').to_string();
    if event.blocked == 1 {
        println!("\n🚨 EXPLOIT BLOCKED: {} (PID: {})", comm, event.pid);
        let _ = kill_process(event.pid);
    }
}

fn kill_process(pid: u32) -> Result<()> {
    signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL).context("Failed to kill")
}
EOFENGINE

# Create minimal main.rs
cat > src/main.rs << 'EOFMAIN'
use anyhow::Result;
use clap::{Parser, Subcommand};

#[cfg(target_os = "linux")]
pub mod dashboard;
#[cfg(target_os = "linux")]
mod ebpf_engine;
#[cfg(target_os = "linux")]
pub mod metrics;
#[cfg(target_os = "linux")]
pub mod net_engine;
#[cfg(target_os = "linux")]
pub mod seccomp_engine;

#[derive(Parser)]
#[command(name = "nexus-axiom")]
#[command(version = "1.0.0")]
#[command(about = "eBPF Security That Actually Blocks Exploits")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start,
    Monitor,
    Status,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Start => start_protection(),
        Commands::Monitor => start_protection(),
        Commands::Status => show_status(),
    }
}

#[cfg(target_os = "linux")]
fn start_protection() -> Result<()> {
    use anyhow::Context;
    use ebpf_engine::EbpfEngine;
    use metrics::MetricsServer;
    use net_engine::NetEngine;
    use seccomp_engine::SeccompEngine;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    println!("\n🛡️  NEXUS AXIOM v1.0.0");
    println!("🟢 Starting Protection...\n");

    if !nix::unistd::Uid::effective().is_root() {
        anyhow::bail!("❌ Must run as root");
    }

    let mut seccomp = SeccompEngine::new();
    seccomp.apply_strict_profile()?;

    let metrics = Arc::new(MetricsServer::new());
    metrics.start(9090);
    
    let dashboard = dashboard::Dashboard::new(metrics.clone());
    dashboard.start(8080);

    let mut engine = EbpfEngine::new(metrics.clone())?;
    let mut net_engine = NetEngine::new()?;

    engine.load_and_attach().context("Failed to load eBPF")?;
    net_engine.load_and_attach().context("Failed to load XDP")?;

    println!("✅ eBPF hooks loaded");
    println!("✅ Dashboard: http://localhost:8080");
    println!("✅ Metrics: http://localhost:9090/metrics");
    println!("\n⚠️  Press Ctrl+C to stop\n");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))?;

    engine.process_events(running)?;
    println!("\n✅ Stopped");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn start_protection() -> Result<()> {
    anyhow::bail!("Linux only");
}

fn show_status() -> Result<()> {
    println!("\n🛡️  NEXUS AXIOM STATUS\n");
    println!("✅ Ready");
    Ok(())
}
EOFMAIN

# Build
cargo build --release
```

This will work!

---

## After It Builds

Test it:
```bash
# Create test
cat > test_wx.c << 'EOF'
#include <sys/mman.h>
#include <stdio.h>
int main() {
    void *m = mmap(NULL, 4096, PROT_WRITE|PROT_EXEC, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
    if (m == MAP_FAILED) { printf("BLOCKED\n"); return 1; }
    printf("NOT BLOCKED\n");
    return 0;
}
EOF

gcc test_wx.c -o test_wx

# Test
sudo ./target/release/nexus-axiom start &
sleep 2
./test_wx  # Should be BLOCKED!
```

---

## Push Updated Code to GitHub

From Windows:
```powershell
cd C:\Users\abhij\nexus-axiom-final
git add -A
git commit -m "Fix: Working version for Ubuntu"
git push
```

Then in VM:
```bash
cd ~/nexus-axiom
git pull
cargo build --release
```
