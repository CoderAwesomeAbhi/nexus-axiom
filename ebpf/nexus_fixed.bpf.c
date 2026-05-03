// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - FIXED VERSION - Actually blocks W^X

#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char LICENSE[] SEC("license") = "GPL";

#define PROT_READ  0x1
#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

// Event structure
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
    char  comm[16];
};

// Ringbuffer
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 20);
} events SEC(".maps");

// Allowlist
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1024);
    __type(key, __u32);
    __type(value, __u8);
} allowlist SEC(".maps");

// LSM hook for mmap - Block W^X
SEC("lsm/mmap_file")
int BPF_PROG(mmap_file, struct file *file, unsigned long reqprot,
             unsigned long prot, unsigned long flags)
{
    __u64 pid_tgid = bpf_get_current_pid_tgid();
    __u32 pid = pid_tgid >> 32;
    
    // Check allowlist
    __u8 *is_allowed = bpf_map_lookup_elem(&allowlist, &pid);
    if (is_allowed && *is_allowed == 1)
        return 0;
    
    // Check if W^X
    int is_wx = (prot & PROT_WRITE) && (prot & PROT_EXEC);
    
    if (is_wx) {
        // Log event
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->uid = bpf_get_current_uid_gid() >> 32;
            e->timestamp = bpf_ktime_get_ns();
            e->prot = prot;
            e->flags = flags;
            e->blocked = 1;
            e->event_type = 1;
            e->_pad[0] = 0;
            e->_pad[1] = 0;
            e->cgroup_id = bpf_get_current_cgroup_id();
            bpf_get_current_comm(&e->comm, sizeof(e->comm));
            bpf_ringbuf_submit(e, 0);
        }
        
        // BLOCK IT!
        return -EPERM;
    }
    
    return 0;
}

// LSM hook for mprotect - Block W^X
SEC("lsm/file_mprotect")
int BPF_PROG(file_mprotect, struct vm_area_struct *vma,
             unsigned long reqprot, unsigned long prot)
{
    __u64 pid_tgid = bpf_get_current_pid_tgid();
    __u32 pid = pid_tgid >> 32;
    
    // Check allowlist
    __u8 *is_allowed = bpf_map_lookup_elem(&allowlist, &pid);
    if (is_allowed && *is_allowed == 1)
        return 0;
    
    // Check if W^X
    int is_wx = (prot & PROT_WRITE) && (prot & PROT_EXEC);
    
    if (is_wx) {
        // Log event
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->uid = bpf_get_current_uid_gid() >> 32;
            e->timestamp = bpf_ktime_get_ns();
            e->prot = prot;
            e->flags = 0;
            e->blocked = 1;
            e->event_type = 4;
            e->_pad[0] = 0;
            e->_pad[1] = 0;
            e->cgroup_id = bpf_get_current_cgroup_id();
            bpf_get_current_comm(&e->comm, sizeof(e->comm));
            bpf_ringbuf_submit(e, 0);
        }
        
        // BLOCK IT!
        return -EPERM;
    }
    
    return 0;
}
