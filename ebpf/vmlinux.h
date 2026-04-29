/* Minimal vmlinux.h for eBPF compilation */
#ifndef __VMLINUX_H__
#define __VMLINUX_H__

typedef unsigned char __u8;
typedef unsigned short __u16;
typedef unsigned int __u32;
typedef unsigned long long __u64;

typedef signed char __s8;
typedef signed short __s16;
typedef signed int __s32;
typedef signed long long __s64;

typedef __u8 u8;
typedef __u16 u16;
typedef __u32 u32;
typedef __u64 u64;

typedef __u16 __be16;
typedef __u32 __be32;
typedef __u32 __wsum;

#define PROT_READ  0x1
#define PROT_WRITE 0x2
#define PROT_EXEC  0x4

#define BPF_MAP_TYPE_HASH 1
#define BPF_MAP_TYPE_RINGBUF 27

#define EPERM 1
#define EACCES 13

struct file;
struct linux_binprm;
struct vm_area_struct;

// Tracepoint context for syscall entry
struct trace_event_raw_sys_enter {
    u64 __unused__;
    long id;
    unsigned long args[6];
};

#endif /* __VMLINUX_H__ */
