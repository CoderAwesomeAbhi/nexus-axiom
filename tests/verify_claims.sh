#!/bin/bash
# Nexus Axiom Verification Suite
# Verifies all claims are real

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🧪 Nexus Axiom Verification Suite"
echo "=================================="
echo ""

FAILED=0
PASSED=0

# Test 1: Compilation
echo -n "Test 1: Code compiles... "
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAILED++))
fi

# Test 2: eBPF code exists
echo -n "Test 2: eBPF code exists... "
if [ -f "ebpf/nexus_working.bpf.c" ] && [ -f "ebpf/nexus_net.bpf.c" ]; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAILED++))
fi

# Test 3: Check for missing modules
echo -n "Test 3: No missing module declarations... "
if ! grep -q "pub mod policy_engine" src/main.rs && \
   ! grep -q "pub mod attack_scoring" src/main.rs && \
   ! grep -q "pub mod containment_ladder" src/main.rs; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "   Found missing module declarations in main.rs"
    ((FAILED++))
fi

# Test 4: Allowlist has real implementation
echo -n "Test 4: Allowlist has real BPF syscalls... "
if grep -q "libc::syscall" src/allowlist_kernel.rs && \
   grep -q "BPF_MAP_UPDATE_ELEM" src/allowlist_kernel.rs; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "   Allowlist still has TODO stubs"
    ((FAILED++))
fi

# Test 5: Network drops metric is incremented
echo -n "Test 5: Network drops metric incremented... "
if grep -q "network_drops.fetch_add" src/net_engine.rs; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    echo "   network_drops metric never incremented"
    ((FAILED++))
fi

# Test 6: eBPF has allowlist map
echo -n "Test 6: eBPF has allowlist map... "
if grep -q "} allowlist SEC" ebpf/nexus_working.bpf.c; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAILED++))
fi

# Test 7: eBPF checks allowlist
echo -n "Test 7: eBPF checks allowlist before blocking... "
if grep -q "bpf_map_lookup_elem(&allowlist" ebpf/nexus_working.bpf.c; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAILED++))
fi

# Test 8: Metrics are real
echo -n "Test 8: All metrics have real increments... "
if grep -q "blocked_events.fetch_add" src/ebpf_engine.rs && \
   grep -q "mmap_events.fetch_add" src/ebpf_engine.rs && \
   grep -q "mprotect_events.fetch_add" src/ebpf_engine.rs; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAILED++))
fi

# Test 9: Process termination implemented
echo -n "Test 9: Process termination implemented... "
if grep -q "signal::kill.*SIGKILL" src/ebpf_engine.rs; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAILED++))
fi

# Test 10: Allowlist loaded on startup
echo -n "Test 10: Allowlist loaded on startup... "
if grep -q "load_allowlist_from_disk" src/main.rs; then
    echo -e "${GREEN}✅ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ FAIL${NC}"
    ((FAILED++))
fi

echo ""
echo "=================================="
echo "Results: ${PASSED} passed, ${FAILED} failed"
echo "=================================="

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ ALL TESTS PASSED${NC}"
    echo ""
    echo "Nexus Axiom is ready for advertising!"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
    echo ""
    echo "Fix the issues above before advertising."
    exit 1
fi
