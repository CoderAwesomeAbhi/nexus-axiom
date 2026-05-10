#!/bin/bash
# Pre-flight check for Nexus Axiom installation

set -e

echo "🔍 Nexus Axiom Pre-Flight Check"
echo "================================"
echo ""

ERRORS=0
WARNINGS=0

# Check 1: Root access
echo -n "✓ Checking root access... "
if [ "$EUID" -ne 0 ]; then
    echo "❌ FAIL"
    echo "  Error: Must run as root (sudo)"
    ERRORS=$((ERRORS + 1))
else
    echo "✅ OK"
fi

# Check 2: Kernel version
echo -n "✓ Checking kernel version... "
KERNEL_VERSION=$(uname -r | cut -d. -f1,2)
REQUIRED_VERSION="5.8"
if awk "BEGIN {exit !($KERNEL_VERSION >= $REQUIRED_VERSION)}"; then
    echo "✅ OK ($KERNEL_VERSION)"
else
    echo "❌ FAIL"
    echo "  Error: Kernel $KERNEL_VERSION < $REQUIRED_VERSION required"
    echo "  Current: $(uname -r)"
    ERRORS=$((ERRORS + 1))
fi

# Check 3: BPF LSM enabled
echo -n "✓ Checking BPF LSM... "
if [ -f /sys/kernel/security/lsm ]; then
    LSM_LIST=$(cat /sys/kernel/security/lsm)
    if echo "$LSM_LIST" | grep -q "bpf"; then
        echo "✅ OK"
    else
        echo "❌ FAIL"
        echo "  Error: BPF LSM not enabled"
        echo "  Current LSMs: $LSM_LIST"
        echo ""
        echo "  To fix:"
        echo "  1. Edit /etc/default/grub"
        echo "  2. Add 'lsm=bpf' to GRUB_CMDLINE_LINUX"
        echo "  3. Run: sudo update-grub"
        echo "  4. Reboot"
        echo ""
        ERRORS=$((ERRORS + 1))
    fi
else
    echo "❌ FAIL"
    echo "  Error: /sys/kernel/security/lsm not found"
    ERRORS=$((ERRORS + 1))
fi

# Check 4: BTF available
echo -n "✓ Checking BTF... "
if [ -f /sys/kernel/btf/vmlinux ]; then
    echo "✅ OK"
else
    echo "⚠️  WARNING"
    echo "  Warning: BTF not available"
    echo "  eBPF programs may not compile"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 5: Required tools
echo -n "✓ Checking bpftool... "
if command -v bpftool &> /dev/null; then
    echo "✅ OK"
else
    echo "⚠️  WARNING"
    echo "  Warning: bpftool not found"
    echo "  Install: sudo apt-get install linux-tools-generic"
    WARNINGS=$((WARNINGS + 1))
fi

echo -n "✓ Checking clang... "
if command -v clang &> /dev/null; then
    echo "✅ OK"
else
    echo "❌ FAIL"
    echo "  Error: clang not found"
    echo "  Install: sudo apt-get install clang"
    ERRORS=$((ERRORS + 1))
fi

# Check 6: Memory
echo -n "✓ Checking available memory... "
AVAILABLE_MB=$(free -m | awk '/^Mem:/{print $7}')
if [ "$AVAILABLE_MB" -gt 512 ]; then
    echo "✅ OK (${AVAILABLE_MB}MB available)"
else
    echo "⚠️  WARNING"
    echo "  Warning: Low memory (${AVAILABLE_MB}MB available)"
    echo "  Recommended: 512MB+ free"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 7: Disk space
echo -n "✓ Checking disk space... "
AVAILABLE_MB=$(df -m . | awk 'NR==2 {print $4}')
if [ "$AVAILABLE_MB" -gt 100 ]; then
    echo "✅ OK (${AVAILABLE_MB}MB available)"
else
    echo "⚠️  WARNING"
    echo "  Warning: Low disk space (${AVAILABLE_MB}MB available)"
    echo "  Recommended: 100MB+ free"
    WARNINGS=$((WARNINGS + 1))
fi

# Summary
echo ""
echo "================================"
if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo "✅ All checks passed!"
    echo ""
    echo "Ready to install Nexus Axiom:"
    echo "  curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo "⚠️  $WARNINGS warning(s) - installation may work but not optimal"
    echo ""
    echo "You can proceed, but fix warnings for best results."
    exit 0
else
    echo "❌ $ERRORS error(s), $WARNINGS warning(s)"
    echo ""
    echo "Fix errors above before installing."
    exit 1
fi
