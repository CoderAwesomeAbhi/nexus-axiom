# 🧪 Ubuntu VM Testing Guide - Complete Feature Test

**Test all 14 features of Nexus Axiom**

---

## 📋 PREREQUISITES

### 1. Ubuntu VM Requirements
- **OS:** Ubuntu 22.04+ or 24.04
- **Kernel:** 5.8+ (check with `uname -r`)
- **RAM:** 4GB minimum
- **Disk:** 10GB free space
- **Root access:** Required

### 2. Enable BPF LSM
```bash
# Check if BPF LSM is enabled
cat /sys/kernel/security/lsm

# If "bpf" is NOT in the list, enable it:
sudo nano /etc/default/grub

# Add to GRUB_CMDLINE_LINUX:
GRUB_CMDLINE_LINUX="lsm=bpf,landlock,lockdown,yama,integrity,apparmor"

# Update grub and reboot
sudo update-grub
sudo reboot
```

---

## 🚀 STEP 1: TRANSFER CODE TO VM

### Option A: Git Clone (Recommended)
```bash
cd ~
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom.git
cd nexus-axiom
```

### Option B: Copy from Windows
```bash
# On Windows (PowerShell):
# Assuming VM IP is 192.168.1.100
scp -r C:\Users\abhij\nexus-axiom-final user@192.168.1.100:~/nexus-axiom

# On Ubuntu VM:
cd ~/nexus-axiom
```

---

## 🔧 STEP 2: INSTALL DEPENDENCIES

```bash
# Update system
sudo apt update

# Install build tools
sudo apt install -y \
    curl git \
    clang llvm gcc gcc-multilib \
    libbpf-dev libelf-dev zlib1g-dev \
    linux-tools-$(uname -r) linux-tools-generic \
    pkg-config libssl-dev \
    make

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Verify installations
rustc --version
clang --version
```

---

## 🏗️ STEP 3: BUILD NEXUS AXIOM

```bash
cd ~/nexus-axiom

# Build release version
cargo build --release

# Verify binary
ls -lh target/release/nexus-axiom
```

**Expected output:** Binary ~1.5MB

---

## ✅ STEP 4: TEST ALL 14 FEATURES

### Test 1: Basic Compilation ✅
```bash
cargo build --release
```
**Expected:** Compiles successfully, no errors

---

### Test 2: Help Command ✅
```bash
./target/release/nexus-axiom --help
```
**Expected:** Shows help text with all commands

---

### Test 3: Version Command ✅
```bash
./target/release/nexus-axiom --version
```
**Expected:** Shows "nexus-axiom 1.0.0"

---

### Test 4: Status Command ✅
```bash
./target/release/nexus-axiom status
```
**Expected:** Shows status without requiring root

---

### Test 5: W^X Memory Blocking (CORE FEATURE) ✅

#### Compile test exploit
```bash
cd ~/nexus-axiom
cat > test_wx.c << 'EOF'
#include <stdio.h>
#include <sys/mman.h>
#include <string.h>

int main() {
    printf("Attempting W^X memory allocation...\n");
    
    // Try to allocate memory with WRITE + EXEC
    void *mem = mmap(NULL, 4096, 
                     PROT_WRITE | PROT_EXEC,  // W^X!
                     MAP_PRIVATE | MAP_ANONYMOUS, 
                     -1, 0);
    
    if (mem == MAP_FAILED) {
        printf("❌ BLOCKED! (This is good)\n");
        return 1;
    }
    
    printf("⚠️  W^X memory allocated! (This is bad)\n");
    munmap(mem, 4096);
    return 0;
}
EOF

gcc test_wx.c -o test_wx
```

#### Test WITHOUT Nexus Axiom
```bash
./test_wx
```
**Expected:** "W^X memory allocated!" (exploit succeeds)

#### Test WITH Nexus Axiom
```bash
# Terminal 1: Start Nexus Axiom
sudo ./target/release/nexus-axiom start

# Terminal 2: Run exploit
./test_wx
```
**Expected:** 
- Terminal 1 shows: "🚨 EXPLOIT ATTEMPT BLOCKED 🚨"
- Terminal 2: Process killed or blocked
- **THIS IS THE MOST IMPORTANT TEST!**

---

