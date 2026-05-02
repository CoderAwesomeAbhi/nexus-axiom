use anyhow::{Context, Result};
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use libbpf_rs::MapFlags;
use std::net::Ipv4Addr;

include!(concat!(env!("OUT_DIR"), "/nexus_net.skel.rs"));

pub struct NetEngine {
    skel: Option<NexusNetSkel<'static>>,
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
        Ok(())
    }
    
    pub fn block_ip(&self, ip: Ipv4Addr) -> Result<()> {
        let skel = self.skel.as_ref().context("XDP not loaded")?;
        let maps = skel.maps();
        let blocklist = maps.blocklist_ipv4();
        
        let ip_bytes = u32::from(ip).to_be_bytes(); // Convert to network byte order
        let val: u8 = 1;
        
        blocklist.update(&ip_bytes, &val.to_ne_bytes(), MapFlags::ANY)?;
        log::info!("🚫 Blocked IP: {}", ip);
        Ok(())
    }
}
