// SPDX-License-Identifier: GPL-2.0
// nexus_working.bpf.c — Minimal eBPF LSM that ACTUALLY blocks W^X memory.
//
// KEY FIXES vs nexus_real.bpf.c:
//   1. Proper headers: <linux/bpf.h>, <bpf/bpf_helpers.h>, <bpf/bpf_tracing.h>
//   2. BPF_PROG macro for typed argument access (not raw ctx indexing)
//   3. -EPERM return value (explicit, not -1)
//   4. Ringbuffer + Event struct for userspace notification

#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char LICENSE[] SEC("license") = "GPL";

#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

#define EVENT_TYPE_MMAP     1
#define EVENT_TYPE_MPROTECT 4

// ── Event struct — MUST match Rust exactly ──────────────────────────────────
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

// ── Maps ─────────────────────────────────────────────────────────────────────
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 20); // 1MB
} events SEC(".maps");

// ── Helper: fill and submit an event ─────────────────────────────────────────
static __always_inline void emit_event(__u32 prot, __u32 flags, __u8 event_type)
{
    struct event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return;

    __u64 pid_tgid = bpf_get_current_pid_tgid();
    __u64 uid_gid  = bpf_get_current_uid_gid();

    e->pid        = (__u32)(pid_tgid >> 32);
    e->uid        = (__u32)uid_gid;
    e->timestamp  = bpf_ktime_get_ns();
    e->prot       = prot;
    e->flags      = flags;
    e->blocked    = 1;
    e->event_type = event_type;
    e->_pad[0]    = 0;
    e->_pad[1]    = 0;
    e->cgroup_id  = bpf_get_current_cgroup_id();
    bpf_get_current_comm(e->comm, sizeof(e->comm));

    bpf_ringbuf_submit(e, 0);
}

// ── LSM hook: mmap_file ──────────────────────────────────────────────────────
SEC("lsm/mmap_file")
int BPF_PROG(lsm_mmap_file,
             struct file *file,
             unsigned long reqprot,
             unsigned long prot,
             unsigned long flags)
{
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        emit_event((__u32)prot, (__u32)flags, EVENT_TYPE_MMAP);
        return -EPERM;
    }
    return 0;
}

// ── LSM hook: file_mprotect ──────────────────────────────────────────────────
SEC("lsm/file_mprotect")
int BPF_PROG(lsm_file_mprotect,
             struct vm_area_struct *vma,
             unsigned long reqprot,
             unsigned long prot)
{
    if ((prot & PROT_WRITE) && (prot & PROT_EXEC)) {
        emit_event((__u32)prot, 0, EVENT_TYPE_MPROTECT);
        return -EPERM;
    }
    return 0;
}
