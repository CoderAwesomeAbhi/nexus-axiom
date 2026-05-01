# 🚀 UBUNTU VM - EXACT EXECUTION STEPS

**Status:** Ready to execute  
**Goal:** Get Nexus Axiom working, then add all features  
**Timeline:** Start NOW

---

## 📋 STEP-BY-STEP COMMANDS

### STEP 1: Initial Setup (5 minutes)

```bash
# SSH into your Ubuntu VM
ssh ubuntu@<YOUR_VM_IP>

# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Check kernel version (need 5.8+)
uname -r
# If less than 5.8, upgrade kernel first

# Check if LSM BPF is enabled
cat /sys/kernel/security/lsm
# Look for "bpf" in the output
```

**Expected:** You should see something like `lockdown,yama,integrity,apparmor`  
**Problem:** "bpf" is probably NOT there yet

---

### STEP 2: Enable LSM BPF (CRITICAL - 10 minutes)

```bash
# Edit grub configuration
sudo nano /etc/default/grub

# Find this line:
# GRUB_CMDLINE_LINUX=""

# Change it to:
# GRUB_CMDLINE_LINUX="lsm=lockdown,yama,integrity,apparmor,bpf"

# Save and exit (Ctrl+X, Y, Enter)

# Update grub
sudo update-grub

# Reboot
sudo reboot now

# Wait 2 minutes, then SSH back in
ssh ubuntu@<YOUR_VM_IP>

# Verify LSM BPF is now enabled
cat /sys/kernel/security/lsm | grep bpf
# Should see "bpf" in output now
```

**Expected:** Output includes "bpf"  
**If not:** Your kernel might not support LSM BPF (need 5.8+)

---

### STEP 3: Install All Dependencies (10 minutes)

```bash
# Install build tools
sudo apt-get install -y \
    clang \
    llvm \
    llvm-dev \
    libclang-dev \
    libbpf-dev \
    linux-headers-$(uname -r) \
    linux-tools-$(uname -r) \
    linux-tools-common \
    linux-tools-generic \
    build-essential \
    pkg-config \
    libelf-dev \
    zlib1g-dev \
    git \
    curl \
    wget \
    musl-tools \
    bpftool

# Verify bpftool works
bpftool version

# Check BTF is available
ls /sys/kernel/btf/vmlinux
# Should exist

# If BTF doesn't exist, you need a newer kernel
```

**Expected:** All packages install successfully  
**Problem:** If bpftool not found, install manually

---

### STEP 4: Install Rust (5 minutes)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Choose option 1 (default installation)

# Load Rust environment
source $HOME/.cargo/env

# Verify Rust installed
rustc --version
cargo --version

# Add musl target for static binaries
rustup target add x86_64-unknown-linux-musl

# Verify target added
rustup target list | grep musl
```

**Expected:** Rust 1.70+ installed  
**Problem:** If fails, check internet connection

---

### STEP 5: Clone Your Repository (2 minutes)

```bash
# Create workspace
mkdir -p ~/projects
cd ~/projects

# Clone from Windows machine
# Option A: If you have GitHub repo
git clone https://github.com/YOUR_USERNAME/nexus-axiom
cd nexus-axiom

# Option B: If not on GitHub yet, copy from Windows
# On Windows:
# scp -r C:\Users\abhij\nexus-axiom-viral ubuntu@<VM_IP>:~/projects/nexus-axiom

# Verify files are there
ls -la
# Should see: Cargo.toml, src/, ebpf/, examples/, etc.
```

**Expected:** All files copied  
**Problem:** If missing files, copy them

---

### STEP 6: Generate vmlinux.h (CRITICAL - 5 minutes)

```bash
cd ~/projects/nexus-axiom

# Check if vmlinux.h already exists
ls ebpf/vmlinux.h

# If NOT exists, generate it
bpftool btf dump file /sys/kernel/btf/vmlinux format c > ebpf/vmlinux.h

# Verify it was created
ls -lh ebpf/vmlinux.h
# Should be ~5MB

