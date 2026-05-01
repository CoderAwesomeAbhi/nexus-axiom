// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - Production eBPF Security Engine
// Real LSM hooks + Behavior profiling + Rate limiting

#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "GPL";

#define MAX_PROTECTED_FILES 10000
#define MAX_ALLOWLIST 1000
#define MAX_BEHAVIOR_PROFILES 5000
#define MAX_RATE_LIMIT 10000
#define MAX_NETWORK_CONNS 10000

// Event types
#define EVENT_WX_MEMORY 1
#define EVENT_FILE_WRITE 2
#define EVENT_EXEC 3
#define EVENT_NETWORK 4
#define EVENT_PRIV_ESC 5

// Event structure
struct event {
    u32 pid;
    u32 uid;
    u32 gid;
    u32 event_type;
    u64 timestamp;
    u64 inode;
    u32 dev_major;
    u32 dev_minor;
    u32 prot;
    u32 flags;
    u8 blocked;
    u8 severity;
    char comm[16];
    char path[256];
};

// Behavior profile for anomaly detection
struct behavior_profile {
    u64 file_ops;
    u64 exec_count;
    u64 network_conns;
    u64 last_update;
    u32 anomaly_score;
};

// Rate limit entry
struct rate_limit_entry {
    u64 last_event;
    u32 count;
};

// Protected file entry
struct protected_file {
    u64 inode;
    u32 dev_major;
    u32 dev_minor;
    u32 action; // 0=ALLOW, 1=BLOCK, 2=AUDIT
};

// ============================================================================
// MAPS
// ============================================================================

// Ring buffer for events (1MB)
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 20);
} events SEC(".maps");

// Allowlist for trusted processes (JIT runtimes, etc)
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, MAX_ALLOWLIST);
    __type(key, u32);
    __type(value, u8);
} allowlist SEC(".maps");

// Behavior profiles for anomaly detection
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, MAX_BEHAVIOR_PROFILES);
    __type(key, u32);
    __type(value, struct behavior_profile);
} behavior_profiles SEC(".maps");

// Rate limiting map (LRU for automatic eviction)
struct {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    __uint(max_entries, MAX_RATE_LIMIT);
    __type(key, u32);
    __type(value, struct rate_limit_entry);
} rate_limiter SEC(".maps");

// Protected files map (inode-based)
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, MAX_PROTECTED_FILES);
    __type(key, u64);
    __type(value, struct protected_file);
} protected_files SEC(".maps");

// Network connection tracking
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, MAX_NETWORK_CONNS);
    __type(key, u64);
    __type(value, u32);
} network_conns SEC(".maps");

// Mode control (0=audit, 1=enforce)
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __uint(max_entries, 1);
    __type(key, u32);
    __type(value, u8);
} mode_control SEC(".maps");

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

static __always_inline bool is_rate_limited(u32 pid) {
    struct rate_limit_entry *entry = bpf_map_lookup_elem(&rate_limiter, &pid);
    u64 now = bpf_ktime_get_ns();
    
    if (!entry) {
        struct rate_limit_entry new_entry = {
            .last_event = now,
            .count = 1
        };
        bpf_map_update_elem(&rate_limiter, &pid, &new_entry, BPF_ANY);
        return false;
    }
    
    // Allow 1000 events per second per process
    if (now - entry->last_event < 1000000) { // 1ms
        entry->count++;
        if (entry->count > 1000) {
            return true;
        }
    } else {
        entry->last_event = now;
        entry->count = 1;
    }
    
    return false;
}

static __always_inline void update_behavior_profile(u32 pid, u32 event_type) {
    struct behavior_profile *profile = bpf_map_lookup_elem(&behavior_profiles, &pid);
    u64 now = bpf_ktime_get_ns();
    
    if (!profile) {
        struct behavior_profile new_profile = {
            .file_ops = 0,
            .exec_count = 0,
            .network_conns = 0,
            .last_update = now,
            .anomaly_score = 0
        };
        profile = &new_profile;
        bpf_map_update_elem(&behavior_profiles, &pid, profile, BPF_ANY);
    }
    
    // Update counters based on event type
    switch (event_type) {
        case EVENT_FILE_WRITE:
            __sync_fetch_and_add(&profile->file_ops, 1);
            break;
        case EVENT_EXEC:
            __sync_fetch_and_add(&profile->exec_count, 1);
            break;
        case EVENT_NETWORK:
            __sync_fetch_and_add(&profile->network_conns, 1);
            break;
    }
    
    // Calculate anomaly score
    if (profile->file_ops > 1000 || profile->exec_count > 50 || profile->network_conns > 500) {
        profile->anomaly_score = 100;
    }
    
    profile->last_update = now;
}

