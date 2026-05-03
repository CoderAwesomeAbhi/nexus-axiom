// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - Simple eBPF LSM that blocks W^X

#include <linux/bpf.h>
#include <linux/errno.h>
#include <bpf/bpf_helpers.h>

char LICENSE[] SEC("license") = "GPL";

#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

// Event struct
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

// LSM hook: mmap_file - Block W^X mmap
SEC("lsm/mmap_file")
int mmap_file_hook(void *ctx)
{
    unsigned long *args = ctx;
    unsigned long prot = args[2];
    
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        return -EPERM;
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
        return -EPERM;
    }
    return 0;
}
