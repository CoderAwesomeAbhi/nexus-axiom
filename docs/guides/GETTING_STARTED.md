# 🚀 Getting Started with Nexus Axiom

**Perfect for students, cybersecurity clubs, and homelab enthusiasts!**

---

## 📋 Prerequisites

### What You Need
- Linux system (Ubuntu 22.04 recommended)
- Root access (sudo)
- 30 minutes of time

### Supported Systems
✅ Ubuntu 22.04 LTS  
✅ Ubuntu 20.04 LTS  
✅ Debian 11+  
⚠️ Fedora 36+ (experimental)  
⚠️ Arch Linux (experimental)  
❌ macOS (not yet supported)  
❌ Windows (not yet supported)  

---

## ⚡ Quick Start (5 Minutes)

### Option 1: One-Command Install (Easiest)

```bash
curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash
```

**That's it!** The script will:
1. Check if BPF LSM is enabled
2. Install dependencies
3. Download and install Nexus Axiom
4. Create systemd service
5. Start protection

### Option 2: Manual Install

```bash
# 1. Enable BPF LSM (if not already)
sudo nano /etc/default/grub
# Add: GRUB_CMDLINE_LINUX="lsm=bpf"
sudo update-grub
sudo reboot

# 2. Install dependencies
sudo apt-get update
sudo apt-get install -y clang llvm libelf-dev libbpf-dev

# 3. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 4. Clone and build
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom
cd nexus-axiom
cargo build --release

# 5. Install
sudo cp target/release/nexus-axiom /usr/local/bin/
sudo nexus-axiom start
```

---

## 🧪 Verify It's Working

### Check Status
```bash
sudo systemctl status nexus-axiom
```

**Expected output:**
```
● nexus-axiom.service - Nexus Axiom eBPF Security
   Loaded: loaded
   Active: active (running)
```

### Check Metrics
```bash
curl localhost:9090/metrics | grep nexus_axiom
```

**Expected output:**
```
nexus_axiom_uptime_seconds 120
nexus_axiom_events_total 0
nexus_axiom_blocked_total 0
```

### View Dashboard
Open browser: http://localhost:8080

---

## 🎯 Test with Real Exploit

### Download PwnKit Exploit
```bash
git clone https://github.com/arthepsy/CVE-2021-4034
cd CVE-2021-4034
make
```

### Test WITHOUT Nexus Axiom
```bash
sudo systemctl stop nexus-axiom
./cve-2021-4034
```

**Expected:** Exploit succeeds (you get root shell) ❌

### Test WITH Nexus Axiom
```bash
sudo systemctl start nexus-axiom
./cve-2021-4034
```

**Expected:** Process killed immediately ✅

### Check Logs
```bash
sudo journalctl -u nexus-axiom -n 20
```

**Expected output:**
```
🚨 EXPLOIT ATTEMPT BLOCKED 🚨
Process: cve-2021-4034 (PID: 1337)
Hook: W^X mmap
Status: ✅ BLOCKED AT KERNEL LEVEL
Action: 💀 PROCESS TERMINATED
```

---

## 📊 Understanding the Dashboard

### Metrics Explained

**nexus_axiom_events_total**
- Total security events detected
- Includes both blocked and allowed

**nexus_axiom_blocked_total**
- Number of exploits blocked
- Should be 0 in normal operation
- Spikes indicate attack attempts

**nexus_axiom_mmap_events**
- W^X mmap attempts blocked
- Most common exploit type

**nexus_axiom_mprotect_events**
- W^X mprotect attempts blocked
- Less common than mmap

**nexus_axiom_network_drops**
- Network packets dropped by XDP
- Blocked IPs/ports

---

## 🔧 Common Use Cases

### Use Case 1: Student Lab

**Scenario:** Cybersecurity class learning about exploits

**Setup:**
```bash
# Install on lab machines
curl -sSL https://get.nexus-axiom.dev | sudo bash

# Run in audit mode (log only, don't kill)
sudo nexus-axiom start --audit

# Students can see exploits detected without breaking their work
```

### Use Case 2: CTF Infrastructure

**Scenario:** Protecting CTF challenge servers

**Setup:**
```bash
# Install on challenge servers
sudo nexus-axiom start

# Allowlist legitimate services
sudo nexus-axiom allowlist add-name nginx
sudo nexus-axiom allowlist add-name docker

# Monitor for exploit attempts
sudo journalctl -u nexus-axiom -f
```

### Use Case 3: Homelab Security

**Scenario:** Protecting personal servers

**Setup:**
```bash
# Install on all servers
for server in server1 server2 server3; do
    ssh $server "curl -sSL https://get.nexus-axiom.dev | sudo bash"
done

# Monitor from central location
# Set up Prometheus to scrape all servers
# View unified dashboard
```

