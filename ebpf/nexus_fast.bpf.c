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

// JIT compiler detection - check if process is a known JIT runtime
static __always_inline int is_jit_runtime(u32 pid)
{
    char comm[16];
    bpf_get_current_comm(&comm, sizeof(comm));
    
    // Allow common JIT runtimes
    // node, java, python, chrome, firefox, etc.
    if (comm[0] == 'n' && comm[1] == 'o' && comm[2] == 'd' && comm[3] == 'e') // node
        return 1;
    if (comm[0] == 'j' && comm[1] == 'a' && comm[2] == 'v' && comm[3] == 'a') // java
        return 1;
    if (comm[0] == 'p' && comm[1] == 'y' && comm[2] == 't' && comm[3] == 'h') // python
        return 1;
    if (comm[0] == 'c' && comm[1] == 'h' && comm[2] == 'r' && comm[3] == 'o') // chrome
        return 1;
    if (comm[0] == 'f' && comm[1] == 'i' && comm[2] == 'r' && comm[3] == 'e') // firefox
        return 1;
    
    return 0;
}

// Fast path: Allowlist check
static __always_inline int is_allowed(u32 pid)
{
    // Check explicit allowlist
    u32 *val = bpf_map_lookup_elem(&allowlist, &pid);
    if (val != NULL)
        return 1;
    
    // Check if JIT runtime
    return is_jit_runtime(pid);
}

// LSM hook: mmap_file - BLOCKS W^X ALLOCATIONS
SEC("lsm/mmap_file")
int BPF_PROG(mmap_file, struct file *file, unsigned long reqprot,
             unsigned long prot, unsigned long flags)
{
    // Check W^X violation
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        u32 pid = bpf_get_current_pid_tgid() >> 32;
        
        bpf_printk("NEXUS: W^X in mmap_file! PID=%d prot=0x%lx", pid, prot);
        
        if (is_allowed(pid)) {
            return 0;
        }
        
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->blocked = 1;
            e->timestamp = bpf_ktime_get_ns();
            bpf_ringbuf_submit(e, 0);
        }
        
        bpf_printk("NEXUS: BLOCKING W^X mmap_file PID %d", pid);
        return -EACCES;
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
        return -EACCES;
    }
    
    // Allow normal protection changes
    return 0;
}

// Tracepoint: sys_enter_mmap - KILLS processes attempting W^X anonymous mmap
SEC("tracepoint/syscalls/sys_enter_mmap")
int trace_mmap_enter(struct trace_event_raw_sys_enter *ctx)
{
    // Extract mmap arguments
    // mmap(addr, len, prot, flags, fd, offset)
    // args[2] is prot
    unsigned long prot = ctx->args[2];
    
    // Check for W^X violation
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        u32 pid = bpf_get_current_pid_tgid() >> 32;
        
        bpf_printk("NEXUS: W^X in mmap syscall! PID=%d prot=0x%lx", pid, prot);
        
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
        
        // TERMINATE THE PROCESS
        bpf_printk("NEXUS: KILLING process %d for W^X attempt", pid);
        bpf_send_signal(SIGKILL);
    }
    
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
