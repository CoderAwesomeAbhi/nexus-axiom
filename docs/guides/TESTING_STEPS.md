# 🚀 COMPLETE TESTING GUIDE - Copy & Paste Commands

## STEP 0: Push to GitHub

```bash
cd C:\Users\abhij\nexus-axiom-final
git push origin main
```

---

## STEP 1: Get a Linux Machine (FREE OPTIONS)

### Option A: Oracle Cloud (FREE FOREVER)
1. Go to: https://cloud.oracle.com/
2. Sign up (free tier)
3. Create Ubuntu 22.04 VM
4. SSH into it

### Option B: WSL2 (Already have it!)
```bash
wsl
```

---

## STEP 2: Clone Your Repo

```bash
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom
cd nexus-axiom
```

---

## STEP 3: Install Dependencies

```bash
sudo apt-get update
sudo apt-get install -y clang llvm libbpf-dev build-essential linux-headers-$(uname -r)
```

---

## STEP 4: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

## STEP 5: Compile Everything

```bash
make
```

**Expected output:**
```
✅ eBPF LSM compiled: target/bpf/nexus_real.bpf.o
✅ eBPF XDP compiled: target/bpf/nexus_net.bpf.o
✅ Rust binary compiled
```

---

## STEP 6: Check if eBPF LSM is Enabled

```bash
cat /sys/kernel/security/lsm
```

**Should contain:** `bpf`

**If NOT, you need to:**
1. Edit `/etc/default/grub`
2. Add `lsm=bpf` to `GRUB_CMDLINE_LINUX`
3. Run `sudo update-grub`
4. Reboot

---

## STEP 7: Test Compilation (Already Done!)

```bash
ls -lh target/bpf/
ls -lh target/release/nexus-axiom
```

**Should see:**
- `nexus_real.bpf.o` (eBPF LSM)
- `nexus_net.bpf.o` (eBPF XDP)
- `nexus-axiom` (Rust binary)

---

## STEP 8: Run Nexus Axiom

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

---

## STEP 9: Test Metrics (In Another Terminal)

```bash
curl http://localhost:9090/metrics
```

**Should see:**
```
nexus_axiom_events_total 0
nexus_axiom_blocked_total 0
nexus_axiom_uptime_seconds 5
```

---

## STEP 10: Test Dashboard (In Another Terminal)

```bash
curl http://localhost:8080
```

**Should see:** HTML with "Nexus Axiom Dashboard"

**Or open in browser:** http://localhost:8080

---

## STEP 11: Test Exploit Blocking (In Another Terminal)

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

**Check metrics again:**
```bash
curl http://localhost:9090/metrics | grep blocked
```

**Should show:** `nexus_axiom_blocked_total 1`

---

## STEP 12: Test Network Filtering

```bash
# Block SSH port
sudo ./target/release/nexus-axiom block-port 22

# Try to SSH (should fail)
ssh localhost
```

---

## STEP 13: Test Kubernetes (Optional)

```bash
# Install minikube
curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
sudo install minikube-linux-amd64 /usr/local/bin/minikube

# Start minikube
minikube start

# Deploy Nexus Axiom
kubectl apply -f deploy/kubernetes/manifests/crd.yaml
kubectl apply -f deploy/kubernetes/manifests/daemonset.yaml
kubectl apply -f deploy/kubernetes/manifests/example-policy.yaml

# Check status
kubectl get pods -n kube-system | grep nexus-axiom
kubectl logs -n kube-system ds/nexus-axiom
```

---

## STEP 14: Verify Everything Works

### Check eBPF Programs Loaded
```bash
sudo bpftool prog list | grep nexus
```

### Check Metrics
```bash
curl http://localhost:9090/metrics
```

### Check Dashboard
```bash
curl http://localhost:8080
```

### Check Logs
```bash
sudo journalctl -f | grep nexus
```

---

## ✅ SUCCESS CRITERIA

| Test | Command | Expected Result |
|------|---------|-----------------|
| **Compilation** | `make` | ✅ No errors |
| **eBPF Loaded** | `sudo bpftool prog list` | ✅ See nexus programs |
| **Metrics** | `curl :9090/metrics` | ✅ See metrics |
| **Dashboard** | `curl :8080` | ✅ See HTML |
| **Exploit Block** | `./test_pwnkit` | ✅ Process killed |
| **Network Block** | Block port 22 | ✅ SSH fails |

---

## 🐛 TROUBLESHOOTING

### If "eBPF LSM not enabled"
```bash
# Check kernel version
uname -r  # Need 5.8+

# Check LSM
cat /sys/kernel/security/lsm  # Should have "bpf"

# If not, add to grub:
sudo nano /etc/default/grub
# Add: GRUB_CMDLINE_LINUX="lsm=bpf"
sudo update-grub
sudo reboot
```

### If "Permission denied"
```bash
# Run with sudo
sudo ./target/release/nexus-axiom start
```

### If "Cargo not found"
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### If "libbpf not found"
```bash
sudo apt-get install -y libbpf-dev
```

---

## 📸 TAKE SCREENSHOTS

1. **Compilation success**
2. **Nexus Axiom running**
3. **Metrics output**
4. **Dashboard in browser**
5. **Exploit being killed**
6. **Blocked counter increasing**

---

## 🎥 RECORD DEMO VIDEO

```bash
# Install asciinema
sudo apt-get install -y asciinema

# Record
asciinema rec demo.cast

# Run these commands:
sudo ./target/release/nexus-axiom start &
sleep 2
curl http://localhost:9090/metrics
./cve_tests/test_pwnkit
curl http://localhost:9090/metrics | grep blocked

# Stop recording: Ctrl+D
```

---

## 📝 UPDATE README WITH RESULTS

Add to README.md:
```markdown
## ✅ Verified Working

Tested on Ubuntu 22.04 (Kernel 5.15):
- ✅ Blocks W^X mmap
- ✅ Blocks W^X mprotect
- ✅ Blocks ptrace
- ✅ Network filtering works
- ✅ Metrics endpoint works
- ✅ Dashboard works
- ✅ Kubernetes deployment works

See [VERIFICATION_GUIDE.md](VERIFICATION_GUIDE.md) for reproduction steps.
```

---

## 🚀 FINAL STEP: Announce It!

### Post on:
1. **Reddit** - r/netsec, r/linux, r/kubernetes
2. **HackerNews** - https://news.ycombinator.com/submit
3. **Twitter/X** - Tag @ebpf_io
4. **LinkedIn** - Share with #cybersecurity #ebpf

### Title:
"Nexus Axiom: eBPF Security Tool That Actually Blocks Exploits (Not Just Observes)"

---

## 💪 YOU'RE DONE!

If all steps pass, your code is **PRODUCTION READY** and will easily get **5K+ stars**! 🚀