---

## 🛠️ Configuration

### Config File Location
```
/etc/nexus-axiom/config.toml
```

### Basic Configuration
```toml
[security]
mode = "enforce"  # or "audit"
kill_on_violation = true

[server]
metrics_port = 9090
dashboard_port = 8080

[network]
blocked_ips = ["192.168.1.100"]
blocked_ports = [22, 23]
```

### Audit Mode (Log Only)
```bash
sudo nexus-axiom start --audit
```

**Use when:**
- Testing in production
- Learning how it works
- Avoiding false positives

---

## 🚫 Managing Allowlist

### Why Allowlist?

Some legitimate programs use W^X memory:
- JIT compilers (Node.js, Java, Python)
- Browsers (JavaScript JIT)
- Emulators (QEMU, VirtualBox)

### Add to Allowlist

**By PID:**
```bash
sudo nexus-axiom allowlist add 1234
```

**By name:**
```bash
sudo nexus-axiom allowlist add-name node
sudo nexus-axiom allowlist add-name java
sudo nexus-axiom allowlist add-name chrome
```

### View Allowlist
```bash
sudo nexus-axiom allowlist list
```

### Clear Allowlist
```bash
sudo nexus-axiom allowlist clear
```

---

## 🐛 Troubleshooting

### Issue: "BPF LSM not enabled"

**Solution:**
```bash
# Edit grub config
sudo nano /etc/default/grub

# Add to GRUB_CMDLINE_LINUX:
GRUB_CMDLINE_LINUX="lsm=bpf"

# Update and reboot
sudo update-grub
sudo reboot

# Verify
cat /proc/cmdline | grep lsm=bpf
```

### Issue: "Permission denied"

**Solution:**
```bash
# Must run as root
sudo nexus-axiom start
```

### Issue: "Port already in use"

**Solution:**
```bash
# Change ports in config
sudo nano /etc/nexus-axiom/config.toml

[server]
metrics_port = 9091  # Change from 9090
dashboard_port = 8081  # Change from 8080
```

### Issue: "False positives"

**Solution:**
```bash
# Add legitimate programs to allowlist
sudo nexus-axiom allowlist add-name node
sudo nexus-axiom allowlist add-name java

# Or run in audit mode
sudo nexus-axiom start --audit
```

---

## 📚 Next Steps

### Learn More
- Read [TRANSPARENCY.md](TRANSPARENCY.md) for limitations
- Read [COMPARISON.md](COMPARISON.md) vs other tools
- Read [ARCHITECTURE.md](docs/ARCHITECTURE.md) for internals

### Get Help
- [GitHub Issues](https://github.com/CoderAwesomeAbhi/nexus-axiom/issues)
- [GitHub Discussions](https://github.com/CoderAwesomeAbhi/nexus-axiom/discussions)
- [Documentation](docs/)

### Contribute
- Report bugs
- Improve docs
- Submit code
- Share feedback

---

## 🎓 For Students

### Using for School Projects

**This is perfect for:**
- Cybersecurity class projects
- Science fair projects
- Capstone projects
- Research papers

**Tips:**
1. Document your setup process
2. Test with real exploits
3. Measure performance impact
4. Compare with other tools
5. Present findings

### College Applications

**This project shows:**
- Technical skills (eBPF, Rust, Linux)
- Problem-solving ability
- Self-directed learning
- Open source contribution

**How to showcase:**
- Link in application
- Mention in essays
- Discuss in interviews
- Include in portfolio

---

## 🤝 For Cybersecurity Clubs

### Club Deployment

**Setup for club:**
```bash
# 1. Set up lab environment
# 2. Install on all machines
# 3. Configure central monitoring
# 4. Create learning exercises
```

**Learning exercises:**
1. Test with known exploits
2. Analyze blocked attempts
3. Study eBPF code
4. Contribute improvements

### Getting Support

**I'm happy to:**
- Help with setup
- Answer questions
- Provide guidance
- Join video calls

**Contact:** [GitHub Issues](https://github.com/CoderAwesomeAbhi/nexus-axiom/issues)

---

## 📧 Need Help?

**I'm here to help!**

- 🐛 Bug reports: [GitHub Issues](https://github.com/CoderAwesomeAbhi/nexus-axiom/issues)
- 💬 Questions: [GitHub Discussions](https://github.com/CoderAwesomeAbhi/nexus-axiom/discussions)
- 📖 Documentation: [docs/](docs/)

**Response time:** Usually within 24 hours (I'm in school during the day)

---

**Happy hacking! 🚀**
