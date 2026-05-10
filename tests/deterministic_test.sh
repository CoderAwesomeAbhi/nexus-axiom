#!/bin/bash
# Deterministic Test Suite - Proof-grade tests

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "🧪 Nexus Axiom Deterministic Test Suite"
echo "========================================"

# Build test exploit
echo "Building test exploit..."
gcc -o test_exploit test_exploit.c || exit 1

# Start daemon
echo "Starting Nexus Axiom..."
sudo ../target/release/nexus-axiom start &
DAEMON_PID=$!
sleep 3

# Test 1: W^X blocking
echo -n "Test 1: W^X blocking... "
if ./test_exploit 2>&1 | grep -q "Killed"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    kill $DAEMON_PID 2>/dev/null || true
    exit 1
fi

# Test 2: Metrics increment
echo -n "Test 2: Metrics increment... "
BEFORE=$(curl -s localhost:9090/metrics | grep nexus_axiom_blocked_total | awk '{print $2}')
./test_exploit 2>&1 || true
sleep 1
AFTER=$(curl -s localhost:9090/metrics | grep nexus_axiom_blocked_total | awk '{print $2}')

if [ "$AFTER" -gt "$BEFORE" ]; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL (before: $BEFORE, after: $AFTER)${NC}"
    kill $DAEMON_PID 2>/dev/null || true
    exit 1
fi

# Test 3: Dashboard responds
echo -n "Test 3: Dashboard... "
if curl -s localhost:8080 | grep -q "Nexus Axiom"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    kill $DAEMON_PID 2>/dev/null || true
    exit 1
fi

# Cleanup
kill $DAEMON_PID 2>/dev/null || true
wait $DAEMON_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}✅ ALL TESTS PASSED${NC}"
echo ""
echo "Test Results:"
echo "  - W^X blocking: PASS"
echo "  - Metrics: PASS"
echo "  - Dashboard: PASS"
