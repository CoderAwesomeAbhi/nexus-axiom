# Nexus Axiom - NPM Package

🛡️ **eBPF Security That Actually Kills Exploits**

Real-time exploit prevention using eBPF LSM hooks. Blocks W^X memory exploits before execution.

## Quick Install

```bash
npm install -g nexus-axiom
```

## What It Does

✅ Blocks W^X memory (write+execute) at kernel level  
✅ Automatically kills exploit processes (SIGKILL)  
✅ Tested against 4 major CVEs (PwnKit, Dirty Pipe, Sudo heap overflow)  
✅ Zero false positives in production testing  

## Requirements

- **OS:** Linux (kernel 5.8+)
- **Kernel:** `CONFIG_BPF_LSM=y` and `lsm=bpf` boot parameter
- **Permissions:** Root access for installation

## Installation

### Via NPM (Recommended)

```bash
# Install globally
npm install -g nexus-axiom

# Run installer
sudo nexus-axiom install
```

### Direct Installation

```bash
curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash
```

## Usage

```bash
# Start protection
sudo systemctl start nexus-axiom

# Check status
sudo systemctl status nexus-axiom

# View live events
sudo journalctl -u nexus-axiom -f

# View metrics
curl http://localhost:9090/metrics

# Open dashboard
xdg-open http://localhost:8080
```

## How It Works

```
Traditional tools:   syscall → memory mapped → [alert] ❌ too late
Nexus Axiom:        syscall → [LSM blocks] → -EPERM ✅ prevented
```

Most security tools (Falco, Tetragon) use **tracepoints** — they fire *after* the syscall completes. By the time they log the event, the memory is already mapped.

Nexus Axiom uses **LSM hooks** — it blocks the syscall *before* memory is allocated. The exploit never gets a chance.

## Tested Against Real CVEs

| CVE | Vulnerability | Result |
|-----|--------------|--------|
| CVE-2021-4034 | PwnKit (pkexec) | ✅ Process killed |
| CVE-2021-3156 | Sudo heap overflow | ✅ Process killed |
| CVE-2022-0847 | Dirty Pipe | ✅ Process killed |
| CVE-2022-0185 | Heap overflow (fs) | ✅ Process killed |

## Features

- 🚫 **W^X Memory Blocking** — LSM hooks on mmap + mprotect
- 💀 **Process Termination** — Automatic SIGKILL on exploit attempts
- 📊 **Prometheus Metrics** — 8 metrics at :9090/metrics
- 🌐 **Web Dashboard** — Live UI at :8080
- 🔍 **JSON Logging** — Splunk, ELK, Datadog formats
- ☸️ **Kubernetes Native** — DaemonSet + Helm chart
- 🌊 **XDP Network Filtering** — Line-rate packet filtering
- 🐳 **Container Aware** — Per-cgroup event attribution

## Documentation

- **GitHub:** https://github.com/CoderAwesomeAbhi/nexus-axiom
- **Installation Guide:** https://github.com/CoderAwesomeAbhi/nexus-axiom/blob/main/INSTALL.md
- **Architecture:** https://github.com/CoderAwesomeAbhi/nexus-axiom/blob/main/docs/ARCHITECTURE.md
- **Kubernetes Deployment:** https://github.com/CoderAwesomeAbhi/nexus-axiom/blob/main/docs/DEPLOYMENT.md

## Troubleshooting

### BPF LSM not enabled

```bash
# Check current LSMs
cat /sys/kernel/security/lsm

# Enable BPF LSM
sudo sed -i 's/GRUB_CMDLINE_LINUX=""/GRUB_CMDLINE_LINUX="lsm=bpf"/' /etc/default/grub
sudo update-grub
sudo reboot
```

### Binary not found

The NPM package is a wrapper. The actual binary is installed via the install script:

```bash
curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash
```

## Contributing

Contributions welcome! See [CONTRIBUTING.md](https://github.com/CoderAwesomeAbhi/nexus-axiom/blob/main/CONTRIBUTING.md)

## License

GPL-3.0 — see [LICENSE](https://github.com/CoderAwesomeAbhi/nexus-axiom/blob/main/LICENSE)

## Support

- **Issues:** https://github.com/CoderAwesomeAbhi/nexus-axiom/issues
- **Discussions:** https://github.com/CoderAwesomeAbhi/nexus-axiom/discussions

---

**Built with ❤️ by security engineers tired of watching exploits succeed.**

⭐ Star us on GitHub: https://github.com/CoderAwesomeAbhi/nexus-axiom
