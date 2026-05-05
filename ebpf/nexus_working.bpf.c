// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - eBPF LSM that blocks W^X and reports events

#include <linux/bpf.h>
#include <linux/errno.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char LICENSE[] SEC("license") = "GPL";

#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

// Event struct - must match Rust side
struct event {
    __u32 pid;
    __u32 uid;
    __u64 timestamp;
    __u32 prot;
    __u32 flags;
    __u8  blocked;
    __u8  event_type;
    __u8  _pad[2];
    __u64 cgroup_id;
    __u8  comm[16];
};

// Maps
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 20);
} events SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1024);
    __type(key, __u32);
    __type(value, __u8);
} allowlist SEC(".maps");

// Config map for audit mode
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __uint(max_entries, 1);
    __type(key, __u32);
    __type(value, __u8);
} config SEC(".maps");

// LSM hook: mmap_file - Block W^X mmap
SEC("lsm/mmap_file")
int mmap_file_hook(void *ctx)
{
    unsigned long *args = ctx;
    unsigned long prot = args[2];
    
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        // Check audit mode
        __u32 key = 0;
        __u8 *audit_mode = bpf_map_lookup_elem(&config, &key);
        __u8 should_block = (audit_mode && *audit_mode == 0) ? 1 : 0;
        
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = bpf_get_current_pid_tgid() >> 32;
            e->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
            e->timestamp = bpf_ktime_get_ns();
            e->prot = prot;
            e->flags = args[3];
            e->blocked = should_block;
            e->event_type = 1; // EVENT_TYPE_MMAP
            e->cgroup_id = bpf_get_current_cgroup_id();
            bpf_get_current_comm(&e->comm, sizeof(e->comm));
            bpf_ringbuf_submit(e, 0);
        }
        return should_block ? -EPERM : 0;
    }
    return 0;
}

// LSM hook: file_mprotect - Block W^X mprotect
SEC("lsm/file_mprotect")
int mprotect_hook(void *ctx)
{
    unsigned long *args = ctx;
    unsigned long prot = args[2];
    
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        // Check audit mode
        __u32 key = 0;
        __u8 *audit_mode = bpf_map_lookup_elem(&config, &key);
        __u8 should_block = (audit_mode && *audit_mode == 0) ? 1 : 0;
        
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = bpf_get_current_pid_tgid() >> 32;
            e->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
            e->timestamp = bpf_ktime_get_ns();
            e->prot = prot;
            e->flags = 0;
            e->blocked = should_block;
            e->event_type = 4; // EVENT_TYPE_MPROTECT
            e->cgroup_id = bpf_get_current_cgroup_id();
            bpf_get_current_comm(&e->comm, sizeof(e->comm));
            bpf_ringbuf_submit(e, 0);
        }
        return should_block ? -EPERM : 0;
    }
    return 0;
}


// LSM hook: ptrace_access_check - Monitor debugging attempts
SEC("lsm/ptrace_access_check")
int ptrace_hook(void *ctx)
{
    struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (e) {
        e->pid = bpf_get_current_pid_tgid() >> 32;
        e->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
        e->timestamp = bpf_ktime_get_ns();
        e->prot = 0;
        e->flags = 0;
        e->blocked = 0; // Changed to 0 - just monitor, don't block
        e->event_type = 5; // EVENT_TYPE_PTRACE
        e->cgroup_id = bpf_get_current_cgroup_id();
        bpf_get_current_comm(&e->comm, sizeof(e->comm));
        bpf_ringbuf_submit(e, 0);
    }
    return 0; // Allow ptrace (gdb, strace, etc.)
}

// LSM hook: bprm_check_security - Monitor process execution
SEC("lsm/bprm_check_security")
int exec_hook(void *ctx)
{
    struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (e) {
        e->pid = bpf_get_current_pid_tgid() >> 32;
        e->uid = bpf_get_current_uid_gid() & 0xFFFFFFFF;
        e->timestamp = bpf_ktime_get_ns();
        e->prot = 0;
        e->flags = 0;
        e->blocked = 0;
        e->event_type = 6; // EVENT_TYPE_EXEC
        e->cgroup_id = bpf_get_current_cgroup_id();
        bpf_get_current_comm(&e->comm, sizeof(e->comm));
        bpf_ringbuf_submit(e, 0);
    }
    return 0;
}
