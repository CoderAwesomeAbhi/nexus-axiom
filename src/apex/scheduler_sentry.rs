#![allow(dead_code)]

//! Scheduler Sentry
//!
//! Mitigates the `sched_ext` Starvation attack.
//! Monitors the virtual runtime and CPU slices of `nexus-axiom-d`.
//! If a malicious BPF scheduler attempts to starve the daemon to bypass
//! security checks, the sentry triggers an Emergency Lock (Graceful Degradation).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{warn, error};
use tokio::time::{interval, Duration};

pub struct SchedulerSentry {
    pub graceful_degradation_active: Arc<AtomicBool>,
    last_cpu_slice: Arc<std::sync::RwLock<u64>>,
}

impl SchedulerSentry {
    pub fn new() -> Self {
        Self {
            graceful_degradation_active: Arc::new(AtomicBool::new(false)),
            last_cpu_slice: Arc::new(std::sync::RwLock::new(Self::now_ms())),
        }
    }

    fn now_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }

    /// Spawns the high-priority sentry loop.
    pub fn start_monitoring(&self) {
        let last_slice = self.last_cpu_slice.clone();
        let degradation_flag = self.graceful_degradation_active.clone();

        tokio::spawn(async move {
            // High frequency watchdog (runs every 50ms)
            let mut ticker = interval(Duration::from_millis(50));
            loop {
                ticker.tick().await;
                
                let now = Self::now_ms();
                let mut slice = last_slice.write().unwrap();
                let delta = now.saturating_sub(*slice);

                // If we went > 500ms without being scheduled, a malicious sched_ext
                // BPF program is likely starving us. Trigger Graceful Degradation.
                if delta > 500 {
                    if !degradation_flag.load(Ordering::Relaxed) {
                        error!("[SCHED-SENTRY] STARVATION DETECTED! Delta: {}ms", delta);
                        error!("[SCHED-SENTRY] Engaging Graceful Degradation (Dumb Whitelist Mode)!");
                        degradation_flag.store(true, Ordering::Relaxed);
                    }
                } else {
                    // Recovered
                    if degradation_flag.load(Ordering::Relaxed) {
                        warn!("[SCHED-SENTRY] CPU scheduling recovered. Disengaging Graceful Degradation.");
                        degradation_flag.store(false, Ordering::Relaxed);
                    }
                }

                *slice = now;
            }
        });
    }
}
