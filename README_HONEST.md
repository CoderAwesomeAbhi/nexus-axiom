# 🛡️ Nexus Axiom - eBPF LSM Security (HONEST VERSION)

**Real eBPF LSM Implementation | W^X Memory Blocking | Open Source**

---

## 🎯 Current State (Brutal Honesty)

### ✅ What Actually Works RIGHT NOW
1. **Real eBPF LSM Hook** - `ebpf/nexus_real.bpf.c` blocks W^X memory at kernel level
2. **Rust Userspace Loader** - Loads eBPF programs and processes events
3. **15-Second Demo** - Shows actual exploit blocking (on Linux)
4. **Emergency Unload** - Safe way to remove hooks if something breaks

### 🚧 What's Currently Simulated (Being Built)
- Quantum-resistant crypto (framework exists, needs real Dilithium/Kyber)
- Neural network inference (needs actual ONNX runtime integration)
- Blockchain audit (local Merkle tree works, Ethereum integration pending)
- Cloud features (architecture designed, AWS/K8s integration in progress)

### 📊 Realistic Star Projection
- **Week 1**: 50-200 stars (if we're honest about current state)
- **Month 1**: 500-1K stars (with real eBPF demos working)
- **Month 3**: 5K+ stars (with production-ready features)
- **Month 6**: 10K+ stars (if we become the best at ONE thing)

---

## 🚀 Quick Start (Linux Only)

### Prerequisites
```bash
# Ubuntu/Debian
sudo apt-get install clang llvm libbpf-dev linux-headers-$(uname -r)

# Check if your kernel supports eBPF LSM
cat /sys/kernel/security/lsm | grep bpf
```

### Build
```bash
cargo build --release
```

### Run Demo
```bash
# See the 15-second exploit demo
chmod +x demo.sh
sudo ./demo.sh

# Or run directly
sudo ./target/release/nexus-axiom start --audit-only
```

---

## 🔥 What Makes This Different

### We're Honest About What Works
- ✅ Real eBPF LSM hooks (not just tracepoints)
- ✅ Actual W^X memory blocking (returns -EPERM to kernel)
- ✅ Ring buffer event streaming (1MB, zero-copy)
- ⚠️  AI/ML features are userspace-only (eBPF can't run neural nets)
- ⚠️  Quantum crypto is framework code (needs real PQC library integration)

### Our Focus: Do ONE Thing Perfectly
**Goal**: Be the absolute best open-source eBPF LSM tool for W^X enforcement.

Once that's rock-solid, we'll add:
1. More LSM hooks (file access, network, etc.)
2. Real post-quantum signatures (Dilithium)
3. Local ML inference (quantized models)
4. Immutable audit logs (Merkle tree + Sigstore)

---

## 📖 Architecture

### What Actually Runs in eBPF
```c
// ebpf/nexus_real.bpf.c
SEC("lsm/mmap_file")
int BPF_PROG(mmap_file, ...) {
    if (is_write && is_exec) {
        return -EPERM;  // ACTUALLY blocks at kernel level
    }
}
```

### What Runs in Userspace
- Event processing from ring buffer
- AI/ML inference (ONNX)
- Cryptographic operations
- Audit log management
- Dashboard/metrics

---

## 🎬 15-Second Demo

```bash
$ sudo ./demo.sh

📝 Step 1: Creating test exploit (W^X memory allocation)...
  ✅ Exploit compiled

📝 Step 2: Running exploit WITHOUT Nexus Axiom...
[EXPLOIT] ✅ Success! Got W^X memory
[EXPLOIT] 🚨 System is vulnerable!

📝 Step 3: Starting Nexus Axiom protection...
  🔧 Loading eBPF LSM hooks...
  ✅ mmap_file hook attached

📝 Step 4: Running exploit WITH Nexus Axiom...
[EXPLOIT] ❌ BLOCKED by kernel!
[EXPLOIT] errno: Operation not permitted

✅ DEMO COMPLETE
```

---

## 🏆 Competitive Comparison (Honest)

| Feature | Nexus Axiom | Falco | Tetragon |
|---------|-------------|-------|----------|
| **LSM Enforcement** | ✅ Yes | ❌ Monitor only | ❌ Monitor only |
| **W^X Blocking** | ✅ Working | ❌ | ❌ |
| **Production Ready** | ⚠️  Alpha | ✅ Yes | ✅ Yes |
| **Total Features** | 5 real, 145 planned | ~20 | ~25 |
| **Maturity** | New (2026) | Mature (2016) | Mature (2022) |

**Our Advantage**: We actually BLOCK attacks, not just monitor them.

**Their Advantage**: They're battle-tested in production.

---

## 🛠️ Development Roadmap

### Phase 1: eBPF Core (Current)
- [x] W^X memory blocking (LSM mmap_file)
- [x] Process execution monitoring (LSM bprm_check)
- [x] Ring buffer event streaming
- [ ] File access control (LSM file_open)
- [ ] Network filtering (LSM socket_connect)
- [ ] BTF/CO-RE for kernel compatibility

### Phase 2: Real Crypto (Next)
- [ ] Integrate actual Dilithium signatures
- [ ] Hybrid crypto (X25519 + Kyber)
- [ ] TPM integration for key storage
- [ ] Sigstore/Rekor for audit logs

### Phase 3: Local AI (Future)
- [ ] Quantized ONNX models (INT8)
- [ ] Syscall sequence analysis
- [ ] Anomaly detection with SHAP explanations
- [ ] Federated learning with differential privacy

### Phase 4: Production Hardening
- [ ] Comprehensive test suite
- [ ] Fuzzing with AFL++
- [ ] Security audit by Trail of Bits
- [ ] Performance benchmarks vs Falco/Tetragon

---

## 🤝 Contributing

We need help with:
- [ ] Real Dilithium/Kyber integration
- [ ] ONNX runtime integration
- [ ] More eBPF LSM hooks
- [ ] Kernel compatibility testing
- [ ] Documentation

**Join us**: We're building this in public, honestly.

---

## ⚠️  Warnings

### Do NOT Use in Production Yet
This is alpha software. It WILL have bugs. Start with `--audit-only`.

### Requires Root
eBPF LSM hooks require CAP_BPF or root. This is a kernel-level tool.

### Linux Only
eBPF LSM is Linux-specific. No Windows/macOS support planned.

### May Break Things
If something goes wrong: `sudo ./nexus-axiom unload`

---

## 📜 License

GPL-3.0 - eBPF code must remain open source per kernel requirements.

---

## 🙏 Acknowledgments

**Inspired by**:
- Falco (monitoring architecture)
- Tetragon (eBPF tracing)
- gVisor (W^X enforcement)

**Built with**:
- eBPF/LSM (Linux kernel)
- Rust (userspace daemon)
- libbpf (eBPF loading)

---

## 📞 Contact

- **Issues**: GitHub Issues
- **Security**: security@nexus-axiom.dev (PGP key in repo)
- **Discussions**: GitHub Discussions

---

<div align="center">

**⭐ Star us if you believe in honest, open-source security ⭐**

**We're building this the right way: one real feature at a time.**

</div>
