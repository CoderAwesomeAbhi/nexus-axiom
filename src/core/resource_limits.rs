#![allow(dead_code)]

//! Resource Limits & Daemon Hardening
//!
//! Enforces `rlimit` constraints on the Nexus Axiom user-space daemon
//! to prevent File Descriptor Exhaustion DoS and other local attacks.

use log::{info, warn};

pub struct ResourceLimits;

impl ResourceLimits {
    pub fn enforce_safe_ceilings() {
        // Enforce maximum open file descriptors (RLIMIT_NOFILE)
        // Set to a reasonable ceiling to prevent FD exhaustion attacks.
        #[cfg(target_os = "linux")]
        unsafe {
            let rlim = libc::rlimit {
                rlim_cur: 4096,
                rlim_max: 4096,
            };
            if libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) != 0 {
                warn!("[HARDEN] Failed to set RLIMIT_NOFILE. FD exhaustion possible.");
            } else {
                info!("[HARDEN] RLIMIT_NOFILE locked to 4096. FD Exhaustion mitigated.");
            }
        }
        #[cfg(not(target_os = "linux"))]
        info!("[HARDEN] Simulated setting RLIMIT_NOFILE (non-Linux OS).");
    }
}
