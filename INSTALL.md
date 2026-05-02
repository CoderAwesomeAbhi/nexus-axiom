# Installation Guide

## Requirements

- **OS:** Linux with kernel 5.8+ (LSM BPF support)
- **Architecture:** x86_64 (ARM64 coming soon)
- **Root access:** Required for loading eBPF programs

## Quick Install

### Ubuntu/Debian
```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y \
    clang \
    llvm \
    libbpf-dev \
    linux-headers-$(uname -r) \
    build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build Nexus Axiom
git clone https://github.com/YOUR_USERNAME/nexus-axiom
cd nexus-axiom
cargo build --release

# Run
sudo ./target/release/nexus-axiom start
```

### RHEL/Fedora/CentOS
```bash
# Install dependencies
sudo dnf install -y \
    clang \
    llvm \
    libbpf-devel \
    kernel-devel \
    make \
    gcc

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build Nexus Axiom
git clone https://github.com/YOUR_USERNAME/nexus-axiom
cd nexus-axiom
cargo build --release

# Run
sudo ./target/release/nexus-axiom start
```

### Arch Linux
```bash
# Install dependencies
sudo pacman -S clang llvm libbpf linux-headers

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Build Nexus Axiom
git clone https://github.com/YOUR_USERNAME/nexus-axiom
cd nexus-axiom
cargo build --release

# Run
sudo ./target/release/nexus-axiom start
```

## Verify Installation

```bash
# Check kernel version (must be 5.8+)
uname -r

# Check if LSM BPF is enabled
cat /sys/kernel/security/lsm | grep bpf

# If "bpf" is not in the list, add it to kernel boot parameters:
# Edit /etc/default/grub and add to GRUB_CMDLINE_LINUX:
# lsm=lockdown,yama,integrity,apparmor,bpf

# Then update grub and reboot:
sudo update-grub
sudo reboot
```

## Usage

### Start Protection (Enforce Mode)
```bash
sudo nexus-axiom start
```

### Start in Audit Mode (Log Only)
```bash
sudo nexus-axiom start --audit
```

### Monitor Events
```bash
sudo nexus-axiom monitor
```

### Check Status
```bash
sudo nexus-axiom status
```

### Stop Protection
```bash
sudo nexus-axiom stop
```

## Running as a Service

### systemd Service
Create `/etc/systemd/system/nexus-axiom.service`:

```ini
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
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable nexus-axiom
sudo systemctl start nexus-axiom
sudo systemctl status nexus-axiom
```

## Troubleshooting

### "Failed to load eBPF programs"
- Check kernel version: `uname -r` (must be 5.8+)
- Check if running as root: `id -u` (must be 0)
- Check if LSM BPF is enabled: `cat /sys/kernel/security/lsm | grep bpf`

### "Permission denied"
- Must run with sudo
- Check SELinux/AppArmor isn't blocking

### "Cannot find vmlinux.h"
- Install kernel headers: `sudo apt-get install linux-headers-$(uname -r)`
- Or generate BTF: `bpftool btf dump file /sys/kernel/btf/vmlinux format c > vmlinux.h`

### High CPU Usage
- Check if rate limiting is working
- Reduce logging verbosity
- Add more processes to allowlist

## Uninstall

```bash
# Stop service
sudo systemctl stop nexus-axiom
sudo systemctl disable nexus-axiom

# Remove binary
sudo rm /usr/local/bin/nexus-axiom

# Remove service file
sudo rm /etc/systemd/system/nexus-axiom.service
sudo systemctl daemon-reload
```

## Next Steps

- Read [ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand how it works
- Run the [demo](demo.sh) to see it in action
- Check [CONTRIBUTING.md](CONTRIBUTING.md) to contribute
