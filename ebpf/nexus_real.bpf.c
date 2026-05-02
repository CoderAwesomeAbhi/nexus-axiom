// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - Minimal eBPF LSM Implementation

// Type definitions
typedef unsigned char __u8;
typedef unsigned int __u32;
typedef unsigned long long __u64;

// BPF helper function declarations
static long (*bpf_get_current_pid_tgid)(void) = (void *) 14;
static long (*bpf_get_current_uid_gid)(void) = (void *) 15;
static long (*bpf_get_current_comm)(void *buf, __u32 size) = (void *) 16;
static long (*bpf_ktime_get_ns)(void) = (void *) 5;
static void *(*bpf_map_lookup_elem)(void *map, const void *key) = (void *) 1;
static void *(*bpf_ringbuf_reserve)(void *ringbuf, __u64 size, __u64 flags) = (void *) 131;
static void (*bpf_ringbuf_submit)(void *data, __u64 flags) = (void *) 132;

#define SEC(name) __attribute__((section(name), used))
#define BPF_MAP_TYPE_RINGBUF 27
#define BPF_MAP_TYPE_HASH 1

#define __uint(name, val) int (*name)[val]
#define __type(name, val) typeof(val) *name

char LICENSE[] SEC("license") = "GPL";

// Ring buffer for events
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 20);
} events SEC(".maps");

// Allowlist map
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1000);
    __type(key, __u32);
    __type(value, __u8);
} allowlist SEC(".maps");

// Event structure
struct event {
    __u32 pid;
    __u32 uid;
    __u64 timestamp;
    __u32 prot;
    __u32 flags;
    __u8 blocked;
    char comm[16];
};

#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

// Simple LSM hook for mmap_file
SEC("lsm/mmap_file")
int mmap_file(void *ctx)
{
    __u64 pid_tgid = bpf_get_current_pid_tgid();
    __u32 pid = pid_tgid >> 32;
    
    // Check allowlist
    __u8 *allowed = bpf_map_lookup_elem(&allowlist, &pid);
    if (allowed && *allowed == 1)
        return 0;
    
    // For now, just log all mmap attempts
    // In production, would parse ctx to get prot flags
    struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (e) {
        e->pid = pid;
        e->uid = bpf_get_current_uid_gid() >> 32;
        e->timestamp = bpf_ktime_get_ns();
        e->prot = 0;
        e->flags = 0;
        e->blocked = 0;
        bpf_get_current_comm(&e->comm, sizeof(e->comm));
        bpf_ringbuf_submit(e, 0);
    }
    
    return 0;
}
