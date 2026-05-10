#!/bin/bash
# Nexus Axiom One-Command Installer
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🛡️  Nexus Axiom Installer"
echo "=========================="
echo ""

# Check root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}❌ Please run as root: sudo bash install.sh${NC}"
    exit 1
fi

# Check Linux
if [ "$(uname)" != "Linux" ]; then
    echo -e "${RED}❌ Linux only (for now)${NC}"
    exit 1
fi

# Check BPF LSM
echo -n "Checking BPF LSM... "
if grep -q "lsm=.*bpf" /proc/cmdline 2>/dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️  Not enabled${NC}"
    echo ""
    echo "BPF LSM is required. Enable it:"
    echo "1. Edit /etc/default/grub"
    echo "2. Add 'lsm=bpf' to GRUB_CMDLINE_LINUX"
    echo "3. Run: sudo update-grub"
    echo "4. Reboot"
    echo ""
    read -p "Enable automatically? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cp /etc/default/grub /etc/default/grub.backup
        sed -i 's/GRUB_CMDLINE_LINUX="\(.*\)"/GRUB_CMDLINE_LINUX="\1 lsm=bpf"/' /etc/default/grub
        update-grub
        echo -e "${GREEN}✅ Enabled. Reboot and run installer again.${NC}"
        exit 0
    else
        exit 1
    fi
fi

# Install dependencies
echo "Installing dependencies..."
apt-get update -qq
apt-get install -y -qq clang llvm libelf-dev libbpf-dev pkg-config curl > /dev/null 2>&1
echo -e "${GREEN}✅ Dependencies installed${NC}"

# Download binary
echo "Downloading Nexus Axiom..."
LATEST_URL="https://github.com/CoderAwesomeAbhi/nexus-axiom/releases/latest/download/nexus-axiom"
curl -sSL "$LATEST_URL" -o /usr/local/bin/nexus-axiom
chmod +x /usr/local/bin/nexus-axiom
echo -e "${GREEN}✅ Binary installed${NC}"

# Create systemd service
echo "Creating systemd service..."
cat > /etc/systemd/system/nexus-axiom.service <<EOF
[Unit]
Description=Nexus Axiom eBPF Security
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/nexus-axiom start
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable nexus-axiom
echo -e "${GREEN}✅ Service created${NC}"

# Create config directory
mkdir -p /var/lib/nexus-axiom
mkdir -p /var/log/nexus-axiom

echo ""
echo -e "${GREEN}✅ Installation complete!${NC}"
echo ""
echo "Start:    sudo systemctl start nexus-axiom"
echo "Status:   sudo systemctl status nexus-axiom"
echo "Logs:     sudo journalctl -u nexus-axiom -f"
echo "Metrics:  http://localhost:9090/metrics"
echo "Dashboard: http://localhost:8080"
echo ""
