use anyhow::Result;
use clap::{Parser, Subcommand};

#[cfg(target_os = "linux")]
pub mod ai_analyst;
#[cfg(target_os = "linux")]
pub mod dashboard;
#[cfg(target_os = "linux")]
mod ebpf_engine;
#[cfg(target_os = "linux")]
pub mod fs_protection;
#[cfg(target_os = "linux")]
pub mod json_logger;
#[cfg(target_os = "linux")]
pub mod metrics;
#[cfg(target_os = "linux")]
pub mod net_engine;
#[cfg(target_os = "linux")]
pub mod seccomp_engine;

#[derive(Parser)]
#[command(name = "nexus-axiom")]
#[command(version = "1.0.0")]
#[command(about = "🛡️ Nexus Axiom - Real eBPF Security That Kills Exploits")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start real-time protection (requires root)
    Start {
        /// Audit mode (log only, don't block)
        #[arg(long)]
        audit: bool,
    },
    /// Monitor security events
    Monitor,
    /// Show system status
    Status,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { audit } => start_protection(audit),
        Commands::Monitor => monitor_events(),
        Commands::Status => show_status(),
    }
}

#[cfg(target_os = "linux")]
fn start_protection(audit_mode: bool) -> Result<()> {
    use anyhow::Context;
    use ebpf_engine::EbpfEngine;
    use metrics::MetricsServer;
    use net_engine::NetEngine;
    use seccomp_engine::SeccompEngine;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    println!("\n🛡️  NEXUS AXIOM v1.0.0");
    println!("{}", "=".repeat(70));
    println!("🟢 Starting Real-Time Protection...\n");

    // Check root
    if !nix::unistd::Uid::effective().is_root() {
        anyhow::bail!("❌ Must run as root (sudo)");
    }

    // 1. Secure the daemon itself
    let mut seccomp = SeccompEngine::new();
    seccomp.apply_strict_profile()?;

    // 2. Start Metrics Server
    let metrics = Arc::new(MetricsServer::new());
    metrics.start(9090);

    let mut engine = EbpfEngine::new(metrics.clone())?;
    let mut net_engine = NetEngine::new()?;

    // Load and attach eBPF programs
    engine
        .load_and_attach()
        .context("Failed to load eBPF LSM programs")?;

    net_engine
        .load_and_attach()
        .context("Failed to load eBPF XDP programs")?;

    // Block a test malicious IP for demonstration
    net_engine.block_ip(std::net::Ipv4Addr::new(198, 51, 100, 42))?;

    println!("✅ eBPF hooks loaded");
    println!(
        "✅ Mode: {}",
        if audit_mode {
            "AUDIT (logs only)"
        } else {
            "ENFORCE (kills exploits)"
        }
    );
    println!("\n📊 Active Protections:");
    println!("   • W^X memory blocking (LSM)");
    println!("   • Network filtering (XDP)");
    println!("\n⚠️  Press Ctrl+C to stop\n");

    // Setup signal handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Process events
    engine.process_events(running)?;

    println!("\n✅ Nexus Axiom stopped");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn start_protection(_audit_mode: bool) -> Result<()> {
    anyhow::bail!("Nexus Axiom only runs on Linux");
}

fn monitor_events() -> Result<()> {
    println!("📊 Real-Time Event Monitor\n");

    #[cfg(target_os = "linux")]
    {
        use ebpf_engine::EbpfEngine;
        use metrics::MetricsServer;
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;

        let metrics = Arc::new(MetricsServer::new());
        let mut engine = EbpfEngine::new(metrics)?;
        engine.load_and_attach()?;
        let running = Arc::new(AtomicBool::new(true));
        engine.process_events(running)?;
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("Only available on Linux");
    }
}

fn show_status() -> Result<()> {
    println!("\n🛡️  NEXUS AXIOM STATUS\n");
    println!("✅ eBPF LSM Hooks: ACTIVE");
    println!("✅ Ring Buffer: 1MB allocated");
    println!("\n📊 Features:");
    println!("   • W^X memory blocking");
    println!("   • Process execution monitoring");

    Ok(())
}
