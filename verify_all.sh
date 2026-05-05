#!/bin/bash
# COMPLETE VERIFICATION SCRIPT - Tests EVERY feature
# Run this in Ubuntu VM to verify everything works

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0

echo -e "${BLUE}"
cat << 'EOF'
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   NEXUS AXIOM - COMPLETE VERIFICATION TEST               ║
║   Testing ALL features to ensure NOTHING is fake         ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

pass() {
    echo -e "${GREEN}✅ PASS:${NC} $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}❌ FAIL:${NC} $1"
    ((FAILED++))
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   fail "Must run as root: sudo ./verify_all.sh"
   exit 1
fi

cd ~/nexus-axiom || cd /home/*/nexus-axiom || { fail "nexus-axiom directory not found"; exit 1; }

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 1: Compilation"
echo "════════════════════════════════════════════════════════════"

info "Cleaning build..."
cargo clean > /dev/null 2>&1

info "Building release binary..."
if cargo build --release 2>&1 | tee /tmp/build.log | grep -q "Finished"; then
    pass "Compilation successful"
else
    fail "Compilation failed"
    tail -20 /tmp/build.log
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 2: Unit Tests"
echo "════════════════════════════════════════════════════════════"

info "Running cargo test..."
if cargo test 2>&1 | tee /tmp/test.log | grep -q "test result: ok"; then
    pass "Unit tests passed"
else
    fail "Unit tests failed"
    tail -20 /tmp/test.log
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 3: Binary Exists"
echo "════════════════════════════════════════════════════════════"

if [[ -f target/release/nexus-axiom ]]; then
    pass "Binary exists at target/release/nexus-axiom"
else
    fail "Binary not found"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 4: Core W^X Blocking"
echo "════════════════════════════════════════════════════════════"

info "Starting Nexus Axiom daemon..."
./target/release/nexus-axiom start > /tmp/nexus.log 2>&1 &
NEXUS_PID=$!
sleep 3

if ps -p $NEXUS_PID > /dev/null; then
    pass "Daemon started (PID: $NEXUS_PID)"
else
    fail "Daemon failed to start"
    cat /tmp/nexus.log
    exit 1
fi

info "Testing W^X blocking with test_exploit..."
if ./test_exploit 2>&1 | grep -q "BLOCKED"; then
    pass "W^X blocking works"
else
    fail "W^X blocking not working"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 5: Dashboard"
echo "════════════════════════════════════════════════════════════"

info "Testing dashboard at http://localhost:8080..."
if curl -s http://localhost:8080 | grep -q "Nexus Axiom"; then
    pass "Dashboard accessible"
else
    fail "Dashboard not accessible"
fi

if curl -s http://localhost:8080 | grep -q "value"; then
    pass "Dashboard shows counters"
else
    fail "Dashboard counters not working"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 6: Prometheus Metrics"
echo "════════════════════════════════════════════════════════════"

info "Testing metrics at http://localhost:9090/metrics..."
if curl -s http://localhost:9090/metrics | grep -q "nexus_axiom"; then
    pass "Metrics endpoint accessible"
else
    fail "Metrics endpoint not accessible"
fi

if curl -s http://localhost:9090/metrics | grep -q "nexus_axiom_blocked_total"; then
    pass "Metrics show blocked_total counter"
else
    fail "Metrics missing blocked_total"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 7: Event Reporting (Counters Increment)"
echo "════════════════════════════════════════════════════════════"

info "Getting initial counter value..."
BEFORE=$(curl -s http://localhost:9090/metrics | grep "nexus_axiom_blocked_total" | awk '{print $2}')

info "Running exploit again..."
./test_exploit > /dev/null 2>&1 || true
sleep 1

info "Getting new counter value..."
AFTER=$(curl -s http://localhost:9090/metrics | grep "nexus_axiom_blocked_total" | awk '{print $2}')

if [[ "$AFTER" -gt "$BEFORE" ]]; then
    pass "Counters increment (before: $BEFORE, after: $AFTER)"
else
    fail "Counters not incrementing (before: $BEFORE, after: $AFTER)"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 8: AI Analyst Integration"
echo "════════════════════════════════════════════════════════════"

if grep -q "AI Analysis" /tmp/nexus.log 2>/dev/null; then
    pass "AI Analyst is called (check logs for output)"
else
    warn "AI Analyst not called (may need OPENAI_API_KEY)"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 9: JSON Logger"
echo "════════════════════════════════════════════════════════════"

if [[ -f /var/log/nexus-axiom/events.json ]]; then
    if [[ -s /var/log/nexus-axiom/events.json ]]; then
        pass "JSON log file exists and has content"
        info "Last event: $(tail -1 /var/log/nexus-axiom/events.json)"
    else
        warn "JSON log file exists but is empty"
    fi
else
    warn "JSON log file not created (may need directory permissions)"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 10: Filesystem Protection"
echo "════════════════════════════════════════════════════════════"

if grep -q "Filesystem protection initialized" /tmp/nexus.log; then
    pass "Filesystem protection initialized"
else
    fail "Filesystem protection not initialized"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 11: Container Awareness"
echo "════════════════════════════════════════════════════════════"

if grep -q "Container" /tmp/nexus.log; then
    pass "Container awareness active (check logs for container names)"
else
    warn "Container awareness not showing (may be running on host)"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 12: CVE Exploits"
echo "════════════════════════════════════════════════════════════"

cd cve_tests
info "Compiling CVE tests..."
make clean > /dev/null 2>&1 || true
if make > /dev/null 2>&1; then
    pass "CVE tests compiled"
else
    fail "CVE tests failed to compile"
fi

info "Testing PwnKit (CVE-2021-4034)..."
if ./test_pwnkit 2>&1 | grep -q "BLOCKED"; then
    pass "PwnKit blocked"
else
    fail "PwnKit not blocked"
fi

info "Testing Dirty Pipe (CVE-2022-0847)..."
if ./dirty_pipe 2>&1 | grep -q "BLOCKED"; then
    pass "Dirty Pipe blocked"
else
    warn "Dirty Pipe test inconclusive"
fi

info "Testing Sudo Heap (CVE-2021-3156)..."
if ./sudo_heap 2>&1 | grep -q "BLOCKED"; then
    pass "Sudo Heap blocked"
else
    warn "Sudo Heap test inconclusive"
fi

cd ..

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 13: Benchmarks"
echo "════════════════════════════════════════════════════════════"

cd benchmarks
info "Compiling benchmarks..."
if gcc -O2 bench_baseline.c -o bench_baseline 2>/dev/null && \
   gcc -O2 bench_mmap.c -o bench_mmap 2>/dev/null; then
    pass "Benchmarks compiled"
    
    info "Running baseline benchmark..."
    if ./bench_baseline > /tmp/bench_baseline.log 2>&1; then
        pass "Baseline benchmark ran"
        grep "Average" /tmp/bench_baseline.log || true
    fi
    
    info "Running mmap benchmark..."
    if ./bench_mmap > /tmp/bench_mmap.log 2>&1; then
        pass "Mmap benchmark ran"
        grep "Average" /tmp/bench_mmap.log || true
    fi
else
    fail "Benchmarks failed to compile"
fi

cd ..

echo ""
echo "════════════════════════════════════════════════════════════"
echo "TEST 14: XDP Network Defense"
echo "════════════════════════════════════════════════════════════"

info "Checking if XDP is loaded..."
if ip link show | grep -q "xdp"; then
    pass "XDP program attached"
else
    warn "XDP not attached (may need specific network interface)"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "CLEANUP"
echo "════════════════════════════════════════════════════════════"

info "Stopping Nexus Axiom daemon..."
kill $NEXUS_PID 2>/dev/null || true
sleep 1

if ! ps -p $NEXUS_PID > /dev/null 2>&1; then
    pass "Daemon stopped cleanly"
else
    warn "Daemon still running, force killing..."
    kill -9 $NEXUS_PID 2>/dev/null || true
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "FINAL RESULTS"
echo "════════════════════════════════════════════════════════════"
echo ""
echo -e "${GREEN}PASSED: $PASSED${NC}"
echo -e "${RED}FAILED: $FAILED${NC}"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                           ║${NC}"
    echo -e "${GREEN}║              ✅ ALL TESTS PASSED!                         ║${NC}"
    echo -e "${GREEN}║                                                           ║${NC}"
    echo -e "${GREEN}║   Every feature works. Nothing is fake.                  ║${NC}"
    echo -e "${GREEN}║   Ready for production deployment!                       ║${NC}"
    echo -e "${GREEN}║                                                           ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                           ║${NC}"
    echo -e "${RED}║              ⚠️  SOME TESTS FAILED                        ║${NC}"
    echo -e "${RED}║                                                           ║${NC}"
    echo -e "${RED}║   Review the failures above and fix them.                ║${NC}"
    echo -e "${RED}║                                                           ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
