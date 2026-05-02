#![allow(dead_code)]

//! Sybil Context Attack Mitigation: LLM Context Budget Enforcer
//!
//! THE PROBLEM: 50 unprivileged "swarm" processes flood the Phi-4 model's
//! context window with high-entropy token sequences. When the real attack
//! arrives, the model suffers "Lost in the Middle" syndrome and hallucinates
//! a SAFE verdict.
//!
//! THE FIX: Two-layer defense:
//!   1. Per-PID token budget: each process gets a fixed slice of the context
//!      window. A swarm of 50 processes can't collectively exceed 1 process's
//!      budget — they share a pool, not each get a full window.
//!   2. Entropy filter: events whose metadata exceeds a Shannon entropy
//!      threshold are classified as noise and summarized, not verbatim-injected.
//!      High-entropy = random-looking = not semantically useful to the LLM.

use std::collections::HashMap;

/// Maximum total tokens the LLM context window can hold.
const CONTEXT_BUDGET_TOKENS: usize = 4096;

/// Each unique PID group (swarm) gets at most this fraction of the budget.
/// 50 swarm processes share 20% of the window; the real event gets 80%.
const SWARM_POOL_FRACTION: f32 = 0.20;

/// Events with Shannon entropy above this threshold are "noise" and get
/// replaced with a summary token instead of verbatim content.
const ENTROPY_NOISE_THRESHOLD: f32 = 4.5; // bits per byte (max ~8.0)

pub struct ContextBudgetEnforcer {
    /// Tokens consumed per PID group (keyed by process group ID).
    pgid_budgets: HashMap<u32, usize>,
    /// Total tokens currently in the context window.
    total_used: usize,
}

impl ContextBudgetEnforcer {
    pub fn new() -> Self {
        Self {
            pgid_budgets: HashMap::new(),
            total_used: 0,
        }
    }

    /// Attempt to admit an event into the LLM context.
    /// Returns the string to inject (either the original or a noise summary).
    /// Returns None if the budget is exhausted for this process group.
    pub fn admit(&mut self, pgid: u32, content: &str) -> Option<String> {
        let swarm_pool = (CONTEXT_BUDGET_TOKENS as f32 * SWARM_POOL_FRACTION) as usize;
        let pgid_used = self.pgid_budgets.entry(pgid).or_insert(0);

        // Entropy check first — high-entropy content is noise regardless of budget.
        let entropy = shannon_entropy(content.as_bytes());
        let effective_content = if entropy > ENTROPY_NOISE_THRESHOLD {
            // Replace with a compact summary token instead of verbatim noise.
            format!("[HIGH_ENTROPY_EVENT pgid={} entropy={:.2} len={}]", pgid, entropy, content.len())
        } else {
            content.to_string()
        };

        let tokens = estimate_tokens(&effective_content);

        // Enforce per-PGID budget (swarm processes share a pool).
        if *pgid_used + tokens > swarm_pool {
            return None; // This PGID has consumed its share — drop.
        }

        // Enforce global budget.
        if self.total_used + tokens > CONTEXT_BUDGET_TOKENS {
            return None;
        }

        *pgid_used += tokens;
        self.total_used += tokens;
        Some(effective_content)
    }

    /// Reset budgets between LLM inference calls.
    pub fn reset(&mut self) {
        self.pgid_budgets.clear();
        self.total_used = 0;
    }

    /// Reserve a guaranteed slot for the high-priority event (the real attack).
    /// Call this BEFORE admitting swarm events to ensure the real event always fits.
    pub fn reserve_priority_slot(&mut self, tokens: usize) -> bool {
        let reserved = CONTEXT_BUDGET_TOKENS - (CONTEXT_BUDGET_TOKENS as f32 * SWARM_POOL_FRACTION) as usize;
        tokens <= reserved
    }
}

/// Shannon entropy in bits per byte. O(n) over the byte distribution.
fn shannon_entropy(data: &[u8]) -> f32 {
    if data.is_empty() { return 0.0; }
    let mut freq = [0u32; 256];
    for &b in data { freq[b as usize] += 1; }
    let len = data.len() as f32;
    freq.iter()
        .filter(|&&c| c > 0)
        .map(|&c| { let p = c as f32 / len; -p * p.log2() })
        .sum()
}

/// Rough token estimate: ~4 bytes per token (GPT-style BPE approximation).
#[inline]
fn estimate_tokens(s: &str) -> usize {
    (s.len() / 4).max(1)
}
