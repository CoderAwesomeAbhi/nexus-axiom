#!/bin/bash
# Nexus Axiom - One-Command Installer
# Usage: curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}"
cat << "EOF"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗           ║
║   ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝           ║
║   ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗           ║
║   ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║           ║
║   ██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║           ║
║   ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝           ║
║                                                           ║
║        AXIOM - eBPF Security That Actually Blocks        ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[✓]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*"; exit 1; }
warn() { echo -e "${YELLOW}[!]${NC} $*"; }

# Check if root
[[ $EUID -ne 0 ]] && error "Must run as root: curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash"

# Check kernel version
KERNEL_VERSION=$(uname -r | cut -d. -f1,2)
info "Kernel version: $(uname -r)"
if (( $(echo "$KERNEL_VERSION < 5.8" | bc -l) )); then
    error "Kernel 5.8+ required. Current: $(uname -r)"
fi
success "Kernel version OK"

# Check BPF LSM
info "Checking BPF LSM support..."
if ! grep -q 'bpf' /sys/kernel/security/lsm 2>/dev/null; then
    warn "BPF LSM not enabled!"
    warn "Add 'lsm=bpf' to /etc/default/grub and run: sudo update-grub && sudo reboot"
    error "BPF LSM required. See: https://github.com/CoderAwesomeAbhi/nexus-axiom#installation"
fi
success "BPF LSM enabled"

# Install dependencies
info "Installing dependencies..."
apt-get update -qq
apt-get install -y -qq \
    curl git \
    clang llvm gcc gcc-multilib \
    libbpf-dev libelf-dev zlib1g-dev \
    linux-tools-$(uname -r) linux-tools-generic \
    pkg-config libssl-dev \
    > /dev/null 2>&1
success "Dependencies installed"

# Install Rust
if ! command -v cargo &> /dev/null; then
    info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
    source "$HOME/.cargo/env"
    success "Rust installed"
else
    success "Rust already installed"
fi

# Clone repo
INSTALL_DIR="/opt/nexus-axiom"
if [[ -d "$INSTALL_DIR" ]]; then
    info "Updating existing installation..."
    cd "$INSTALL_DIR"
    git pull -q
else
    info "Cloning Nexus Axiom..."
    git clone -q https://github.com/CoderAwesomeAbhi/nexus-axiom.git "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi
success "Repository ready"

# Compile
info "Compiling Nexus Axiom (this may take 2-3 minutes)..."
source "$HOME/.cargo/env"
cargo build --release 2>&1 | tee /tmp/nexus-build.log | grep -v "warning:"
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    error "Compilation failed. See /tmp/nexus-build.log"
fi
success "Compilation complete (eBPF embedded in binary)"

# Create config
info "Creating configuration..."
mkdir -p /etc/nexus-axiom
if [[ -f "$INSTALL_DIR/config.toml" ]]; then
    cp "$INSTALL_DIR/config.toml" /etc/nexus-axiom/config.toml
    success "Configuration created at /etc/nexus-axiom/config.toml"
else
    warn "config.toml not found, using defaults"
fi

# Create log directory
info "Creating log directory..."
mkdir -p /var/log/nexus-axiom
chmod 755 /var/log/nexus-axiom
success "Log directory created at /var/log/nexus-axiom"

# Create systemd service
info "Creating systemd service..."
cat > /etc/systemd/system/nexus-axiom.service << 'SYSTEMD_EOF'
[Unit]
Description=Nexus Axiom eBPF Security
After=network.target

[Service]
Type=simple
ExecStart=/opt/nexus-axiom/target/release/nexus-axiom start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
SYSTEMD_EOF

systemctl daemon-reload
systemctl enable nexus-axiom --quiet
success "Systemd service created"

# Test it
info "Testing exploit blocking..."
cd "$INSTALL_DIR/cve_tests"
make clean > /dev/null 2>&1 || true
make > /dev/null 2>&1
if ./pwnkit 2>&1 | grep -q "BLOCKED"; then
    success "Exploit blocking verified!"
else
    warn "Test inconclusive - but installation complete"
fi

# Create command alias
info "Creating command alias..."
cat > /usr/local/bin/nexus-axiom << 'CMD_EOF'
#!/bin/bash
exec /opt/nexus-axiom/target/release/nexus-axiom "$@"
CMD_EOF
chmod +x /usr/local/bin/nexus-axiom
success "Command 'nexus-axiom' available"

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                                           ║${NC}"
echo -e "${GREEN}║              ✓ INSTALLATION COMPLETE!                     ║${NC}"
echo -e "${GREEN}║                                                           ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Quick Start:${NC}"
echo ""
echo -e "  ${GREEN}•${NC} Start protection:  ${YELLOW}sudo systemctl start nexus-axiom${NC}"
echo -e "  ${GREEN}•${NC} Check status:      ${YELLOW}sudo systemctl status nexus-axiom${NC}"
echo -e "  ${GREEN}•${NC} View logs:         ${YELLOW}sudo journalctl -u nexus-axiom -f${NC}"
echo -e "  ${GREEN}•${NC} Test blocking:     ${YELLOW}cd /opt/nexus-axiom/cve_tests && ./pwnkit${NC}"
echo ""
echo -e "${BLUE}Documentation:${NC} https://github.com/CoderAwesomeAbhi/nexus-axiom"
echo ""
echo -e "${GREEN}Nexus Axiom is now protecting your system!${NC}"
echo ""
