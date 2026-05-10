#!/bin/bash
# Nexus Axiom Verification Suite
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "🧪 Nexus Axiom Verification Suite"
echo "===================================="

# Build test exploit if needed
if [ ! -f test_exploit ]; then
    gcc -o test_exploit test_exploit.c 2>/dev/null || echo "⚠️  test_exploit.c not found"
fi

# Test 1: W^X blocking
echo -n "1. W^X blocking... "
if [ -f test_exploit ]; then
    if ./test_exploit 2>&1 | grep -q "Killed"; then
        echo -e "${GREEN}✅ PASS${NC}"
    else
        echo -e "${RED}❌ FAIL${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}⚠️  SKIP${NC}"
fi

# Test 2: Metrics
echo -n "2. Metrics endpoint... "
if curl -s localhost:9090/metrics | grep -q "nexus_axiom"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    exit 1
fi

# Test 3: Dashboard
echo -n "3. Dashboard... "
if curl -s localhost:8080 | grep -q "Nexus Axiom"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ VERIFICATION COMPLETE${NC}"
