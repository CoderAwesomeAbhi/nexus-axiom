use anyhow::{Context, Result};
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use libbpf_rs::MapFlags;
use log;
use std::net::Ipv4Addr;

include!(concat!(env!("OUT_DIR"), "/nexus_net.skel.rs"));

pub struct NetEngine {
    skel: Option<NexusNetSkel<'static>>,
}

impl Default for NetEngine {
    fn default() -> Self {
        Self::new().expect("NetEngine::default failed")
    }
}

impl NetEngine {
    pub fn new() -> Result<Self> {
        Ok(Self { skel: None })
    }

    pub fn load_and_attach(&mut self) -> Result<()> {
        let skel_builder = NexusNetSkelBuilder::default();
        let open_skel = skel_builder.open()?;
        let mut skel = open_skel.load()?;
        skel.attach()?;

        self.skel = Some(skel);
        log::info!("✅ eBPF XDP Network Defense loaded and attached to all interfaces");
        log::info!("   XDP filtering active at line-rate (10Gbps+ capable)");
        Ok(())
    }

    /// Block an IPv4 address
    pub fn block_ip(&self, ip: Ipv4Addr) -> Result<()> {
        let skel = self.skel.as_ref().context("XDP not loaded")?;
        let maps = skel.maps();
        let blocklist = maps.blocklist_ipv4();

        let ip_bytes = u32::from(ip).to_be_bytes();
        let val: u8 = 1;

        blocklist.update(&ip_bytes, &val.to_ne_bytes(), MapFlags::ANY)?;
        log::info!("🚫 Blocked IP: {}", ip);
        Ok(())
    }

    /// Block a TCP/UDP port
    pub fn block_port(&self, port: u16) -> Result<()> {
        let skel = self.skel.as_ref().context("XDP not loaded")?;
        let maps = skel.maps();
        let blocked_ports = maps.blocked_ports();

        let port_bytes = port.to_ne_bytes();
        let val: u8 = 1;

        blocked_ports.update(&port_bytes, &val.to_ne_bytes(), MapFlags::ANY)?;
        log::info!("🚫 Blocked port: {}", port);
        Ok(())
    }

    /// Unblock a TCP/UDP port
    pub fn unblock_port(&self, port: u16) -> Result<()> {
        let skel = self.skel.as_ref().context("XDP not loaded")?;
        let maps = skel.maps();
        let blocked_ports = maps.blocked_ports();

        let port_bytes = port.to_ne_bytes();
        blocked_ports.delete(&port_bytes)?;
        log::info!("✅ Unblocked port: {}", port);
        Ok(())
    }

    /// Unblock an IPv4 address
    pub fn unblock_ip(&self, ip: Ipv4Addr) -> Result<()> {
        let skel = self.skel.as_ref().context("XDP not loaded")?;
        let maps = skel.maps();
        let blocklist = maps.blocklist_ipv4();

        let ip_bytes = u32::from(ip).to_be_bytes();
        blocklist.delete(&ip_bytes)?;
        log::info!("✅ Unblocked IP: {}", ip);
        Ok(())
    }

    /// Block multiple IPs in batch (for performance)
    pub fn block_ips_batch(&self, ips: &[Ipv4Addr]) -> Result<usize> {
        let mut blocked = 0;
        for ip in ips {
            if self.block_ip(*ip).is_ok() {
                blocked += 1;
            }
        }
        log::info!("🚫 Batch blocked {} IPs", blocked);
        Ok(blocked)
    }

    /// Stress test: Block and unblock rapidly
    pub fn stress_test(&self, iterations: usize) -> Result<()> {
        log::info!("🔥 Starting XDP stress test ({} iterations)...", iterations);

        let test_ips: Vec<Ipv4Addr> = (1..=iterations)
            .map(|i| {
                let octet = (i % 254 + 1) as u8;
                Ipv4Addr::new(192, 168, 100, octet)
            })
            .collect();

        let start = std::time::Instant::now();

        // Block all
        for ip in &test_ips {
            self.block_ip(*ip)?;
        }

        // Unblock all
        for ip in &test_ips {
            self.unblock_ip(*ip)?;
        }

        let elapsed = start.elapsed();
        let ops_per_sec = (iterations * 2) as f64 / elapsed.as_secs_f64();

        log::info!(
            "✅ Stress test complete: {} ops in {:?} ({:.0} ops/sec)",
            iterations * 2,
            elapsed,
            ops_per_sec
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_engine_creation() {
        let engine = NetEngine::new();
        assert!(engine.is_ok());
    }
}