# Check first few lines
head -20 ebpf/vmlinux.h
# Should see C struct definitions
```

**Expected:** vmlinux.h created, ~5MB  
**Problem:** If BTF not available, kernel too old

---

### STEP 7: First Build Attempt (THE MOMENT OF TRUTH - 10 minutes)

```bash
cd ~/projects/nexus-axiom

# Try to build
cargo build --release 2>&1 | tee build.log

# This will probably take 5-10 minutes
# Watch for errors
```

**Expected Outcomes:**

**SUCCESS:** 
```
Finished release [optimized] target(s) in 8m 32s
```

**FAILURE (Common Errors):**

#### Error 1: "vmlinux.h not found"
```bash
# Already fixed in Step 6
# If still failing, check path in nexus.bpf.c
grep "vmlinux.h" ebpf/nexus.bpf.c
# Should be: #include "vmlinux.h"
```

#### Error 2: "libbpf-sys build failed"
```bash
# Install missing dependencies
sudo apt-get install -y pkg-config libelf-dev zlib1g-dev

# Try again
cargo clean
cargo build --release
```

#### Error 3: "cannot find -lbpf"
```bash
# Install libbpf
sudo apt-get install -y libbpf-dev

# Try again
cargo build --release
```

#### Error 4: "LSM hooks not found"
```bash
# Check kernel version
uname -r
# Need 5.8+

# Check LSM BPF enabled
cat /sys/kernel/security/lsm | grep bpf
# Must see "bpf"
```

---

### STEP 8: If Build Succeeds - Test It! (5 minutes)

```bash
# Build succeeded! Binary is here:
ls -lh target/release/nexus-axiom

# Test help command
./target/release/nexus-axiom --help

# Build examples
cd examples
make clean
make
cd ..

# THE CRITICAL TEST: Start Nexus Axiom
sudo ./target/release/nexus-axiom start --audit

# You should see:
# 🛡️  NEXUS AXIOM v1.0.0
# ✅ eBPF LSM hooks loaded
# ✅ Mode: AUDIT (logs only)
```

**If you see this:** IT WORKS! 🎉

---

### STEP 9: Test Exploit Blocking (THE PROOF - 5 minutes)

```bash
# Open a second SSH session
ssh ubuntu@<YOUR_VM_IP>

cd ~/projects/nexus-axiom/examples

# Test 1: W^X memory
./test_wx_memory

# Expected output in first terminal:
# [CRITICAL] 🔴 BLOCKED | PID: XXXX | Process: test_wx_memory
# 💀 EXPLOIT TERMINATED 💀

# Test 2: PwnKit
./test_pwnkit

# Expected: Same blocking message

# Test 3: mprotect
./test_mprotect

# Expected: Same blocking message
```

**If all 3 are blocked:** YOU DID IT! Core functionality works! 🚀

---

### STEP 10: Switch to Enforce Mode (THE REAL TEST - 2 minutes)

```bash
# Stop audit mode (Ctrl+C in first terminal)

# Start enforce mode
sudo ./target/release/nexus-axiom start

# In second terminal, try exploit
./test_wx_memory

# Expected: Process should be KILLED
# Output: "Killed" or nothing (process terminated)

# Check dmesg for kernel messages
sudo dmesg | tail -20
# Should see Nexus Axiom blocking messages
```

**If process is killed:** FULL SUCCESS! 🎉🎉🎉

---

## 🎯 WHAT TO DO NEXT (After Core Works)

### Immediate (Same Day):

```bash
# 1. Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# 2. Test static binary on fresh system
scp target/x86_64-unknown-linux-musl/release/nexus-axiom ubuntu2@<OTHER_VM>:/tmp/
ssh ubuntu2@<OTHER_VM>
/tmp/nexus-axiom --help  # Should work with zero dependencies!

# 3. Run benchmarks
cd examples
sudo ./benchmark.sh

# 4. Document real results
# Update README.md with actual numbers
```

### Tomorrow (Day 2):

```bash
# 1. Add 5 more CVE tests
cd examples
nano test_sudo_overflow.c  # Create new tests
nano test_dirty_pipe.c
nano test_jit_spray.c
nano test_rop_chain.c
nano test_fork_bomb.c

