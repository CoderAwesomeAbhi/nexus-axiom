// SPDX-License-Identifier: GPL-2.0
// Minimal Working eBPF LSM - Blocks W^X Memory

#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char LICENSE[] SEC("license") = "GPL";

#define PROT_READ  0x1
#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

// Simple event structure
struct event {
    __u32 pid;
    __u32 prot;
    __u8 blocked;
};

// Ringbuffer for events
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024);
} events SEC(".maps");

// LSM hook for mmap - Block W^X
SEC("lsm/mmap_file")
int BPF_PROG(mmap_file, struct file *file, unsigned long reqprot,
             unsigned long prot, unsigned long flags)
{
    __u64 pid_tgid = bpf_get_current_pid_tgid();
    __u32 pid = pid_tgid >> 32;
    
    // Check if W^X
    int is_wx = (prot & PROT_WRITE) && (prot & PROT_EXEC);
    
    if (is_wx) {
        // Log event
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->prot = prot;
            e->blocked = 1;
            bpf_ringbuf_submit(e, 0);
        }
        
        // Block it!
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
    
    // Check if W^X
    int is_wx = (prot & PROT_WRITE) && (prot & PROT_EXEC);
    
    if (is_wx) {
        // Log event
        struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
        if (e) {
            e->pid = pid;
            e->prot = prot;
            e->blocked = 1;
            bpf_ringbuf_submit(e, 0);
        }
        
        // Block it!
        return -EPERM;
    }
    
    return 0;
}