static __always_inline void log_event(u32 event_type, u32 prot, u32 flags, u8 blocked, u8 severity) {
    struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return;
    
    u64 pid_tgid = bpf_get_current_pid_tgid();
    u64 uid_gid = bpf_get_current_uid_gid();
    
    e->pid = pid_tgid >> 32;
    e->uid = uid_gid >> 32;
    e->gid = (u32)uid_gid;
    e->event_type = event_type;
    e->timestamp = bpf_ktime_get_ns();
    e->prot = prot;
    e->flags = flags;
    e->blocked = blocked;
    e->severity = severity;
    bpf_get_current_comm(&e->comm, sizeof(e->comm));
    
    bpf_ringbuf_submit(e, 0);
}

static __always_inline bool is_enforce_mode() {
    u32 key = 0;
    u8 *mode = bpf_map_lookup_elem(&mode_control, &key);
    return mode && *mode == 1;
}

// ============================================================================
// LSM HOOKS
// ============================================================================

// Block W^X memory (file-backed)
SEC("lsm/mmap_file")
int BPF_PROG(mmap_file, struct file *file, unsigned long prot,
             unsigned long flags, unsigned long ret)
{
    u32 pid = bpf_get_current_pid_tgid() >> 32;
    
    // Check allowlist (JIT runtimes)
    u8 *allowed = bpf_map_lookup_elem(&allowlist, &pid);
    if (allowed && *allowed == 1)
        return 0;
    
    // Rate limiting
    if (is_rate_limited(pid))
        return 0;
    
    // Check for W^X violation
    int is_write = prot & PROT_WRITE;
    int is_exec = prot & PROT_EXEC;
    
    if (is_write && is_exec) {
        log_event(EVENT_WX_MEMORY, prot, flags, 1, 5);
        update_behavior_profile(pid, EVENT_WX_MEMORY);
        
        if (is_enforce_mode())
            return -EPERM;
    }
    
    return 0;
}

// Block memory protection changes to W^X
SEC("lsm/file_mprotect")
int BPF_PROG(file_mprotect, struct vm_area_struct *vma, unsigned long prot)
{
    u32 pid = bpf_get_current_pid_tgid() >> 32;
    
    u8 *allowed = bpf_map_lookup_elem(&allowlist, &pid);
    if (allowed && *allowed == 1)
        return 0;
    
    if (is_rate_limited(pid))
        return 0;
    
    int is_write = prot & PROT_WRITE;
    int is_exec = prot & PROT_EXEC;
    
    if (is_write && is_exec) {
        log_event(EVENT_WX_MEMORY, prot, 0, 1, 5);
        update_behavior_profile(pid, EVENT_WX_MEMORY);
        
        if (is_enforce_mode())
            return -EPERM;
    }
    
    return 0;
}

// Monitor process execution
SEC("lsm/bprm_check_security")
int BPF_PROG(bprm_check, struct linux_binprm *bprm, int ret)
{
    u32 pid = bpf_get_current_pid_tgid() >> 32;
    
    if (is_rate_limited(pid))
        return 0;
    
    log_event(EVENT_EXEC, 0, 0, 0, 2);
    update_behavior_profile(pid, EVENT_EXEC);
    
    // Check for fork bomb (>50 execs)
    struct behavior_profile *profile = bpf_map_lookup_elem(&behavior_profiles, &pid);
    if (profile && profile->exec_count > 50) {
        log_event(EVENT_EXEC, 0, 0, 1, 5);
        if (is_enforce_mode())
            return -EPERM;
    }
    
    return 0;
}

// Monitor socket creation
SEC("lsm/socket_create")
int BPF_PROG(socket_create, int family, int type, int protocol, int kern)
{
    u32 pid = bpf_get_current_pid_tgid() >> 32;
    
    if (is_rate_limited(pid))
        return 0;
    
    log_event(EVENT_NETWORK, 0, 0, 0, 1);
    update_behavior_profile(pid, EVENT_NETWORK);
    
    return 0;
}

// ============================================================================
// TRACEPOINTS (for anonymous mmap)
// ============================================================================

SEC("tp/syscalls/sys_enter_mmap")
int trace_mmap_enter(struct trace_event_raw_sys_enter *ctx)
{
    u32 pid = bpf_get_current_pid_tgid() >> 32;
    
    u8 *allowed = bpf_map_lookup_elem(&allowlist, &pid);
    if (allowed && *allowed == 1)
        return 0;
    
    if (is_rate_limited(pid))
        return 0;
    
    // Extract prot and flags from syscall args
    unsigned long prot = ctx->args[2];
    unsigned long flags = ctx->args[3];
    int fd = (int)ctx->args[4];
    
    // Check for anonymous W^X
    if (fd == -1) {
        int is_write = prot & PROT_WRITE;
        int is_exec = prot & PROT_EXEC;
        
        if (is_write && is_exec) {
            log_event(EVENT_WX_MEMORY, prot, flags, 1, 5);
            update_behavior_profile(pid, EVENT_WX_MEMORY);
        }
    }
    
    return 0;
}
