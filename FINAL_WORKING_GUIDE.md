# Nexus Axiom — Final Working Guide

> **CRITICAL:** This project requires **real Linux** with eBPF LSM support.
> WSL2 will NOT work (Microsoft's kernel lacks `CONFIG_BPF_LSM=y`).
> Use Ubuntu 22.04+ on bare metal, VirtualBox, or a cloud VM.

---

## Prerequisites

Before anything else, verify your environment:

```bash
# Must show "bpf" in the list
cat /sys/kernel/security/lsm
# Expected: capability,landlock,yama,apparmor,bpf

# Must be 5.8+
uname -r

# Must be root for runtime steps
whoami  # should be root, or use sudo
```

If `bpf` is missing from `/sys/kernel/security/lsm`, jump to [Troubleshooting](#troubleshooting) before continuing.

---

## Step 1 — Pull Latest Code

```bash
# Clone (first time)
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom.git
cd nexus-axiom

# Or pull updates (existing clone)
cd nexus-axiom-final
git pull origin main
```

Verify you have the right source file:

```bash
ls ebpf/nexus_working.bpf.c   # must exist
ls ebpf/run_test.sh            # must exist
ls run.sh                      # must exist
```

---

## Step 2 — Compile

Install dependencies first (one-time):

```bash
sudo apt update
sudo apt install -y \
    clang llvm \
    gcc \
    linux-tools-$(uname -r) \
    libbpf-dev \
    pkg-config \
    libssl-dev

# Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Compile:

```bash
# Option A — compile everything at once (recommended)
cargo build --release

# Option B — compile eBPF objects only
make ebpf

# Option C — compile both via make
make all
```

Verify the binary was produced:

```bash
ls -lh target/release/nexus-axiom
# Expected: -rwxr-xr-x ... target/release/nexus-axiom
```

---

## Step 3 — Run

```bash
# Full pipeline: preflight + compile + eBPF load + W^X test + smoke test
sudo bash run.sh

# Equivalent via make
sudo make run

# Run the daemon directly (after build)
sudo ./target/release/nexus-axiom
```

`run.sh` runs four phases automatically:
1. Preflight — checks root, tools, BPF LSM
2. `cargo build --release` — compiles Rust + eBPF skeleton
3. `ebpf/run_test.sh` — compiles eBPF, loads it, runs W^X test
4. Binary smoke test — confirms the binary starts cleanly

---

## Step 4 — Test

```bash
# Run Rust unit tests
cargo test

# Run the eBPF + W^X integration test standalone
sudo bash ebpf/run_test.sh

# Run exploit simulation tests
cd cve_tests && make && sudo ./test_pwnkit
```

Expected output from `ebpf/run_test.sh`:

```
=== Nexus W^X eBPF LSM Test ===
[INFO] Active LSMs: capability,landlock,yama,apparmor,bpf
[INFO] Compiling ebpf/nexus_working.bpf.c ...
[PASS] eBPF compiled → nexus_working.bpf.o
[INFO] Loading eBPF program ...
[PASS] eBPF loaded and pinned at /sys/fs/bpf/nexus_wx
[INFO] Compiling test_wx.c ...
[PASS] Test binary compiled → test_wx
[INFO] Running W^X blocking test ...
---
[PASS] mmap PROT_WRITE|PROT_EXEC blocked: Operation not permitted
[PASS] mprotect RW->RWX blocked: Operation not permitted
---
[PASS] ALL TESTS PASSED — W^X blocking is working correctly
```

---

## Step 5 — Verify It Blocks Exploits

```bash
# Compile the W^X test manually and run it
gcc -O0 -o /tmp/test_wx ebpf/test_wx.c
sudo bash ebpf/run_test.sh   # loads the LSM
/tmp/test_wx                 # run the test (LSM is now active)
```

Both tests must print `[PASS]`. If either prints `[FAIL]`, the LSM is not blocking.

Manual verification:

```bash
# Confirm the eBPF program is pinned and loaded
sudo bpftool prog show pinned /sys/fs/bpf/nexus_wx

# Confirm LSM hooks are attached
sudo bpftool prog list | grep lsm

# Watch events in real time (while running test_wx in another terminal)
sudo cat /sys/kernel/debug/tracing/trace_pipe
```

---

## Troubleshooting

### "BPF LSM not active" / `bpf` missing from `/sys/kernel/security/lsm`

```bash
# Check current boot parameters
cat /proc/cmdline

# Edit grub
sudo nano /etc/default/grub
# Change: GRUB_CMDLINE_LINUX_DEFAULT="quiet splash"
# To:     GRUB_CMDLINE_LINUX_DEFAULT="quiet splash lsm=bpf,lockdown,yama,integrity,apparmor"

sudo update-grub
sudo reboot

# After reboot, verify
cat /sys/kernel/security/lsm   # must contain "bpf"
```

### "Missing: clang / bpftool / gcc"

```bash
sudo apt install -y clang llvm gcc linux-tools-$(uname -r) libbpf-dev
# If linux-tools-$(uname -r) not found:
sudo apt install -y linux-tools-generic
```

### "cargo: command not found"

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
# Verify
cargo --version
```

### Rust build fails with libbpf errors

```bash
sudo apt install -y libbpf-dev libelf-dev zlib1g-dev
# Then retry
cargo build --release
```

### `bpftool prog load` fails with "Operation not supported"

The kernel doesn't have `CONFIG_BPF_LSM=y`. Check:

```bash
zcat /proc/config.gz | grep BPF_LSM
# Must show: CONFIG_BPF_LSM=y
```

If not set, you need a different kernel or must recompile the kernel with that option. On Ubuntu, install a newer HWE kernel:

```bash
sudo apt install linux-generic-hwe-22.04
sudo reboot
```

### `bpftool prog load` fails with "Permission denied"

```bash
# Must be root
sudo bash ebpf/run_test.sh

# Check BPF filesystem is mounted
mount | grep bpf
# If missing:
sudo mount -t bpf bpf /sys/fs/bpf
```

### W^X test prints `[FAIL]` (exploit not blocked)

The eBPF program loaded but isn't blocking. Check:

```bash
# Is the program actually attached to the LSM hook?
sudo bpftool prog list | grep -A3 lsm

# Check dmesg for errors
dmesg | grep -i bpf | tail -10

# Verify the pin exists
ls -la /sys/fs/bpf/nexus_wx
```

### WSL2 users

WSL2 cannot run this project at runtime. Microsoft's kernel does not include `CONFIG_BPF_LSM=y` and the boot parameters cannot be changed. You can compile (`cargo build --release`, `make ebpf`) and run unit tests (`cargo test`) on WSL2, but exploit blocking requires real Linux. Use VirtualBox with Ubuntu 22.04, or a cloud VM (Oracle Cloud free tier, AWS t2.micro free tier).

---

## Quick Reference

| Task | Command |
|------|---------|
| Full pipeline | `sudo bash run.sh` |
| Compile only | `cargo build --release` |
| eBPF test only | `sudo bash ebpf/run_test.sh` |
| Unit tests | `cargo test` |
| Run daemon | `sudo ./target/release/nexus-axiom` |
| Check LSM active | `cat /sys/kernel/security/lsm` |
| Check loaded progs | `sudo bpftool prog list` |
| Clean build | `make clean` |
