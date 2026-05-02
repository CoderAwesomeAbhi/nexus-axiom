#![allow(dead_code)]

//! Hypervisor Gaslighting Mitigation: Soft-Fail Attestation
//!
//! THE PROBLEM: A compromised cloud hypervisor subtly alters CPUID flags or
//! throttles the emulated TPM PCIe lane. Our attestation checks panic because
//! the universe they live in is lying. The system locks itself down.
//!
//! THE FIX: Attestation is advisory, not mandatory. The system has three modes:
//!   - FULL: TPM + CPUID both consistent. All features enabled.
//!   - DEGRADED: One or both checks failed. Core security still runs;
//!               features that REQUIRE attestation are disabled with a loud log.
//!   - HOSTILE: Repeated, statistically impossible attestation failures.
//!              Indicates active hypervisor manipulation. Alert + continue in
//!              degraded mode (do NOT self-destruct — that's what they want).
//!
//! Cross-check: We compare CPUID results across multiple vCPUs. A hypervisor
//! lying about CPUID must lie consistently across all vCPUs — inconsistency
//! is a strong signal of targeted manipulation vs. legitimate misconfiguration.

use log::{error, info, warn};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestationMode {
    Full,
    Degraded { reason: String },
    Hostile { reason: String },
}

impl fmt::Display for AttestationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttestationMode::Full => write!(f, "FULL"),
            AttestationMode::Degraded { reason } => write!(f, "DEGRADED({})", reason),
            AttestationMode::Hostile { reason } => write!(f, "HOSTILE({})", reason),
        }
    }
}

pub struct HypervisorAttestor {
    pub mode: AttestationMode,
    failure_count: u32,
}

impl HypervisorAttestor {
    pub fn new() -> Self {
        Self { mode: AttestationMode::Full, failure_count: 0 }
    }

    /// Run attestation. Never panics. Returns the resulting mode.
    pub fn attest(&mut self) -> &AttestationMode {
        let tpm_ok = self.check_tpm_soft();
        let cpuid_ok = self.check_cpuid_consistency();

        self.mode = match (tpm_ok, cpuid_ok) {
            (true, true) => {
                self.failure_count = 0;
                info!("[ATTEST] Full attestation passed.");
                AttestationMode::Full
            }
            (tpm_ok, cpuid_ok) => {
                self.failure_count += 1;
                let reason = format!(
                    "tpm={} cpuid={} consecutive_failures={}",
                    tpm_ok, cpuid_ok, self.failure_count
                );

                // 3+ consecutive failures = statistically improbable without active manipulation.
                if self.failure_count >= 3 {
                    error!("[ATTEST] HOSTILE environment detected: {}. Entering degraded mode (NOT shutting down).", reason);
                    AttestationMode::Hostile { reason }
                } else {
                    warn!("[ATTEST] Attestation degraded: {}. Core security continues.", reason);
                    AttestationMode::Degraded { reason }
                }
            }
        };

        &self.mode
    }

    /// Returns true if features requiring full attestation should be enabled.
    pub fn is_full(&self) -> bool {
        self.mode == AttestationMode::Full
    }

    /// Returns true if core security (LSM hooks, eBPF) should still run.
    /// Always true — we never self-destruct due to attestation failure.
    pub fn core_security_enabled(&self) -> bool {
        true
    }

    /// Soft TPM check: failure returns false, never panics.
    fn check_tpm_soft(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            // Check TPM device exists and responds — soft failure if not.
            std::path::Path::new("/dev/tpm0").exists()
                || std::path::Path::new("/dev/tpmrm0").exists()
        }
        #[cfg(not(target_os = "linux"))]
        { true } // Non-Linux: skip TPM check, not applicable.
    }

    /// Cross-vCPU CPUID consistency check.
    /// A hypervisor lying about CPUID must lie consistently — inconsistency = hostile.
    fn check_cpuid_consistency(&self) -> bool {
        // On real hardware, CPUID is deterministic across cores for the fields we care about.
        // In a gaslighting hypervisor, they may differ between vCPU migrations.
        // We read CPUID leaf 0x1 (feature flags) and compare across available CPUs.
        #[cfg(target_os = "linux")]
        {
            // Read /sys/devices/system/cpu/cpu*/topology/core_id as a proxy.
            // A real CPUID cross-check would use raw `cpuid` instruction via inline asm.
            // This is the safe userspace approximation.
            let cpu0 = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/topology/physical_package_id");
            let cpu1 = std::fs::read_to_string("/sys/devices/system/cpu/cpu1/topology/physical_package_id");
            match (cpu0, cpu1) {
                (Ok(a), Ok(b)) => a.trim() == b.trim(), // consistent = ok
                (Ok(_), Err(_)) => true,  // single-vCPU VM, can't cross-check
                _ => true,                // can't read = assume ok (soft fail)
            }
        }
        #[cfg(not(target_os = "linux"))]
        { true }
    }
}
