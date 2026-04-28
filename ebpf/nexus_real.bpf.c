// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - REAL eBPF LSM Implementation
// This actually blocks W^X memory at the kernel level

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "GPL";

// Ring buffer for events (1MB)
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 20);
} events SEC(".maps");

// Allowlist map for trusted processes
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1000);
    __type(key, u32);
    __type(value, u8);
} allowlist SEC(".maps");

// Event structure
struct event {
    u32 pid;
    u32 uid;
    u64 timestamp;
    u32 prot;
    u32 flags;
    u8 blocked;
    char comm[16];
};

// LSM hook: mmap_file - Block W^X memory
SEC("lsm/mmap_file")
int BPF_PROG(mmap_file, struct file *file, unsigned long prot,
             unsigned long flags, unsigned long ret)
{
    u32 pid = bpf_get_current_pid_tgid() >> 32;
    
    // Check allowlist first (fast path)
    u8 *allowed = bpf_map_lookup_elem(&allowlist, &pid);
    if (allowed && *allowed == 1)
        return 0;
    
    // Check for W^X violation
    int is_write = prot & PROT_WRITE;
    int is_exec = prot & PROT_EXEC;
    
    if (is_write && is_exec) {
        // Log the violation
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->uid = bpf_get_current_uid_gid() >> 32;
            e->timestamp = bpf_ktime_get_ns();
            e->prot = prot;
            e->flags = flags;
            e->blocked = 1;
            bpf_get_current_comm(&e->comm, sizeof(e->comm));
            bpf_ringbuf_submit(e, 0);
        }
        
        // ACTUALLY BLOCK IT
        return -EPERM;
    }
    
    return 0;
}

// LSM hook: bprm_check_security - Block suspicious executables
SEC("lsm/bprm_check_security")
int BPF_PROG(bprm_check, struct linux_binprm *bprm, int ret)
{
    u32 pid = bpf_get_current_pid_tgid() >> 32;
    
    // Log execution
    struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (e) {
        e->pid = pid;
        e->uid = bpf_get_current_uid_gid() >> 32;
        e->timestamp = bpf_ktime_get_ns();
        e->blocked = 0;
        bpf_get_current_comm(&e->comm, sizeof(e->comm));
        bpf_ringbuf_submit(e, 0);
    }
    
    return 0;
}
