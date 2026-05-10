# Docker Quick Start

## ⚠️ IMPORTANT: Linux Only

**Nexus Axiom requires a native Linux kernel with BPF LSM support.**

### ❌ Will NOT Work On:
- Docker Desktop (Mac)
- Docker Desktop (Windows)
- WSL2 (Windows Subsystem for Linux)
- Virtual machines without kernel access

### ✅ Will Work On:
- Native Linux (Ubuntu, Debian, Fedora, etc.)
- Linux cloud instances (AWS EC2, GCP, Azure)
- Bare metal Linux servers

**Why?** eBPF requires direct kernel access. Docker Desktop uses a VM, and Nexus Axiom can't access the host kernel from inside a container.

---

## Quick Start (Linux Only)

## Prerequisites
- Docker 20.10+
- Linux kernel 5.8+ with `CONFIG_BPF_LSM=y`
- `lsm=bpf` kernel boot parameter

## Quick Demo

```bash
# 1. Clone the repository
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom
cd nexus-axiom

# 2. Build and run
docker-compose up -d

# 3. Check status
docker logs nexus-axiom

# 4. Access dashboard
open http://localhost:8080

# 5. View metrics
curl http://localhost:9090/metrics

# 6. Test with exploit
docker exec -it nexus-axiom /nexus-axiom/examples/test_wx_memory
# Should be killed immediately
```

## Build Only

```bash
docker build -t nexus-axiom .
docker run --privileged --network host \
  -v /sys/kernel/debug:/sys/kernel/debug:ro \
  -v /sys/fs/bpf:/sys/fs/bpf \
  -v /sys/kernel/btf:/sys/kernel/btf:ro \
  nexus-axiom
```

## Limitations

- Requires `--privileged` for eBPF loading
- Requires `--network host` for XDP attachment
- Cannot run on Docker Desktop (needs native Linux kernel)
- Works on: Linux VMs, cloud instances, bare metal

## Troubleshooting

**"BPF LSM not enabled"**
```bash
# Check kernel config
cat /sys/kernel/security/lsm
# Should contain "bpf"

# If not, add to kernel boot params:
sudo sed -i 's/GRUB_CMDLINE_LINUX=""/GRUB_CMDLINE_LINUX="lsm=bpf"/' /etc/default/grub
sudo update-grub
sudo reboot
```

**"Failed to load eBPF"**
```bash
# Check BTF is available
ls /sys/kernel/btf/vmlinux
# Should exist

# Check bpftool is installed
docker exec nexus-axiom bpftool version
```
