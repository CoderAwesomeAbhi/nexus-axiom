# ❌ Why WSL2 Doesn't Work for Nexus Axiom

## The Core Problem

**WSL2 does NOT support eBPF LSM (Linux Security Module) hooks.**

---

## Technical Explanation

### What Nexus Axiom Needs

Nexus Axiom uses **eBPF LSM hooks** to block exploits:

```c
SEC("lsm/mmap_file")
int mmap_file(void *ctx) {
    // Block W^X memory BEFORE it's allocated
    if (is_wx) {
        return -EPERM;  // ← Kernel blocks the syscall
    }
}
```

**This requires:**
1. Linux kernel 5.8+
2. eBPF LSM enabled in kernel (`CONFIG_BPF_LSM=y`)
3. LSM framework configured with `lsm=bpf` boot parameter

---

## Why WSL2 Fails

### 1. **Microsoft's Custom Kernel**

WSL2 uses a **custom Microsoft kernel**, not the standard Linux kernel.

**Check WSL2 kernel:**
```bash
uname -r
# Output: 5.15.90.1-microsoft-standard-WSL2
#         ^^^^^^^^^ Microsoft's custom build
```

**Microsoft compiles this kernel WITHOUT:**
- `CONFIG_BPF_LSM=y` (eBPF LSM support)
- `CONFIG_SECURITY_BPF=y` (BPF security module)

### 2. **LSM Framework Not Configured**

Even if eBPF is available, the LSM framework doesn't include BPF:

```bash
cat /sys/kernel/security/lsm
# WSL2 output: capability,landlock,yama,apparmor
#              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
#              No "bpf" in the list!

# Real Linux output: capability,landlock,yama,apparmor,bpf
#                                                       ^^^
#                                                       This is missing in WSL2!
```

### 3. **Boot Parameters Can't Be Changed**

On real Linux, you can add `lsm=bpf` to grub:
```bash
# /etc/default/grub
GRUB_CMDLINE_LINUX="lsm=bpf"
```

**WSL2 doesn't use grub.** The kernel is loaded by Windows, and you can't modify boot parameters.

---

## What DOES Work in WSL2

### ✅ eBPF Compilation
```bash
make ebpf
# ✅ This works! Compiles to .o files
```

**Why:** Compilation only needs `clang` and `libbpf-dev`, which WSL2 has.

### ✅ Basic eBPF Programs
```bash
# Tracepoints work
SEC("tp/syscalls/sys_enter_openat")

# Kprobes work
SEC("kprobe/do_sys_open")

# XDP might work (network)
SEC("xdp")
```

**Why:** These don't require LSM framework.

### ❌ LSM Hooks
```bash
# These DON'T work
SEC("lsm/mmap_file")      ❌
SEC("lsm/file_open")      ❌
SEC("lsm/bprm_check")     ❌
```

**Why:** Require `CONFIG_BPF_LSM=y` which WSL2 doesn't have.

---

## The Exact Error You'd See

If you try to run Nexus Axiom on WSL2:

```bash
sudo ./target/release/nexus-axiom start
```

**Error:**
```
Error: Failed to load eBPF program
Caused by:
    libbpf: prog 'mmap_file': failed to attach to lsm/mmap_file
    libbpf: prog 'mmap_file': failed to attach: Operation not supported
```

**Translation:** "The kernel doesn't support LSM hooks for eBPF"

---

## Why Microsoft Doesn't Enable It

### 1. **Security Concerns**
eBPF LSM allows **kernel-level security enforcement**. Microsoft wants to control security at the Windows level, not inside WSL2.

### 2. **Compatibility**
Enabling eBPF LSM could break existing WSL2 applications that expect standard Linux behavior.

### 3. **Maintenance Burden**
Microsoft maintains a custom kernel. Adding eBPF LSM means more code to maintain and test.

### 4. **Windows Defender Integration**
Microsoft wants security to go through Windows Defender, not Linux-level tools.

---

## Proof: Check Kernel Config

```bash
# On WSL2
zcat /proc/config.gz | grep BPF_LSM
# Output: (nothing) or CONFIG_BPF_LSM is not set

# On real Linux with LSM support
zcat /proc/config.gz | grep BPF_LSM
# Output: CONFIG_BPF_LSM=y
```

---

## What You CAN Test on WSL2

### 1. **Compilation**
```bash
make ebpf
# ✅ Works - generates .o files
```

### 2. **Rust Code**
```bash
cargo build --release
# ✅ Works - compiles Rust binary
```

### 3. **Syntax Checking**
```bash
cargo check
cargo clippy
# ✅ Works - checks for errors
```

### 4. **Unit Tests**
```bash
cargo test
# ✅ Works - runs Rust tests
```

---

## What You CANNOT Test on WSL2

### 1. **Runtime Execution**
```bash
sudo ./target/release/nexus-axiom start
# ❌ Fails - can't attach LSM hooks
```

### 2. **Exploit Blocking**
```bash
./cve_tests/test_pwnkit
# ❌ Won't be blocked - LSM hooks not active
```

### 3. **Metrics from eBPF**
```bash
curl http://localhost:9090/metrics
# ❌ Shows 0 events - eBPF not running
```

