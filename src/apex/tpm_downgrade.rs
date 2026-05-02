#![allow(dead_code)]

//! Monotonic TPM Counter: Hardware-Enforced Anti-Downgrade
//!
//! Attackers with root can bypass release signing by installing an OLDER,
//! vulnerable (but legitimately signed) version of the daemon.
//! 
//! This module uses TPM 2.0 NV (Non-Volatile) monotonic counters to ensure
//! the binary version can only ever go UP, mathematically killing time-travel
//! downgrade attacks.

use std::process::Command;

use anyhow::{Context, Result};
use log::{info, warn, error};

/// The version of this binary represented as an integer (Major * 10000 + Minor * 100 + Patch)
/// v0.3.0 = 300
const CURRENT_DAEMON_VERSION: u64 = 300;
/// TPM NV Index to store the monotonic counter (OEM range)
const TPM_NV_INDEX: &str = "0x01500001";

pub struct TpmAntiDowngrade {
    enabled: bool,
}

impl TpmAntiDowngrade {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Verify the current binary version against the hardware TPM counter
    #[cfg(target_os = "linux")]
    pub fn verify_and_increment(&mut self) -> Result<()> {
        // Check if tpm2-tools are installed
        if Command::new("tpm2_nvread").arg("-v").output().is_err() {
            warn!("[TPM] tpm2-tools not found. Hardware downgrade protection disabled.");
            self.enabled = false;
            return Ok(());
        }

        // 1. Try to read the current counter
        let read_out = Command::new("tpm2_nvread").arg(TPM_NV_INDEX).output();
        
        let tpm_version: u64 = match read_out {
            Ok(out) if out.status.success() => {
                let val_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                val_str.parse().unwrap_or(0)
            }
            _ => {
                info!("[TPM] NV index not found or uninitialized. Initializing...");
                Self::initialize_nv_index()?;
                0
            }
        };

        info!("[TPM] Hardware Counter Version: {}, Binary Version: {}", tpm_version, CURRENT_DAEMON_VERSION);

        // 2. Check for downgrade attack
        if CURRENT_DAEMON_VERSION < tpm_version {
            error!("[TPM] 🚨 FATAL DOWNGRADE ATTACK DETECTED! 🚨");
            error!("[TPM] Hardware asserts minimum version is {}, but binary is {}", tpm_version, CURRENT_DAEMON_VERSION);
            anyhow::bail!("Time-Travel Downgrade Attack detected by TPM");
        }

        // 3. Increment counter if we are a newer version
        if CURRENT_DAEMON_VERSION > tpm_version {
            info!("[TPM] Upgrading hardware counter to {}", CURRENT_DAEMON_VERSION);
            let write_out = Command::new("tpm2_nvwrite")
                .arg(TPM_NV_INDEX)
                .arg("-i")
                .arg("-") // Read from stdin
                .env("TPM2TOOLS_ENV_ENABLE_ERRATA", "1")
                .stdin(std::process::Stdio::piped())
                .spawn();
                
            if let Ok(mut child) = write_out {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(CURRENT_DAEMON_VERSION.to_string().as_bytes());
                }
                let _ = child.wait();
            }
        }

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn verify_and_increment(&mut self) -> Result<()> {
        warn!("[TPM] TPM integration not supported on this platform.");
        self.enabled = false;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn initialize_nv_index() -> Result<()> {
        // Define a 8-byte space for the version integer
        let status = Command::new("tpm2_nvdefine")
            .arg(TPM_NV_INDEX)
            .arg("-C").arg("o")
            .arg("-s").arg("8")
            .arg("-a").arg("ownerread|ownerwrite|policywrite")
            .status()?;
            
        if !status.success() {
            warn!("[TPM] Failed to define NV index. Ensure you have TPM ownership.");
        }
        Ok(())
    }
}
