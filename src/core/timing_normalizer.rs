#![allow(dead_code)]

//! Heisenbug Payload Mitigation: Syscall Timing Normalization
//!
//! THE PROBLEM: Malware uses `rdtsc` to measure its own syscall latency.
//! If a syscall takes ~100µs it's unmonitored; if it takes ~3-5ms it knows
//! Nexus Axiom's AI jury is watching and goes permanently dormant.
//!
//! THE FIX: Every intercepted syscall is padded to a CONSTANT target window
//! (e.g., 5ms ± random jitter). The malware's rdtsc measurement is now
//! meaningless — monitored and unmonitored calls are indistinguishable.
//!
//! Secondary defense: inject random sub-millisecond jitter to defeat
//! statistical timing attacks that average many samples.

use std::time::{Duration, Instant};

/// Target window for all normalized syscalls (5ms baseline).
const NORMALIZATION_WINDOW_MS: u64 = 5;

/// Maximum additional jitter in microseconds (uniform random).
/// Prevents statistical de-anonymization via repeated sampling.
const MAX_JITTER_US: u64 = 800;

pub struct TimingNormalizer {
    /// Pseudo-random state (xorshift64 — no stdlib RNG, no allocation).
    rng_state: u64,
}

impl TimingNormalizer {
    pub fn new(seed: u64) -> Self {
        Self { rng_state: if seed == 0 { 0xdeadbeef_cafebabe } else { seed } }
    }

    /// Xorshift64 — fast, no allocation, no syscall.
    #[inline(always)]
    fn next_rand(&mut self) -> u64 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        self.rng_state
    }

    /// Pad the remaining time in the normalization window, then add jitter.
    /// Call this AFTER your LSM hook / AI jury work completes.
    ///
    /// `work_start`: when the hook began.
    /// Returns the actual sleep duration (for telemetry).
    pub fn normalize(&mut self, work_start: Instant) -> Duration {
        let elapsed = work_start.elapsed();
        let target = Duration::from_millis(NORMALIZATION_WINDOW_MS);
        let jitter = Duration::from_micros(self.next_rand() % MAX_JITTER_US);
        let total_target = target + jitter;

        if elapsed < total_target {
            let sleep_for = total_target - elapsed;
            std::thread::sleep(sleep_for);
            sleep_for
        } else {
            // Work took longer than the window (e.g., heavy AI inference).
            // Still add jitter so the malware can't detect the overrun.
            std::thread::sleep(jitter);
            jitter
        }
    }
}

/// Wraps a closure in a normalized timing window.
/// The closure is your LSM hook body. The caller sees constant latency.
pub fn with_normalized_timing<F, T>(normalizer: &mut TimingNormalizer, f: F) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    normalizer.normalize(start);
    result
}
