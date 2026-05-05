#!/bin/bash
# Nexus Axiom - Uninstaller
# Usage: sudo bash uninstall.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[✓]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*"; exit 1; }
warn() { echo -e "${YELLOW}[!]${NC} $*"; }

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                                                           ║"
echo "║              NEXUS AXIOM UNINSTALLER                      ║"
echo "║                                                           ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if root
[[ $EUID -ne 0 ]] && error "Must run as root: sudo bash uninstall.sh"

# Stop service
if systemctl is-active --quiet nexus-axiom; then
    info "Stopping Nexus Axiom service..."
    systemctl stop nexus-axiom
    success "Service stopped"
fi

# Disable service
if systemctl is-enabled --quiet nexus-axiom 2>/dev/null; then
    info "Disabling service..."
    systemctl disable nexus-axiom --quiet
    success "Service disabled"
fi

# Remove systemd service
if [[ -f /etc/systemd/system/nexus-axiom.service ]]; then
    info "Removing systemd service..."
    rm -f /etc/systemd/system/nexus-axiom.service
    systemctl daemon-reload
    success "Systemd service removed"
fi

# Remove installation directory
if [[ -d /opt/nexus-axiom ]]; then
    info "Removing installation directory..."
    rm -rf /opt/nexus-axiom
    success "Installation directory removed"
fi

# Remove command alias
if [[ -f /usr/local/bin/nexus-axiom ]]; then
    info "Removing command alias..."
    rm -f /usr/local/bin/nexus-axiom
    success "Command alias removed"
fi

# Remove config (ask first)
if [[ -d /etc/nexus-axiom ]]; then
    read -p "Remove configuration files? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Removing configuration..."
        rm -rf /etc/nexus-axiom
        success "Configuration removed"
    else
        warn "Configuration kept at /etc/nexus-axiom"
    fi
fi

# Remove logs (ask first)
if [[ -d /var/log/nexus-axiom ]]; then
    read -p "Remove log files? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Removing logs..."
        rm -rf /var/log/nexus-axiom
        success "Logs removed"
    else
        warn "Logs kept at /var/log/nexus-axiom"
    fi
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                                           ║${NC}"
echo -e "${GREEN}║         ✓ NEXUS AXIOM UNINSTALLED SUCCESSFULLY            ║${NC}"
echo -e "${GREEN}║                                                           ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Thank you for using Nexus Axiom!${NC}"
echo ""
