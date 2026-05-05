use anyhow::Result;
use clap::{Parser, Subcommand};

#[cfg(target_os = "linux")]
pub mod ai_analyst;
#[cfg(target_os = "linux")]
pub mod config;
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
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "🛡️ Nexus Axiom - eBPF Security That Actually Blocks Exploits")]
#[command(arg_required_else_help = false)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start real-time protection (requires root). Loads eBPF hooks and monitors for exploits.
    Start {
        /// Audit mode: log security events without blocking or killing processes
        #[arg(long)]
        audit: bool,
    },
    /// Monitor security events in real-time without starting full protection
    Monitor,
    /// Show system status and active protections
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
fn start_protection(audit: bool) -> Result<()> {
    use anyhow::Context;
    use ebpf_engine::EbpfEngine;
    use fs_protection::FsProtection;
    use metrics::MetricsServer;
    use net_engine::NetEngine;
    use seccomp_engine::SeccompEngine;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    println!("\n🛡️  NEXUS AXIOM v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", "=".repeat(70));
    println!("🟢 Starting Real-Time Protection...\n");

    // Load config
    let config = config::Config::load().unwrap_or_else(|e| {
        log::warn!("⚠️  Failed to load config.toml: {}", e);
        log::warn!("   Using default configuration...");
        config::Config::default()
    });

    // Override audit mode if specified in CLI
    let audit_mode = if audit {
        true
    } else {
        config.security.mode == "audit" || !config.security.kill_on_violation
    };

    if audit_mode {
        println!("📋 Running in AUDIT MODE (logging only, no blocking)\n");
    }

    // Check root
    if !nix::unistd::Uid::effective().is_root() {
        anyhow::bail!("❌ Must run as root (sudo)");
    }

    // 1. Initialize Filesystem Protection
    let mut fs_protection = FsProtection::new();
    if let Err(e) = fs_protection.start_monitoring() {
        log::warn!("⚠️  FS Protection failed to start: {}", e);
        log::warn!("   Continuing without real-time file monitoring...");
    } else {
        log::info!("🛡️  Filesystem protection: Real-time monitoring active");
    }

    // 2. Start Metrics Server
    let metrics = Arc::new(MetricsServer::new());
    if let Err(e) = metrics.start(config.server.metrics_port) {
        log::warn!("⚠️  Metrics server failed to start: {}", e);
        log::warn!("   Continuing without metrics endpoint...");
    }

    // 3. Start Dashboard
    let dashboard = dashboard::Dashboard::new(metrics.clone());
    if let Err(e) = dashboard.start(config.server.dashboard_port) {
        log::warn!("⚠️  Dashboard failed to start: {}", e);
        log::warn!("   Continuing without dashboard...");
    }

    let mut engine = EbpfEngine::new(metrics.clone(), audit_mode)?;
    let mut net_engine = NetEngine::new()?;

    // Load and attach eBPF programs
    engine
        .load_and_attach()
        .context("Failed to load eBPF LSM programs")?;

    net_engine
        .load_and_attach()
        .context("Failed to load eBPF XDP programs")?;

    // Apply network blocks from config
    for ip_str in &config.network.blocked_ips {
        if let Ok(ip) = ip_str.parse() {
            if let Err(e) = net_engine.block_ip(ip) {
                log::warn!("Failed to block IP {}: {}", ip_str, e);
            } else {
                log::info!("🚫 Blocked IP from config: {}", ip_str);
            }
        } else {
            log::warn!("Invalid IP in config: {}", ip_str);
        }
    }

    // Apply port blocks from config
    for port in &config.network.blocked_ports {
        if let Err(e) = net_engine.block_port(*port) {
            log::warn!("Failed to block port {}: {}", port, e);
        } else {
            log::info!("🚫 Blocked port from config: {}", port);
        }
    }

    println!("✅ eBPF hooks loaded");

    // 4. Apply seccomp LAST (after all servers/threads are running)
    let mut seccomp = SeccompEngine::new();
    if let Err(e) = seccomp.apply_strict_profile() {
        log::warn!("⚠️  Seccomp failed to apply: {}", e);
        log::warn!("   Continuing without seccomp isolation...");
    }

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
    println!("   • Filesystem protection");
    println!("   • AI threat analysis");
    println!("   • JSON event logging");
    println!("\n⚠️  Press Ctrl+C to stop\n");

    // Setup signal handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Process events
    engine.process_events(running, &mut fs_protection)?;

    println!("\n✅ Nexus Axiom stopped");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn start_protection(_audit: bool) -> Result<()> {
    anyhow::bail!("Nexus Axiom only runs on Linux");
}

fn monitor_events() -> Result<()> {
    println!("📊 Real-Time Event Monitor\n");

    #[cfg(target_os = "linux")]
    {
        use ebpf_engine::EbpfEngine;
        use fs_protection::FsProtection;
        use metrics::MetricsServer;
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;

        let metrics = Arc::new(MetricsServer::new());
        let mut engine = EbpfEngine::new(metrics, false)?;
        engine.load_and_attach()?;
        let running = Arc::new(AtomicBool::new(true));
        let mut fs_protection = FsProtection::new();
        engine.process_events(running, &mut fs_protection)?;
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("Only available on Linux");
    }
}

fn show_status() -> Result<()> {
    println!("\n🛡️  NEXUS AXIOM STATUS\n");

    // Check if actually running via systemd
    let status_output = std::process::Command::new("systemctl")
        .args(&["is-active", "nexus-axiom"])
        .output();

    match status_output {
        Ok(output) if output.status.success() => {
            println!("✅ Service: RUNNING");
            println!("✅ eBPF LSM Hooks: ACTIVE (assumed if service running)");
        }
        _ => {
            println!("❌ Service: NOT RUNNING");
            println!("   Start with: sudo systemctl start nexus-axiom");
            return Ok(());
        }
    }

    println!("\n📊 Features:");
    println!("   • W^X memory blocking");
    println!("   • Process execution monitoring");
    println!("   • Network filtering (XDP)");
    println!("   • File system protection");
    println!("\n💡 View logs: sudo journalctl -u nexus-axiom -f");

    Ok(())
}
