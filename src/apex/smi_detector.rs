#![allow(dead_code)]

//! Ring -2 Phantom: Detecting Invisible Firmware Pauses
//!
//! System Management Mode (SMM) is Ring -2. When a System Management Interrupt
//! (SMI) occurs, the OS is completely suspended while firmware executes. This is
//! a prime hiding spot for the most advanced hardware-level rootkits.
//!
//! This module monitors MSR 0x34 (MSR_SMI_COUNT) to detect anomalous spikes
//! in SMIs, which mathematically proves firmware execution out-of-bounds.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use log::{error, info, warn};
use tokio::sync::RwLock;

/// Intel Model Specific Register for SMI count
const MSR_SMI_COUNT: u64 = 0x34;
/// Threshold for anomalous SMIs per second
const ANOMALOUS_SMI_RATE_PER_SEC: u64 = 150;

#[derive(Debug, Clone)]
pub struct SmiState {
    pub last_count: u64,
    pub last_checked: Instant,
    pub is_anomalous: bool,
}

pub struct SmiDetector {
    state: Arc<RwLock<Option<SmiState>>>,
    enabled: bool,
}

impl SmiDetector {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(None)),
            enabled: true,
        }
    }

    /// Read an MSR on CPU 0
    #[cfg(target_os = "linux")]
    fn read_msr(msr: u64) -> Result<u64> {
        let mut file = File::open("/dev/cpu/0/msr")
            .context("Failed to open /dev/cpu/0/msr (is the msr module loaded?)")?;
        file.seek(SeekFrom::Start(msr))?;
        
        let mut buffer = [0u8; 8];
        file.read_exact(&mut buffer)?;
        Ok(u64::from_le_bytes(buffer))
    }

    #[cfg(not(target_os = "linux"))]
    fn read_msr(_msr: u64) -> Result<u64> {
        anyhow::bail!("MSR reading only supported on Linux")
    }

    /// Initialize the baseline SMI count
    pub async fn initialize(&mut self) -> Result<()> {
        match Self::read_msr(MSR_SMI_COUNT) {
            Ok(count) => {
                info!("[RING-2] Initial SMI count: {}", count);
                let mut state = self.state.write().await;
                *state = Some(SmiState {
                    last_count: count,
                    last_checked: Instant::now(),
                    is_anomalous: false,
                });
                Ok(())
            }
            Err(e) => {
                warn!("[RING-2] MSR access unavailable: {}. Disabling Ring -2 Phantom detection.", e);
                self.enabled = false;
                Ok(()) // Graceful degradation
            }
        }
    }

    /// Check for anomalous SMI spikes
    pub async fn check_anomaly(&self) -> bool {
        if !self.enabled {
            return false;
        }

        let current_count = match Self::read_msr(MSR_SMI_COUNT) {
            Ok(c) => c,
            Err(e) => {
                error!("[RING-2] Failed to read MSR during active check: {}", e);
                return false;
            }
        };

        let mut state_guard = self.state.write().await;
        if let Some(state) = state_guard.as_mut() {
            let elapsed = state.last_checked.elapsed().as_secs_f64();
            if elapsed >= 1.0 { // Avoid division by zero
                let delta = current_count.saturating_sub(state.last_count);
                let rate = (delta as f64 / elapsed) as u64;

                if rate > ANOMALOUS_SMI_RATE_PER_SEC {
                    warn!("[RING-2] 🚨 ANOMALY DETECTED! SMI Rate: {}/sec (Threshold: {})", rate, ANOMALOUS_SMI_RATE_PER_SEC);
                    state.is_anomalous = true;
                } else {
                    state.is_anomalous = false;
                }

                state.last_count = current_count;
                state.last_checked = Instant::now();
                return state.is_anomalous;
            }
        }
        false
    }

    /// Returns true if the system is currently under a suspected Ring -2 attack
    pub async fn is_under_attack(&self) -> bool {
        if let Some(state) = self.state.read().await.as_ref() {
            state.is_anomalous
        } else {
            false
        }
    }
}
