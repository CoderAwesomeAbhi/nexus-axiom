#!/usr/bin/env bash
# Nexus Axiom — One-Click Deploy
# Usage: curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/deploy/quickstart.sh | sudo bash
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()    { echo -e "${CYAN}[nexus]${NC} $*"; }
success() { echo -e "${GREEN}[nexus]${NC} $*"; }
warn()    { echo -e "${YELLOW}[nexus]${NC} $*"; }
die()     { echo -e "${RED}[nexus] ERROR:${NC} $*" >&2; exit 1; }

# ── Checks ────────────────────────────────────────────────────────────────────
[[ $EUID -eq 0 ]] || die "Run as root: sudo bash quickstart.sh"
[[ "$(uname)" == "Linux" ]] || die "Linux required (eBPF is Linux-only)"

KERNEL=$(uname -r | cut -d. -f1-2)
KERNEL_MAJOR=$(echo "$KERNEL" | cut -d. -f1)
KERNEL_MINOR=$(echo "$KERNEL" | cut -d. -f2)
if [[ $KERNEL_MAJOR -lt 5 ]] || { [[ $KERNEL_MAJOR -eq 5 ]] && [[ $KERNEL_MINOR -lt 8 ]]; }; then
  die "Kernel 5.8+ required (you have $(uname -r)). eBPF LSM needs 5.8+."
fi

# Check BTF
[[ -f /sys/kernel/btf/vmlinux ]] || warn "BTF not found at /sys/kernel/btf/vmlinux — eBPF may fall back to simulation mode"

# ── Detect deployment method ──────────────────────────────────────────────────
DEPLOY_METHOD=""
if command -v docker &>/dev/null && command -v docker-compose &>/dev/null; then
  DEPLOY_METHOD="docker-compose"
elif command -v docker &>/dev/null; then
  DEPLOY_METHOD="docker"
elif command -v kubectl &>/dev/null && command -v helm &>/dev/null; then
  DEPLOY_METHOD="helm"
else
  DEPLOY_METHOD="binary"
fi

info "Detected deployment method: $DEPLOY_METHOD"
info "Kernel: $(uname -r)"

# ── Deploy ────────────────────────────────────────────────────────────────────
REPO="https://github.com/CoderAwesomeAbhi/nexus-axiom"
IMAGE="ghcr.io/coderawesomeabhi/nexus-axiom:latest"

case "$DEPLOY_METHOD" in
  docker-compose)
    info "Deploying with Docker Compose..."
    TMPDIR=$(mktemp -d)
    curl -sSL "$REPO/raw/main/deploy/docker-compose.yml" -o "$TMPDIR/docker-compose.yml"
    curl -sSL "$REPO/raw/main/grafana/prometheus.yml"    -o "$TMPDIR/prometheus.yml" 2>/dev/null || true
    cd "$TMPDIR"
    docker-compose pull
    docker-compose up -d
    success "Nexus Axiom running!"
    info "Dashboard: http://localhost:8080"
    info "Metrics:   http://localhost:9090"
    info "Grafana:   http://localhost:3000 (admin / nexus-admin)"
    ;;

  docker)
    info "Deploying with Docker..."
    docker pull "$IMAGE"
    docker run -d \
      --name nexus-axiom \
      --restart unless-stopped \
      --privileged \
      --pid host \
      --network host \
      -v /sys/kernel/debug:/sys/kernel/debug:ro \
      -v /sys/fs/bpf:/sys/fs/bpf \
      -v /sys/kernel/btf:/sys/kernel/btf:ro \
      -e RUST_LOG=info \
      -e NEXUS_MODE=audit \
      -p 8080:8080 \
      -p 9090:9090 \
      "$IMAGE"
    success "Nexus Axiom running!"
    info "Dashboard: http://localhost:8080"
    info "Logs:      docker logs -f nexus-axiom"
    ;;

  helm)
    info "Deploying with Helm..."
    helm repo add nexus-axiom "$REPO" 2>/dev/null || true
    helm upgrade --install nexus-axiom \
      "$REPO/raw/main/deploy/helm/nexus-axiom" \
      --namespace nexus-axiom \
      --create-namespace \
      --set mode=audit
    success "Nexus Axiom deployed to Kubernetes!"
    info "Check status: kubectl get pods -n nexus-axiom"
    ;;

  binary)
    info "Installing binary directly..."
    ARCH=$(uname -m)
    [[ "$ARCH" == "x86_64" ]] || die "Binary releases only for x86_64 (you have $ARCH)"
    LATEST=$(curl -sSL "https://api.github.com/repos/CoderAwesomeAbhi/nexus-axiom/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    BINARY_URL="$REPO/releases/download/$LATEST/nexus-axiom-linux-x86_64"
    curl -sSL "$BINARY_URL" -o /usr/local/bin/nexus-axiom
    chmod +x /usr/local/bin/nexus-axiom
    # Install systemd service
    cat > /etc/systemd/system/nexus-axiom.service <<'EOF'
[Unit]
Description=Nexus Axiom eBPF Security
After=network.target

[Service]
ExecStart=/usr/local/bin/nexus-axiom start
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload
    systemctl enable --now nexus-axiom
    success "Nexus Axiom installed and running!"
    info "Status: systemctl status nexus-axiom"
    info "Logs:   journalctl -u nexus-axiom -f"
    ;;
esac

# ── Verify ────────────────────────────────────────────────────────────────────
sleep 3
if curl -sf http://localhost:8080/health &>/dev/null; then
  success "Health check passed ✓"
else
  warn "Health check pending — service may still be starting"
  info "Check: curl http://localhost:8080/health"
fi

echo ""
success "🛡️  Nexus Axiom is protecting your system!"
info "Mode: AUDIT (logging only). To enforce: set NEXUS_MODE=enforce"
info "Docs: $REPO"