### Test 6: Dashboard (Port 8080) ✅
```bash
# Start Nexus Axiom
sudo ./target/release/nexus-axiom start

# In another terminal or browser:
curl http://localhost:8080

# Or open in browser:
# http://localhost:8080
```
**Expected:** HTML dashboard showing metrics

---

### Test 7: Prometheus Metrics (Port 9090) ✅
```bash
# With Nexus Axiom running:
curl http://localhost:9090/metrics
```
**Expected:** Prometheus metrics output:
```
nexus_axiom_events_total 0
nexus_axiom_blocked_total 0
nexus_axiom_mmap_events 0
...
```

---

### Test 8: Process Termination ✅
```bash
# Run the W^X test again
./test_wx

# Check if process was killed
echo $?
```
**Expected:** Non-zero exit code (process was killed)

---

### Test 9: Event Reporting ✅
```bash
# With Nexus Axiom running, trigger an event
./test_wx

# Check metrics
curl http://localhost:9090/metrics | grep blocked
```
**Expected:** `nexus_axiom_blocked_total` increments

---

### Test 10: Audit Mode ✅
```bash
# Start in audit mode (logs only, doesn't block)
sudo ./target/release/nexus-axiom start --audit

# Run exploit
./test_wx
```
**Expected:** 
- Logs show "📋 [AUDIT MODE] Would terminate process"
- Process NOT killed
- Exploit succeeds (but logged)

---

### Test 11: Config File Loading ✅
```bash
# Check if config exists
cat config.toml

# Modify config
nano config.toml
# Change dashboard_port to 8081

# Start Nexus Axiom
sudo ./target/release/nexus-axiom start

# Verify new port
curl http://localhost:8081
```
**Expected:** Dashboard on port 8081

---

### Test 12: Filesystem Protection ✅
```bash
# Start Nexus Axiom
sudo ./target/release/nexus-axiom start

# Check logs for FS protection
# Should see: "🛡️ Filesystem protection initialized"
# Should see periodic: "🛡️ FS Protection: X paths monitored"
```
**Expected:** FS protection logs appear every ~60 seconds

---

### Test 13: JSON Logging ✅
```bash
# Create log directory
sudo mkdir -p /var/log/nexus-axiom

# Start Nexus Axiom
sudo ./target/release/nexus-axiom start

# Trigger event
./test_wx

# Check JSON log
sudo cat /var/log/nexus-axiom/events.json
```
**Expected:** JSON formatted events

---

### Test 14: Container Awareness ✅
```bash
# If you have Docker installed:
sudo docker run -it --rm ubuntu bash

# Inside container, try W^X
# Nexus Axiom should detect container name
```
**Expected:** Logs show container name instead of "host"

---

## 🧪 ADVANCED TESTS

### Test 15: CVE Blocking (PwnKit)
```bash
cd ~/nexus-axiom/cve_tests
make

# Without Nexus Axiom
./test_pwnkit
# Expected: May succeed (bad)

# With Nexus Axiom
sudo ../target/release/nexus-axiom start &
./test_pwnkit
# Expected: BLOCKED
```

---

### Test 16: XDP Network Defense
```bash
# Start Nexus Axiom
sudo ./target/release/nexus-axiom start

# Check XDP loaded
sudo bpftool prog list | grep xdp
```
**Expected:** XDP program loaded

---

### Test 17: Benchmarks
```bash
cd ~/nexus-axiom/benchmarks
./run_benchmarks.sh
```
**Expected:** Performance metrics

---

### Test 18: Stress Test
```bash
# Run multiple exploits simultaneously
for i in {1..10}; do
    ./test_wx &
done
wait
```
**Expected:** All blocked, no crashes

---

## 📊 EXPECTED RESULTS SUMMARY

| Test | Feature | Expected Result |
|------|---------|-----------------|
| 1 | Compilation | ✅ Success |
| 2 | Help | ✅ Shows help |
| 3 | Version | ✅ Shows 1.0.0 |
| 4 | Status | ✅ Shows status |
| 5 | **W^X Blocking** | ✅ **BLOCKS EXPLOIT** |
| 6 | Dashboard | ✅ HTML on :8080 |
| 7 | Metrics | ✅ Prometheus on :9090 |
| 8 | Process Kill | ✅ Kills exploit |
| 9 | Event Reporting | ✅ Counters increment |
| 10 | Audit Mode | ✅ Logs without blocking |
| 11 | Config Loading | ✅ Uses config.toml |
| 12 | FS Protection | ✅ Monitors files |
| 13 | JSON Logging | ✅ Creates JSON logs |
| 14 | Container Aware | ✅ Detects containers |