### 4. **Real Security Enforcement**
```bash
# Any W^X memory allocation
# ❌ Won't be blocked - LSM not active
```

---

## The Solution: Use Real Linux

### Option 1: Oracle Cloud (FREE)
- Ubuntu 22.04 VM
- Full kernel control
- eBPF LSM support
- Free forever

### Option 2: AWS EC2 (FREE TIER)
- Ubuntu 22.04 t2.micro
- Full kernel control
- eBPF LSM support
- 750 hours/month free

### Option 3: Bare Metal Linux
- Dual boot Ubuntu
- Full control
- Best performance

### Option 4: VirtualBox/VMware
- Ubuntu 22.04 VM
- Full kernel control
- Works on your PC

---

## Technical Deep Dive

### How LSM Hooks Work

1. **Application calls syscall:**
```c
void *mem = mmap(NULL, size, PROT_READ|PROT_WRITE|PROT_EXEC, ...);
```

2. **Kernel enters mmap handler:**
```c
// In kernel: mm/mmap.c
SYSCALL_DEFINE6(mmap, ...) {
    // ... setup code ...
    
    // LSM hook called HERE ↓
    ret = security_mmap_file(file, prot, flags);
    if (ret)
        return ret;  // ← Our eBPF returns -EPERM here
    
    // ... actual allocation ...
}
```

3. **LSM framework calls eBPF:**
```c
// In kernel: security/security.c
int security_mmap_file(struct file *file, unsigned long prot, ...) {
    // Calls all registered LSM hooks
    // Including our eBPF program!
    return call_int_hook(mmap_file, 0, file, prot, flags);
}
```

4. **Our eBPF program runs:**
```c
SEC("lsm/mmap_file")
int mmap_file(void *ctx) {
    if (is_wx) {
        return -EPERM;  // ← Blocks the syscall
    }
    return 0;  // ← Allows it
}
```

### Why WSL2 Can't Do This

**WSL2's kernel doesn't have step 3!**

The `security_mmap_file()` function exists, but it doesn't call eBPF hooks because:
- `CONFIG_BPF_LSM=y` is not set
- BPF is not in the LSM list
- The hook infrastructure isn't compiled in

---

## Comparison Table

| Feature | WSL2 | Real Linux |
|---------|------|------------|
| **eBPF Compilation** | ✅ Yes | ✅ Yes |
| **eBPF Tracepoints** | ✅ Yes | ✅ Yes |
| **eBPF Kprobes** | ✅ Yes | ✅ Yes |
| **eBPF XDP** | ⚠️ Limited | ✅ Yes |
| **eBPF LSM Hooks** | ❌ No | ✅ Yes |
| **Kernel Config Control** | ❌ No | ✅ Yes |
| **Boot Parameters** | ❌ No | ✅ Yes |
| **Security Enforcement** | ❌ No | ✅ Yes |

---

## Can Microsoft Fix This?

**Technically: Yes**  
**Will they: Unlikely**

**What they'd need to do:**
1. Recompile WSL2 kernel with `CONFIG_BPF_LSM=y`
2. Add `bpf` to default LSM list
3. Test compatibility with all WSL2 features
4. Update Windows to load new kernel

**Why they won't:**
- Security concerns (gives users kernel-level control)
- Maintenance burden
- Conflicts with Windows Defender
- Low priority (niche use case)

---

## Workarounds (None Work Fully)

### ❌ Compile Custom WSL2 Kernel
**Problem:** Windows won't load it (signature verification)

### ❌ Use Docker in WSL2
**Problem:** Docker uses host kernel (still WSL2's kernel)

### ❌ Use Nested VM
**Problem:** Performance terrible, still limited

### ✅ Use Real Linux VM
**Solution:** Oracle Cloud, AWS, VirtualBox

---

## Bottom Line

**WSL2 is great for:**
- Development
- Testing compilation
- Running Linux tools
- Web development

**WSL2 is NOT for:**
- Kernel-level security tools
- eBPF LSM programs
- Low-level system programming
- Production security enforcement

**For Nexus Axiom, you MUST use real Linux.**

---

## Quick Test to Confirm

Run this on WSL2:
```bash
cat /sys/kernel/security/lsm
```

**If output doesn't contain "bpf"** → Won't work  
**If output contains "bpf"** → Will work (unlikely on WSL2)

---

## Summary

| Question | Answer |
|----------|--------|
| **Why doesn't WSL2 work?** | Microsoft's kernel lacks `CONFIG_BPF_LSM=y` |
| **Can I fix it?** | No, kernel is controlled by Microsoft |
| **What works on WSL2?** | Compilation, syntax checking, unit tests |
| **What doesn't work?** | Runtime execution, LSM hooks, exploit blocking |
| **What's the solution?** | Use Oracle Cloud, AWS, or bare metal Linux |
| **Will Microsoft fix it?** | Unlikely (security + maintenance concerns) |

---

**TL;DR:** WSL2 uses a custom Microsoft kernel that doesn't support eBPF LSM hooks. You need real Linux (Oracle Cloud free tier) to actually run and test Nexus Axiom.
