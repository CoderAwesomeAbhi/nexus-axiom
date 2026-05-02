// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - eBPF LSM with W^X Detection

typedef unsigned char __u8;
typedef unsigned int __u32;
typedef unsigned long long __u64;

// BPF helpers
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

// Maps
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 20);
} events SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1000);
    __type(key, __u32);
    __type(value, __u8);
} allowlist SEC(".maps");

// Event structure - MUST match Rust exactly
struct event {
    __u32 pid;
    __u32 uid;
    __u64 timestamp;
    __u32 prot;      // Protection flags
    __u32 flags;     // mmap flags
    __u8 blocked;    // 1 if should be blocked
    char comm[16];
};

#define PROT_READ  0x1
#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

// LSM hook for mmap - Block W^X in kernel and send events to userspace
SEC("lsm/mmap_file")
int mmap_file(void *ctx)
{
    unsigned long *args = ctx;
    // args[0] is struct file *file
    // args[1] is reqprot
    __u32 prot = (__u32)args[2];
    __u32 flags = (__u32)args[3];

    __u64 pid_tgid = bpf_get_current_pid_tgid();
    __u32 pid = pid_tgid >> 32;
    
    // Check allowlist
    __u8 *is_allowed = bpf_map_lookup_elem(&allowlist, &pid);
    if (is_allowed && *is_allowed == 1) {
        return 0; // Allow
    }

    int is_wx = (prot & PROT_WRITE) && (prot & PROT_EXEC);

    struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (e) {
        e->pid = pid;
        e->uid = bpf_get_current_uid_gid() >> 32;
        e->timestamp = bpf_ktime_get_ns();
        
        e->prot = prot;
        e->flags = flags;
        e->blocked = is_wx ? 1 : 0;
        
        bpf_get_current_comm(&e->comm, sizeof(e->comm));
        bpf_ringbuf_submit(e, 0);
    }
    
    // Block the W^X mmap request directly in the kernel by returning -EPERM
    if (is_wx) {
        return -1; // -EPERM
    }
    
    return 0;
}

// LSM hook for execution control - Block unauthorized binaries
SEC("lsm/bprm_check_security")
int bprm_check_security(void *ctx)
{
    // args[0] is struct linux_binprm *bprm
    
    __u64 pid_tgid = bpf_get_current_pid_tgid();
    __u32 pid = pid_tgid >> 32;

    // We can use a map of blocked inodes/hashes, or simply alert on suspicious names.
    // For demonstration of genuine capability, we check if the process is explicitly blocked in a map.
    // Re-using allowlist for simplicity: if value is 2, it's blocked.
    __u8 *status = bpf_map_lookup_elem(&allowlist, &pid);
    if (status && *status == 2) {
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->uid = bpf_get_current_uid_gid() >> 32;
            e->timestamp = bpf_ktime_get_ns();
            e->prot = 0;
            e->flags = 0;
            e->blocked = 1;
            bpf_get_current_comm(&e->comm, sizeof(e->comm));
            bpf_ringbuf_submit(e, 0);
        }
        return -1; // -EPERM
    }

    return 0;
}