# Update Makefile
nano Makefile

# Build all
make clean && make

# Test all 8
./run_exploit_zoo.sh
# Goal: 8/8 exploits blocked
```

### Week 1 (Days 3-7):

```bash
# 1. Implement process ancestry tracking
# Edit ebpf/nexus.bpf.c - add parent tracking

# 2. Add Prometheus metrics
# Edit src/main.rs - add /metrics endpoint

# 3. Set up kernel matrix CI/CD
# Edit .github/workflows/kernel-matrix.yml

# 4. Test on multiple distros
# Spin up Fedora VM, test install.sh
# Spin up Debian VM, test install.sh
```

### Week 2 (Days 8-14):

```bash
# 1. Implement forensics recorder (Black Box)
# 2. Add drift detection (Behavioral Identity)
# 3. Implement honeypot redirection (Mirror Hook)
# 4. Add virtual patching (Policy Snippets)
# 5. Create WASM plugin system
# 6. Add network attribution
# 7. Implement blast radius quarantine
# 8. Create GitHub Action
# 9. Add compliance profiles
# 10. Implement panic button
```

---

## 🚨 TROUBLESHOOTING GUIDE

### Problem: Build fails with "vmlinux.h not found"
```bash
# Solution:
bpftool btf dump file /sys/kernel/btf/vmlinux format c > ebpf/vmlinux.h
```

### Problem: "LSM BPF not enabled"
```bash
# Solution:
sudo nano /etc/default/grub
# Add: lsm=lockdown,yama,integrity,apparmor,bpf
sudo update-grub && sudo reboot
```

### Problem: "Kernel too old"
```bash
# Check version
uname -r

# If < 5.8, upgrade kernel:
sudo apt-get install -y linux-generic-hwe-22.04
sudo reboot
```

### Problem: "libbpf-sys build failed"
```bash
# Install dependencies:
sudo apt-get install -y pkg-config libelf-dev zlib1g-dev libbpf-dev
cargo clean && cargo build --release
```

### Problem: Exploits not being blocked
```bash
# Check if eBPF programs loaded:
sudo bpftool prog list | grep nexus

# Check if LSM hooks attached:
sudo bpftool prog list | grep lsm

# Check dmesg for errors:
sudo dmesg | tail -50

# Verify mode is enforce:
# Should see "Mode: ENFORCE" when starting
```

---

## 📊 SUCCESS CHECKLIST

After completing all steps, you should have:

- [x] Ubuntu VM with LSM BPF enabled
- [x] All dependencies installed
- [x] Rust + musl target installed
- [x] Repository cloned
- [x] vmlinux.h generated
- [x] Nexus Axiom builds successfully
- [x] Examples compile
- [x] Audit mode works
- [x] Enforce mode works
- [x] 3/3 exploits blocked
- [x] Static binary builds
- [x] Benchmarks run

**If all checked:** You're ready for Week 2 features! 🚀

---

## 🎯 IMMEDIATE COMMANDS (Copy-Paste)

```bash
# Run this entire block:
sudo apt-get update && \
sudo apt-get install -y clang llvm libbpf-dev linux-headers-$(uname -r) \
    build-essential pkg-config libelf-dev zlib1g-dev git curl musl-tools \
    linux-tools-$(uname -r) && \
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
source $HOME/.cargo/env && \
rustup target add x86_64-unknown-linux-musl && \
echo "✅ All dependencies installed!"

# Then check LSM BPF:
cat /sys/kernel/security/lsm | grep bpf

# If "bpf" not found, run:
sudo sed -i 's/GRUB_CMDLINE_LINUX=""/GRUB_CMDLINE_LINUX="lsm=lockdown,yama,integrity,apparmor,bpf"/' /etc/default/grub && \
sudo update-grub && \
echo "⚠️  Reboot required: sudo reboot"
```

---

**START WITH THESE COMMANDS RIGHT NOW.** 🚀

**Report back:**
1. Did LSM BPF enable successfully?
2. Did the build succeed?
3. Did the exploits get blocked?

**Then we add ALL the advanced features.** 💎
