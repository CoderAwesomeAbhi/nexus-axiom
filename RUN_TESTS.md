# 🚀 HOW TO ACTUALLY TEST & RUN IT

## ⚠️ IMPORTANT: WSL2 Limitation

**WSL2 does NOT support eBPF LSM hooks!** You need:
- Oracle Cloud free VM (Ubuntu 22.04)
- Bare metal Linux
- AWS EC2 free tier

---

## OPTION 1: Oracle Cloud (FREE FOREVER) ⭐ RECOMMENDED

### Step 1: Create Free VM
1. Go to: https://cloud.oracle.com/
2. Sign up (free tier - no credit card for first 30 days)
3. Create Compute Instance:
   - Image: **Ubuntu 22.04**
   - Shape: **VM.Standard.E2.1.Micro** (always free)
4. Download SSH key
5. Note the public IP

### Step 2: SSH into VM
```bash
ssh -i your-key.pem ubuntu@YOUR_VM_IP
```

### Step 3: Install Everything
```bash
# Update system
sudo apt-get update
sudo apt-get upgrade -y

# Install dependencies
sudo apt-get install -y clang llvm libbpf-dev build-essential linux-headers-$(uname -r)

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone your repo
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom
cd nexus-axiom
```

### Step 4: Check eBPF LSM Support
```bash
cat /sys/kernel/security/lsm
```

**Should contain "bpf"**

**If NOT:**
```bash
# Edit grub
sudo nano /etc/default/grub

# Find line: GRUB_CMDLINE_LINUX=""
# Change to: GRUB_CMDLINE_LINUX="lsm=bpf"

# Save (Ctrl+X, Y, Enter)

# Update grub and reboot
sudo update-grub
sudo reboot
```

### Step 5: Compile
```bash
make
```

**Expected output:**
```
✅ eBPF LSM compiled: target/bpf/nexus_real.bpf.o
✅ eBPF XDP compiled: target/bpf/nexus_net.bpf.o
✅ Finished release [optimized] target(s)
```

### Step 6: Run Nexus Axiom
```bash
sudo ./target/release/nexus-axiom start
```

**Expected output:**
```
🛡️  Nexus Axiom v1.0
✅ eBPF LSM hooks loaded
✅ Mode: ENFORCE (kills exploits)
📈 Metrics: http://0.0.0.0:9090/metrics
🌐 Dashboard: http://0.0.0.0:8080
```

### Step 7: Test Metrics (New Terminal)
```bash
# SSH again in new terminal
ssh -i your-key.pem ubuntu@YOUR_VM_IP

# Check metrics
curl http://localhost:9090/metrics
```

**Should see:**
```
nexus_axiom_events_total 0
nexus_axiom_blocked_total 0
nexus_axiom_uptime_seconds 5
```

### Step 8: Test Dashboard
```bash
curl http://localhost:8080
```

**Should see HTML with "Nexus Axiom Dashboard"**

**Or from your PC:**
```
http://YOUR_VM_IP:8080
```

### Step 9: Test Exploit Blocking
```bash
cd cve_tests
make
./test_pwnkit
```

**Expected:**
```
[*] Attempting W^X memory allocation...
Killed
```

### Step 10: Verify It Worked
```bash
# Check metrics again
curl http://localhost:9090/metrics | grep blocked
```

**Should show:**
```
nexus_axiom_blocked_total 1
```

### Step 11: Check eBPF Programs Loaded
```bash
sudo bpftool prog list | grep nexus
```

**Should see your eBPF programs!**

---

## OPTION 2: Quick Test on WSL2 (Limited)

**Note:** eBPF LSM won't work, but you can test compilation:

```bash
cd /mnt/c/Users/abhij/nexus-axiom-final/nexus-axiom

# Compile eBPF (this works)
make ebpf

# Try to compile Rust (might work)
cargo build --release
```

**You'll see:**
```
✅ eBPF compiles
❌ Runtime won't work (no LSM support)
```

---

## OPTION 3: AWS EC2 Free Tier

### Step 1: Create EC2 Instance
1. Go to: https://aws.amazon.com/free/
2. Launch EC2 instance:
   - AMI: **Ubuntu 22.04**
   - Instance type: **t2.micro** (free tier)
3. Download key pair
4. Configure security group:
   - Allow SSH (22)
   - Allow 8080 (dashboard)
   - Allow 9090 (metrics)

### Step 2: Follow Same Steps as Oracle Cloud
(Steps 2-11 from Option 1)

---

## 🎥 RECORD DEMO

Once it's working:

```bash
# Install recording tool
sudo apt-get install -y asciinema

# Record demo
asciinema rec demo.cast

# Run these commands:
sudo ./target/release/nexus-axiom start &
sleep 3
curl http://localhost:9090/metrics
cd cve_tests && ./test_pwnkit
curl http://localhost:9090/metrics | grep blocked

# Stop recording: Ctrl+D

# Upload to asciinema
asciinema upload demo.cast
```

---

## 📸 TAKE SCREENSHOTS

1. **Terminal showing Nexus Axiom running**
2. **Metrics output** (`curl :9090/metrics`)
3. **Dashboard in browser** (http://VM_IP:8080)
4. **Exploit being killed** (`./test_pwnkit`)
5. **Blocked counter = 1**

---

## ✅ SUCCESS CHECKLIST

- [ ] VM created (Oracle/AWS)
- [ ] SSH works
- [ ] eBPF LSM enabled (`cat /sys/kernel/security/lsm` shows "bpf")
- [ ] Code compiles (`make` succeeds)
- [ ] Nexus Axiom runs (`sudo ./target/release/nexus-axiom start`)
- [ ] Metrics work (`curl :9090/metrics`)
- [ ] Dashboard works (`curl :8080`)
- [ ] Exploit blocked (`./test_pwnkit` says "Killed")
- [ ] Counter increases (blocked_total = 1)

---

## 🐛 TROUBLESHOOTING

### "eBPF LSM not enabled"
```bash
# Add to grub
sudo nano /etc/default/grub
# Add: GRUB_CMDLINE_LINUX="lsm=bpf"
sudo update-grub
sudo reboot
```

### "Permission denied"
```bash
# Always use sudo
sudo ./target/release/nexus-axiom start
```

### "Port already in use"
```bash
# Kill existing process
sudo pkill nexus-axiom
```

### "libbpf not found"
```bash
sudo apt-get install -y libbpf-dev
```

---

## 🚀 AFTER IT WORKS

### 1. Update README
Add to your README:
```markdown
## ✅ Verified Working

Tested on Ubuntu 22.04 (Oracle Cloud):
- ✅ Blocks W^X mmap
- ✅ Blocks W^X mprotect  
- ✅ Blocks ptrace
- ✅ Metrics endpoint works
- ✅ Dashboard works

[See demo →](link-to-asciinema)
```

### 2. Create GitHub Release
```bash
git tag v1.0.0
git push origin v1.0.0
```

### 3. Post on Social Media
- Reddit: r/netsec, r/linux
- HackerNews
- Twitter/X with #ebpf #cybersecurity

---

## 💪 YOU'RE READY!

Once you complete these steps and it works, you'll have:
- ✅ Proof it works
- ✅ Demo video
- ✅ Screenshots
- ✅ Real metrics

**This will get you 5K+ stars easily!** 🚀
