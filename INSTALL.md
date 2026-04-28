# Installation Guide - Nexus Axiom

## Prerequisites

### Check Kernel Support
```bash
# Check if eBPF LSM is enabled
cat /sys/kernel/security/lsm | grep bpf

# If not present, you need to enable it
# Add to kernel boot parameters: lsm=bpf,apparmor,selinux
```

### Install Dependencies

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
    clang \
    llvm \
    libbpf-dev \
    linux-headers-$(uname -r) \
    libelf-dev \
    build-essential \
    pkg-config
```

#### Fedora/RHEL
```bash
sudo dnf install -y \
    clang \
    llvm \
    libbpf-devel \
    kernel-devel \
    elfutils-libelf-devel
```

#### Arch Linux
```bash
sudo pacman -S clang llvm libbpf linux-headers
```

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Build from Source

```bash
# Clone repository
git clone https://github.com/YOUR_ORG/nexus-axiom
cd nexus-axiom

# Build everything
make all

# Run tests
make test

# Install system-wide
sudo make install
```

## Verify Installation

```bash
# Check binary
which nexus-axiom

# Check eBPF object
ls -l /usr/lib/nexus-axiom/nexus_real.bpf.o

# Test compilation
gcc -o test_exploit test_exploit.c
./test_exploit
# Should show: System is VULNERABLE (without Nexus Axiom)
```

## First Run

```bash
# Start in audit-only mode (safe)
sudo nexus-axiom start --audit-only

# In another terminal, test the exploit
./test_exploit
# Should show: BLOCKED by kernel!

# Stop
sudo nexus-axiom unload
```

## Troubleshooting

### "Must run as root"
```bash
# eBPF LSM requires root or CAP_BPF
sudo nexus-axiom start
```

### "eBPF LSM not supported"
```bash
# Check kernel version (need 5.7+)
uname -r

# Check LSM support
cat /sys/kernel/security/lsm

# If bpf is missing, add to GRUB:
sudo nano /etc/default/grub
# Add: GRUB_CMDLINE_LINUX="lsm=bpf,apparmor"
sudo update-grub
sudo reboot
```

### "Cannot load eBPF program"
```bash
# Check if BTF is available
ls /sys/kernel/btf/vmlinux

# If missing, install kernel with BTF support
# Ubuntu 20.04+ has this by default
```

### "Compilation failed"
```bash
# Ensure clang is recent enough
clang --version  # Need 10+

# Check libbpf
pkg-config --modversion libbpf  # Need 0.3+
```

## Production Deployment

### Systemd Service
```bash
# Create service file
sudo tee /etc/systemd/system/nexus-axiom.service << EOF
[Unit]
Description=Nexus Axiom eBPF Security
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/nexus-axiom start
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable nexus-axiom
sudo systemctl start nexus-axiom

# Check status
sudo systemctl status nexus-axiom
```

### Docker
```bash
# Build image
docker build -t nexus-axiom .

# Run (requires privileged mode for eBPF)
docker run --privileged --pid=host \
    -v /sys/kernel/debug:/sys/kernel/debug:ro \
    nexus-axiom
```

### Kubernetes
```bash
# Deploy as DaemonSet
kubectl apply -f deploy/k8s-daemonset.yaml

# Check pods
kubectl get pods -n nexus-axiom
```

## Uninstall

```bash
# Stop service
sudo systemctl stop nexus-axiom
sudo systemctl disable nexus-axiom

# Remove files
sudo rm /usr/local/bin/nexus-axiom
sudo rm -rf /usr/lib/nexus-axiom
sudo rm /etc/systemd/system/nexus-axiom.service

# Reload systemd
sudo systemctl daemon-reload
```

## Next Steps

- Read [ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand how it works
- See [QUICKSTART.md](QUICKSTART.md) for usage examples
- Join our [Discord](https://discord.gg/nexus-axiom) for support
