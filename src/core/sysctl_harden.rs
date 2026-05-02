#![allow(dead_code)]

//! Sysctl Hardening
//!
//! Enforces `net.core.bpf_jit_harden=2` to ensure that eBPF JIT
//! compiled code is constantly blinded. This prevents ROP chain
//! overwrites into the executable memory where Axiom resides.

use std::fs;
use log::{info, error};

pub struct SysctlHarden;

impl SysctlHarden {
    pub fn enforce_bpf_jit_hardening() {
        let path = "/proc/sys/net/core/bpf_jit_harden";
        
        // Attempt to write "2" to the sysctl interface
        #[cfg(target_os = "linux")]
        match fs::write(path, b"2") {
            Ok(_) => info!("[HARDEN] Successfully enforced bpf_jit_harden=2."),
            Err(e) => error!("[HARDEN] Failed to set bpf_jit_harden: {}. Are we running as root?", e),
        }
        #[cfg(not(target_os = "linux"))]
        info!("[HARDEN] Simulated enforcing bpf_jit_harden=2 (non-Linux OS).");
    }
}
