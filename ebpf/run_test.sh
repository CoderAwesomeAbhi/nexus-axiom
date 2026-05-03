#!/usr/bin/env bash
# run_test.sh — End-to-end test: compile eBPF, load it, verify W^X blocking.
# Must run as root on a Linux kernel with CONFIG_BPF_LSM=y and lsm=bpf.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BPF_SRC="$SCRIPT_DIR/nexus_working.bpf.c"
BPF_OBJ="$SCRIPT_DIR/nexus_working.bpf.o"
TEST_SRC="$SCRIPT_DIR/test_wx.c"
TEST_BIN="$SCRIPT_DIR/test_wx"
PIN_PATH="/sys/fs/bpf/nexus_wx"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
pass() { echo -e "${GREEN}[PASS]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*"; }
info() { echo -e "${YELLOW}[INFO]${NC} $*"; }

cleanup() {
    rm -f "$BPF_OBJ" "$TEST_BIN"
    [[ -e "$PIN_PATH" ]] && rm -f "$PIN_PATH" && info "Unpinned $PIN_PATH"
}
trap cleanup EXIT

# ── 0. Preflight ─────────────────────────────────────────────────────────────
echo "=== Nexus W^X eBPF LSM Test ==="

[[ $EUID -ne 0 ]] && { fail "Must run as root (sudo $0)"; exit 1; }

for cmd in clang bpftool gcc; do
    command -v "$cmd" &>/dev/null || { fail "Missing required tool: $cmd"; exit 1; }
done

# Check BPF LSM is active
if ! grep -q 'bpf' /sys/kernel/security/lsm 2>/dev/null; then
    fail "BPF LSM not active. Boot with 'lsm=bpf,...' or check /sys/kernel/security/lsm"
    info "Current LSMs: $(cat /sys/kernel/security/lsm 2>/dev/null || echo 'unknown')"
    exit 1
fi
info "Active LSMs: $(cat /sys/kernel/security/lsm)"

# ── 1. Compile eBPF ──────────────────────────────────────────────────────────
info "Compiling $BPF_SRC ..."
clang -O2 -g -target bpf -D__TARGET_ARCH_x86 \
    -I/usr/include/bpf -I/usr/include \
    -c "$BPF_SRC" -o "$BPF_OBJ" 2>&1
pass "eBPF compiled → $BPF_OBJ"

# ── 2. Load with bpftool ─────────────────────────────────────────────────────
[[ -e "$PIN_PATH" ]] && { info "Removing stale pin $PIN_PATH"; rm -f "$PIN_PATH"; }

info "Loading eBPF program ..."
bpftool prog load "$BPF_OBJ" "$PIN_PATH" autoattach 2>&1
pass "eBPF loaded and pinned at $PIN_PATH"

# Confirm it's actually attached
PROG_ID=$(bpftool prog show pinned "$PIN_PATH" --json 2>/dev/null | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)
[[ -n "$PROG_ID" ]] && info "Program ID: $PROG_ID" || info "Program loaded (ID lookup skipped)"

# ── 3. Compile test program ──────────────────────────────────────────────────
info "Compiling $TEST_SRC ..."
gcc -O0 -o "$TEST_BIN" "$TEST_SRC" 2>&1
pass "Test binary compiled → $TEST_BIN"

# ── 4. Run W^X test ──────────────────────────────────────────────────────────
info "Running W^X blocking test ..."
echo "---"
set +e
"$TEST_BIN"
EXIT_CODE=$?
set -e
echo "---"

# ── 5. Result ────────────────────────────────────────────────────────────────
if [[ $EXIT_CODE -eq 0 ]]; then
    pass "ALL TESTS PASSED — W^X blocking is working correctly"
    exit 0
else
    fail "TESTS FAILED (exit $EXIT_CODE) — W^X blocking is NOT working"
    info "Troubleshooting:"
    info "  1. Verify 'lsm=bpf' in /proc/cmdline: $(grep -o 'lsm=[^ ]*' /proc/cmdline || echo 'not found')"
    info "  2. Check kernel config: zcat /proc/config.gz | grep BPF_LSM"
    info "  3. Check dmesg for BPF errors: dmesg | grep -i bpf | tail -5"
    exit 1
fi
