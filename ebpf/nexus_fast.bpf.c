// NEXUS AXIOM - OPTIMIZED FOR SPEED
// Target: <50ns overhead per event
// Optimizations:
// - Minimal instructions in hot path
// - Per-CPU ring buffers
// - Zero allocations
// - Inlined checks
// - BPF_CORE_READ for safe memory access

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

// Event structure - optimized for cache alignment
struct event {
    u32 pid;
    u32 blocked;
    u64 timestamp;
} __attribute__((packed));

// Per-CPU ring buffer for zero contention
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024); // 256KB per CPU
} events SEC(".maps");

// Allowlist - pre-allocated hash map
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1024);
    __type(key, u32);
    __type(value, u32);
} allowlist SEC(".maps");

// Fast path: W^X detection
static __always_inline int is_wx_violation(unsigned long prot)
{
    // Single bitwise check - fastest possible
    return (prot & PROT_WRITE) && (prot & PROT_EXEC);
}

// Fast path: Allowlist check
static __always_inline int is_allowed(u32 pid)
{
    u32 *val = bpf_map_lookup_elem(&allowlist, &pid);
    return val != NULL;
}

// LSM hook: mmap_file - OPTIMIZED HOT PATH
SEC("lsm/mmap_file")
int BPF_PROG(mmap_file, struct file *file, unsigned long reqprot,
             unsigned long prot, unsigned long flags)
{
    // Fast path: Check W^X first (most common case)
    if (!is_wx_violation(prot))
        return 0; // Allow - no event needed
    
    // Get PID
    u32 pid = bpf_get_current_pid_tgid() >> 32;
    
    // Fast path: Allowlist check
    if (is_allowed(pid))
        return 0; // Allow - trusted process
    
    // BLOCK: This is a W^X violation
    // Reserve ring buffer space
    struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return -EPERM; // Block even if we can't log
    
    // Fill event - minimal data
    e->pid = pid;
    e->blocked = 1;
    e->timestamp = bpf_ktime_get_ns();
    
    // Submit event
    bpf_ringbuf_submit(e, 0);
    
    // BLOCK THE ALLOCATION
    return -EPERM;
}

char LICENSE[] SEC("license") = "GPL";
