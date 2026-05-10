#!/bin/bash
# Nexus Axiom Security Audit Script
# Comprehensive validation of system readiness and security posture

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

PASS=0; FAIL=0; WARN=0

check() {
    local name="$1" status="$2" detail="$3"
    case "$status" in
        pass) echo -e "  ${GREEN}✓${NC} $name — $detail"; ((PASS++)) ;;
        fail) echo -e "  ${RED}✗${NC} $name — $detail"; ((FAIL++)) ;;
        warn) echo -e "  ${YELLOW}⚠${NC} $name — $detail"; ((WARN++)) ;;
    esac
}

echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  🛡️  Nexus Axiom Security Audit${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# ── Kernel Configuration ──────────────────────────────────────────
echo -e "${CYAN}[Kernel]${NC}"

if [ -f /sys/kernel/security/lsm ]; then
    LSM=$(cat /sys/kernel/security/lsm)
    if echo "$LSM" | grep -q "bpf"; then
        check "BPF LSM enabled" "pass" "$LSM"
    else
        check "BPF LSM enabled" "fail" "Add lsm=bpf to kernel params. Current: $LSM"
    fi
else
    check "BPF LSM enabled" "fail" "/sys/kernel/security/lsm not found"
fi

KVER=$(uname -r | cut -d. -f1-2)
if awk "BEGIN{exit !($KVER >= 5.8)}"; then
    check "Kernel version" "pass" "$(uname -r) (>= 5.8 required)"
else
    check "Kernel version" "fail" "$(uname -r) (>= 5.8 required)"
fi

if [ -f /sys/kernel/btf/vmlinux ]; then
    check "BTF enabled" "pass" "CONFIG_DEBUG_INFO_BTF=y"
else
    check "BTF enabled" "fail" "/sys/kernel/btf/vmlinux missing"
fi

echo ""

# ── Dependencies ──────────────────────────────────────────────────
echo -e "${CYAN}[Dependencies]${NC}"

for cmd in bpftool ip; do
    if command -v "$cmd" &>/dev/null; then
        check "$cmd" "pass" "$(which $cmd)"
    else
        check "$cmd" "warn" "Not found — some features unavailable"
    fi
done

echo ""

# ── File Permissions ──────────────────────────────────────────────
echo -e "${CYAN}[File Permissions]${NC}"

BINARY=$(which nexus-axiom 2>/dev/null || echo "/usr/local/bin/nexus-axiom")
if [ -f "$BINARY" ]; then
    PERMS=$(stat -c "%a" "$BINARY")
    if [ "$PERMS" = "755" ] || [ "$PERMS" = "750" ]; then
        check "Binary permissions" "pass" "$BINARY ($PERMS)"
    else
        check "Binary permissions" "warn" "$BINARY ($PERMS) — should be 750 or 755"
    fi
else
    check "Binary permissions" "warn" "Binary not installed at $BINARY"
fi

CONFIG="/etc/nexus-axiom/config.toml"
if [ -f "$CONFIG" ]; then
    PERMS=$(stat -c "%a" "$CONFIG")
    if [ "$PERMS" -le "640" ] 2>/dev/null; then
        check "Config permissions" "pass" "$CONFIG ($PERMS)"
    else
        check "Config permissions" "fail" "$CONFIG ($PERMS) — should be 640 or less"
    fi
else
    check "Config permissions" "warn" "No config at $CONFIG"
fi

echo ""

# ── Process Isolation ─────────────────────────────────────────────
echo -e "${CYAN}[Process Isolation]${NC}"

if [ "$(id -u)" -eq 0 ]; then
    check "Running as root" "pass" "UID 0"
else
    check "Running as root" "fail" "Must run as root for eBPF"
fi

echo ""

# ── Network Exposure ──────────────────────────────────────────────
echo -e "${CYAN}[Network]${NC}"

for PORT in 8080 9090; do
    if ss -tlnp 2>/dev/null | grep -q ":$PORT "; then
        check "Port $PORT" "pass" "Active (expected for dashboard/metrics)"
    else
        check "Port $PORT" "warn" "Not active"
    fi
done

echo ""

# ── Summary ───────────────────────────────────────────────────────
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
TOTAL=$((PASS + FAIL + WARN))
echo -e "  Passed: ${GREEN}$PASS${NC}  Failed: ${RED}$FAIL${NC}  Warnings: ${YELLOW}$WARN${NC}  Total: $TOTAL"

if [ "$FAIL" -eq 0 ]; then
    echo -e "  ${GREEN}Overall: PASS${NC}"
    exit 0
else
    echo -e "  ${RED}Overall: FAIL — fix $FAIL issues above${NC}"
    exit 1
fi
