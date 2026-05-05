# 🧪 Testing Guide - How to Test Nexus Axiom

## ✅ GOOD NEWS: Works on WSL2!

Nexus Axiom now compiles and runs on **WSL2** for development and testing!

**What works on WSL2:**
- ✅ Compilation
- ✅ eBPF loading
- ✅ Event monitoring
- ✅ Detection logging

**What needs bare Linux:**
- ⚠️ Full exploit blocking (LSM enforcement)
- ⚠️ Production deployment

---

## 🖥️ Option 1: WSL2 (Easiest - WORKS NOW!)

### Step 1: Install WSL2
```powershell
# In PowerShell (Admin)
wsl --install -d Ubuntu-22.04
```

### Step 2: Enter WSL2
```powershell
wsl
```

### Step 3: Install Dependencies
```bash
sudo apt-get update
sudo apt-get install -y clang llvm libbpf-dev linux-headers-$(uname -r) build-essential
```

### Step 4: Navigate to Project
```bash
cd /mnt/c/Users/<YourUser>/nexus-axiom
# Or wherever you cloned the repo
```

### Step 5: Build
```bash
make
```

### Step 6: Run Demo
```bash
sudo ./demo.sh
```

---

## 🐧 Option 2: Linux VM

### Using VirtualBox:
1. Download Ubuntu 22.04 ISO
2. Create new VM (4GB RAM, 20GB disk)
3. Install Ubuntu
4. Copy project folder to VM
5. Follow Linux testing steps below

---

## ☁️ Option 3: Cloud Linux Instance

### AWS EC2:
```bash
# Launch Ubuntu 22.04 instance
aws ec2 run-instances --image-id ami-0c55b159cbfafe1f0 --instance-type t2.micro

# SSH into instance
ssh -i your-key.pem ubuntu@<instance-ip>

# Clone project
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom
cd nexus-axiom
```

---

## 🧪 Testing Steps (On Linux)

### Test 1: Verify Compilation
```bash
cd /path/to/nexus-axiom-final
make clean
make

# Should see:
# ✅ eBPF program compiled
# ✅ Rust binary compiled
```

### Test 2: Check Kernel Compatibility
```bash
# Check kernel version (need 5.8+)
uname -r

# Check if BPF LSM is enabled
cat /sys/kernel/security/lsm | grep bpf
```

### Test 3: Run Test Exploit (Without Protection)
```bash
cd cve_tests
make
./test_pwnkit

# Should see:
# [✗] VULNERABLE: Got W^X memory
# [✗] System is vulnerable
```

### Test 4: Run With Nexus Axiom
```bash
# Start Nexus Axiom
sudo ../target/release/nexus-axiom start &

# Wait 2 seconds
sleep 2

# Try exploit again
./test_pwnkit

# Should see:
# Killed
# (Process terminated by Nexus Axiom)
```

### Test 5: Check Logs
```bash
# View system logs
sudo dmesg | tail -20

# Should see:
# [nexus-axiom] Blocked W^X memory: PID 1234 (test_pwnkit)
```

### Test 6: Run Benchmarks
```bash
cd ../benchmarks
./benchmark.sh

# Should show:
# CPU overhead: <0.5%
# Latency: <100ns
```

---

## 🎬 Recording Demo Video

### What You Need:
- Linux system (WSL2, VM, or cloud)
- Screen recording tool (OBS, SimpleScreenRecorder)
- Terminal with large font

### Script:
```bash
# 1. Show exploit works without protection
./test_pwnkit
# [Shows: VULNERABLE]

# 2. Start Nexus Axiom
sudo nexus-axiom start
# [Shows: eBPF hooks loaded]

# 3. Try exploit again
./test_pwnkit
# [Shows: Killed]

# 4. Show logs
sudo dmesg | tail -5
# [Shows: Blocked W^X memory]
```

### Recording Tips:
- Use 1920x1080 resolution
- Large terminal font (16pt+)
- Clear, slow typing
- Add text overlays explaining what's happening

---

## 🐛 Common Issues

### Issue 1: "BPF LSM not enabled"
**Solution**: Add `lsm=bpf` to kernel boot parameters
```bash
sudo nano /etc/default/grub
# Add: GRUB_CMDLINE_LINUX="lsm=bpf"
sudo update-grub
sudo reboot
```

### Issue 2: "Permission denied"
**Solution**: Run with sudo
```bash
sudo ./nexus-axiom start
```

### Issue 3: "Kernel too old"
**Solution**: Upgrade to Ubuntu 22.04+ or kernel 5.8+
```bash
sudo apt-get update
sudo apt-get upgrade
```

### Issue 4: "libbpf not found"
**Solution**: Install dependencies
```bash
sudo apt-get install libbpf-dev
```

---

## ✅ Success Criteria

You know it's working when:
1. ✅ Compiles without errors
2. ✅ Loads eBPF programs successfully
3. ✅ Blocks test exploits (process gets killed)
4. ✅ Shows logs in dmesg
5. ✅ CPU overhead <1%

---

## 📹 Next Steps After Testing

1. **Record 2-minute demo video**
2. **Upload to YouTube**
3. **Post on HackerNews**
4. **Share on Twitter/Reddit**
5. **Watch the stars roll in** ⭐

---

## 🆘 Need Help?

- GitHub Issues: https://github.com/CoderAwesomeAbhi/nexus-axiom/issues
