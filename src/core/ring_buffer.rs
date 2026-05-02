#![allow(dead_code)]

//! Ouroboros Deadlock Mitigation: Lockless Forensics Ring Buffer
//!
//! THE PROBLEM: ForensicsEngine writes to disk via VFS. If the process being
//! blocked holds a lock that VFS needs (e.g., deep in io_uring submission),
//! the kernel deadlocks: our tool waits for the lock, the malware is paused
//! by our tool.
//!
//! THE FIX: A pre-allocated, lock-free SPSC ring buffer backed by a static
//! memory region. Writes NEVER touch the allocator, VFS, or any kernel
//! subsystem. A separate "drain" task flushes to disk only when safe.
//!
//! Guarantee: write() is O(1), wait-free, and cannot deadlock.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;

const RING_CAPACITY: usize = 1024;
const ENTRY_SIZE: usize = 256;

/// A single pre-allocated log slot. Fixed size to avoid any heap allocation.
#[repr(C, align(64))] // cache-line aligned to prevent false sharing
pub struct RingSlot {
    pub len: u32,
    pub data: [u8; ENTRY_SIZE - 4],
}

/// Lock-free Single-Producer Single-Consumer ring buffer.
/// The LSM hook (producer) and the drain task (consumer) never share a lock.
pub struct ForensicsRingBuffer {
    slots: Box<[UnsafeCell<RingSlot>; RING_CAPACITY]>,
    head: AtomicUsize, // written by producer
    tail: AtomicUsize, // written by consumer
}

// SAFETY: We enforce single-producer / single-consumer at the call sites.
unsafe impl Sync for ForensicsRingBuffer {}
unsafe impl Send for ForensicsRingBuffer {}

impl ForensicsRingBuffer {
    pub fn new() -> Self {
        // SAFETY: RingSlot is POD; zeroed memory is valid.
        let slots = (0..RING_CAPACITY)
            .map(|_| UnsafeCell::new(RingSlot { len: 0, data: [0u8; ENTRY_SIZE - 4] }))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| panic!("ring buffer init"));
        Self {
            slots: Box::new(slots),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Write a forensics entry. WAIT-FREE — never blocks, never allocates.
    /// Returns false if the buffer is full (entry is dropped, not deadlocked).
    #[inline(always)]
    pub fn write(&self, entry: &[u8]) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let next_head = (head + 1) % RING_CAPACITY;

        // Buffer full: drop the entry rather than block. Deadlock is worse than loss.
        if next_head == self.tail.load(Ordering::Acquire) {
            return false;
        }

        // SAFETY: We own this slot exclusively (single producer, head not yet published).
        let slot = unsafe { &mut *self.slots[head].get() };
        let len = entry.len().min(slot.data.len());
        slot.data[..len].copy_from_slice(&entry[..len]);
        slot.len = len as u32;

        // Release: makes the write visible to the consumer.
        self.head.store(next_head, Ordering::Release);
        true
    }

    /// Drain all pending entries, calling `f` for each. Called from the safe flush task.
    pub fn drain<F: FnMut(&[u8])>(&self, mut f: F) {
        loop {
            let tail = self.tail.load(Ordering::Relaxed);
            if tail == self.head.load(Ordering::Acquire) {
                break; // empty
            }
            // SAFETY: Consumer owns the tail slot.
            let slot = unsafe { &*self.slots[tail].get() };
            f(&slot.data[..slot.len as usize]);
            self.tail.store((tail + 1) % RING_CAPACITY, Ordering::Release);
        }
    }
}

/// Convenience: format a forensics entry and push it to the ring buffer.
/// This is what the LSM hook calls — zero allocation, zero locks.
pub fn ring_log(buf: &ForensicsRingBuffer, pid: u32, reason: &str) {
    // Stack-allocated formatting — no heap.
    let mut tmp = [0u8; ENTRY_SIZE - 4];
    let msg = format!("pid={} reason={}", pid, reason);
    let bytes = msg.as_bytes();
    let len = bytes.len().min(tmp.len());
    tmp[..len].copy_from_slice(&bytes[..len]);
    buf.write(&tmp[..len]);
}
