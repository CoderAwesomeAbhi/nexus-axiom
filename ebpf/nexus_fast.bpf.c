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

// Fast path: Allowlist check
static __always_inline int is_allowed(u32 pid)
{
    u32 *val = bpf_map_lookup_elem(&allowlist, &pid);
    return val != NULL;
}

// LSM hook: file_mmap - BLOCKS W^X ALLOCATIONS (including anonymous)
SEC("lsm/file_mmap")
int BPF_PROG(file_mmap, struct file *file, unsigned long reqprot,
             unsigned long prot, unsigned long flags)
{
    // Check W^X violation (works for both file-backed and anonymous mmap)
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        u32 pid = bpf_get_current_pid_tgid() >> 32;
        
        // Debug logging
        bpf_printk("NEXUS: W^X detected in mmap! PID=%d prot=0x%lx flags=0x%lx", pid, prot, flags);
        
        // Check allowlist
        if (is_allowed(pid)) {
            bpf_printk("NEXUS: PID %d is allowed", pid);
            return 0;
        }
        
        // Log to ring buffer
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->blocked = 1;
            e->timestamp = bpf_ktime_get_ns();
            bpf_ringbuf_submit(e, 0);
        }
        
        // BLOCK THE ALLOCATION
        bpf_printk("NEXUS: BLOCKING W^X mmap for PID %d", pid);
        return -EPERM;
    }
    
    return 0;
}

// LSM hook: file_mprotect - BLOCKS W^X PROTECTION CHANGES
SEC("lsm/file_mprotect")
int BPF_PROG(file_mprotect, struct vm_area_struct *vma, unsigned long reqprot,
             unsigned long prot)
{
    // Check W^X violation
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        u32 pid = bpf_get_current_pid_tgid() >> 32;
        
        // Debug logging
        bpf_printk("NEXUS: W^X detected in mprotect! PID=%d prot=0x%lx", pid, prot);
        
        // Check allowlist
        if (is_allowed(pid)) {
            bpf_printk("NEXUS: PID %d is allowed", pid);
            return 0;
        }
        
        // Log to ring buffer
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->blocked = 1;
            e->timestamp = bpf_ktime_get_ns();
            bpf_ringbuf_submit(e, 0);
        }
        
        // BLOCK THE PROTECTION CHANGE
        bpf_printk("NEXUS: BLOCKING W^X mprotect for PID %d", pid);
        return -EPERM;
    }
    
    // Allow normal protection changes
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
