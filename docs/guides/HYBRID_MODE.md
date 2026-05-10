# 🚀 NEXUS AXIOM HYBRID MODE

## 🎯 GAME-CHANGING ARCHITECTURE

Nexus Axiom is the **ONLY** security tool with both privileged and unprivileged modes.

---

## 🔧 TWO MODES

### Mode 1: eBPF LSM (Default)
**Requires:** Root privileges
**Protects:** All processes system-wide
**How:** Kernel-level LSM hooks

```bash
sudo nexus-axiom start
```

### Mode 2: seccomp User Notification (Unprivileged)
**Requires:** No root (apps opt-in)
**Protects:** Apps that voluntarily connect
**How:** seccomp SECCOMP_RET_USER_NOTIF

```bash
# No sudo needed!
nexus-axiom start --unprivileged

# Apps opt-in:
LD_PRELOAD=libnexus-client.so ./myapp
```

---

## ✅ ADVANTAGES

### eBPF Mode:
- ✅ System-wide protection
- ✅ No app changes needed
- ✅ Blocks at kernel level

### Unprivileged Mode:
- ✅ No root required
- ✅ Works in containers without privileges
- ✅ Apps can self-protect

---

## 🏆 WHY THIS IS UNIQUE

**No other tool offers both:**
- Falco: eBPF only (requires root)
- Tetragon: eBPF only (requires root)
- AppArmor: Kernel only (requires root)
- SELinux: Kernel only (requires root)

**Nexus Axiom: Both modes**

---

## 🔧 HOW IT WORKS

### Unprivileged Mode Architecture:

```
App Process
    ↓
seccomp filter installed
    ↓
Dangerous syscall (mprotect W^X)
    ↓
SECCOMP_RET_USER_NOTIF
    ↓
Nexus Axiom daemon (no root!)
    ↓
Evaluate policy
    ↓
Return: ALLOW or DENY
    ↓
Syscall proceeds or fails
```

### Client Library:
```c
// libnexus-client.so
__attribute__((constructor))
void nexus_init() {
    // Install seccomp filter
    // Connect to Nexus daemon
    // Hand over syscall control
}
```

---

## 📊 USE CASES

### eBPF Mode:
- Production servers
- Kubernetes nodes
- System-wide protection

### Unprivileged Mode:
- Developer workstations
- CI/CD pipelines
- Containers without privileges
- Sandboxed apps

---

## 🚀 IMPLEMENTATION STATUS

- [x] eBPF mode (working)
- [ ] seccomp mode (in progress)
- [ ] Client library (in progress)
- [ ] Hybrid mode switching (in progress)

---

## 🎯 COMPETITIVE ADVANTAGE

**This makes Nexus Axiom the ONLY tool that works in:**
- ✅ Privileged environments (eBPF)
- ✅ Unprivileged environments (seccomp)
- ✅ Both simultaneously

**Result:** Broader adoption, unique differentiator, 5k+ stars.