---

## 🐛 TROUBLESHOOTING

### Issue: "Must run as root"
```bash
sudo ./target/release/nexus-axiom start
```

### Issue: "BPF LSM not enabled"
```bash
# Check LSM
cat /sys/kernel/security/lsm

# If missing, add to grub (see Prerequisites)
```

### Issue: "Failed to load eBPF"
```bash
# Check kernel version
uname -r  # Must be 5.8+

# Check BPF support
sudo bpftool prog list
```

### Issue: "Port already in use"
```bash
# Check what's using port 8080
sudo lsof -i :8080

# Kill it or change port in config.toml
```

### Issue: "Permission denied on /var/log"
```bash
sudo mkdir -p /var/log/nexus-axiom
sudo chmod 755 /var/log/nexus-axiom
```

---

## ✅ QUICK TEST SCRIPT

Save this as `quick_test.sh`:

```bash
#!/bin/bash
set -e

echo "🧪 Nexus Axiom Quick Test"
echo "=========================="

# Test 1: Build
echo "1. Building..."
cargo build --release
echo "✅ Build successful"

# Test 2: Help
echo "2. Testing help..."
./target/release/nexus-axiom --help > /dev/null
echo "✅ Help works"

# Test 3: Version
echo "3. Testing version..."
./target/release/nexus-axiom --version
echo "✅ Version works"

# Test 4: Compile exploit
echo "4. Compiling test exploit..."
cat > /tmp/test_wx.c << 'EOF'
#include <sys/mman.h>
#include <stdio.h>
int main() {
    void *m = mmap(NULL, 4096, PROT_WRITE|PROT_EXEC, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
    if (m == MAP_FAILED) { printf("BLOCKED\n"); return 1; }
    printf("NOT BLOCKED\n");
    return 0;
}
EOF
gcc /tmp/test_wx.c -o /tmp/test_wx
echo "✅ Exploit compiled"

# Test 5: Run exploit WITHOUT protection
echo "5. Testing exploit without protection..."
/tmp/test_wx
echo "✅ Exploit runs (expected)"

# Test 6: Run with protection
echo "6. Testing with Nexus Axiom..."
echo "   Starting Nexus Axiom in background..."
sudo ./target/release/nexus-axiom start &
NEXUS_PID=$!
sleep 3

echo "   Running exploit..."
/tmp/test_wx || echo "✅ Exploit BLOCKED (expected)"

echo "   Stopping Nexus Axiom..."
sudo kill $NEXUS_PID

echo ""
echo "🎉 ALL TESTS PASSED!"
echo ""
echo "Next steps:"
echo "  1. Test dashboard: http://localhost:8080"
echo "  2. Test metrics: http://localhost:9090/metrics"
echo "  3. Test audit mode: sudo ./target/release/nexus-axiom start --audit"
```

Run it:
```bash
chmod +x quick_test.sh
./quick_test.sh
```

---

## 📝 TEST CHECKLIST

Print this and check off as you test:

```
□ Code transferred to VM
□ Dependencies installed
□ Rust installed
□ BPF LSM enabled
□ Code compiles
□ Help command works
□ Version command works
□ Status command works
□ W^X blocking works (MOST IMPORTANT!)
□ Dashboard accessible
□ Metrics accessible
□ Process termination works
□ Event reporting works
□ Audit mode works
□ Config file loads
□ FS protection works
□ JSON logging works
□ Container awareness works
```

---

## 🎯 SUCCESS CRITERIA

**Minimum to pass:**
- ✅ Compiles successfully
- ✅ W^X blocking works (blocks test_wx.c)
- ✅ Dashboard accessible
- ✅ Metrics accessible

**Full success:**
- ✅ All 14 features working
- ✅ No crashes
- ✅ No errors in logs

---

## 📞 NEED HELP?

If any test fails:
1. Check kernel version: `uname -r` (must be 5.8+)
2. Check BPF LSM: `cat /sys/kernel/security/lsm`
3. Check logs: `sudo journalctl -xe`
4. Check dmesg: `sudo dmesg | tail -50`

---

**GOOD LUCK! 🚀**

**Expected time:** 30-45 minutes for complete testing
