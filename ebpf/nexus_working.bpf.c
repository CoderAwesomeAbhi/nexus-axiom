// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - Simple eBPF LSM that blocks W^X

#include <linux/bpf.h>
#include <linux/errno.h>
#include <bpf/bpf_helpers.h>

char LICENSE[] SEC("license") = "GPL";

#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

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
