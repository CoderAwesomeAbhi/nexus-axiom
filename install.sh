#!/bin/bash
# Nexus Axiom - One-Liner Install
# curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/nexus-axiom/main/install.sh | sudo bash

set -e

echo "🛡️  NEXUS AXIOM - ONE-LINER INSTALL"
echo "===================================="
echo ""

# Check if root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run with sudo"
    exit 1
fi

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    echo "❌ Cannot detect OS"
    exit 1
fi

echo "📦 Installing dependencies for $OS..."

# Install dependencies based on OS
case $OS in
    ubuntu|debian)
        apt-get update -qq
        apt-get install -y -qq clang llvm libbpf-dev linux-headers-$(uname -r) curl
        ;;
    fedora|rhel|centos)
        dnf install -y -q clang llvm libbpf-devel kernel-devel curl
        ;;
    arch)
        pacman -Sy --noconfirm clang llvm libbpf linux-headers
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Download and build
echo "📥 Downloading Nexus Axiom..."
cd /tmp
rm -rf nexus-axiom
git clone -q https://github.com/YOUR_USERNAME/nexus-axiom.git
cd nexus-axiom

echo "🔨 Building (this takes ~2 minutes)..."
cargo build --release --quiet

# Install binary
echo "📦 Installing to /usr/local/bin..."
cp target/release/nexus-axiom /usr/local/bin/
chmod +x /usr/local/bin/nexus-axiom

# Create systemd service
echo "⚙️  Creating systemd service..."
cat > /etc/systemd/system/nexus-axiom.service << 'EOF'
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

systemctl daemon-reload

echo ""
echo "✅ Installation complete!"
echo ""
echo "🚀 Quick Start:"
echo "   sudo nexus-axiom start              # Start protection"
echo "   sudo systemctl enable nexus-axiom   # Start on boot"
echo "   sudo systemctl start nexus-axiom    # Start service"
echo ""
echo "📊 Commands:"
echo "   nexus-axiom status    # Check status"
echo "   nexus-axiom monitor   # Watch events"
echo "   nexus-axiom stop      # Stop protection"
echo ""
echo "⭐ Star us: https://github.com/YOUR_USERNAME/nexus-axiom"
