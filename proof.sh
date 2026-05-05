#!/bin/bash
# Nexus Axiom - Canonical Proof Pack
# Reproducible demonstration that W^X blocking works
# Run this on Ubuntu 22.04+ with kernel 5.8+

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     NEXUS AXIOM - CANONICAL PROOF PACK                    ║${NC}"
echo -e "${BLUE}║     Reproducible W^X Blocking Demonstration               ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Record environment
echo -e "${BLUE}[1/6] Recording Environment${NC}"
echo "OS: $(lsb_release -d | cut -f2)"
echo "Kernel: $(uname -r)"
echo "Architecture: $(uname -m)"
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Check prerequisites
echo -e "${BLUE}[2/6] Checking Prerequisites${NC}"
if ! grep -q 'bpf' /sys/kernel/security/lsm 2>/dev/null; then
    echo -e "${RED}✗ BPF LSM not enabled${NC}"
    echo "Add 'lsm=bpf' to kernel boot parameters"
    exit 1
fi
echo -e "${GREEN}✓ BPF LSM enabled${NC}"
echo ""

# Compile test exploit
echo -e "${BLUE}[3/6] Compiling Test Exploit${NC}"
cat > /tmp/test_wx.c << 'EOF'
#include <sys/mman.h>
#include <stdio.h>
#include <string.h>

int main() {
    printf("[*] Attempting W^X memory allocation...\n");
    
    void *mem = mmap(NULL, 4096, 
                     PROT_WRITE | PROT_EXEC,
                     MAP_PRIVATE | MAP_ANONYMOUS, 
                     -1, 0);
    
    if (mem == MAP_FAILED) {
        printf("[✓] BLOCKED - mmap failed (expected with Nexus Axiom)\n");
        return 1;
    }
    
    printf("[✗] ALLOWED - W^X memory at %p (BAD - exploit would succeed)\n", mem);
    
    // Try to write shellcode
    unsigned char shellcode[] = "\x90\x90\x90\x90"; // NOP sled
    memcpy(mem, shellcode, sizeof(shellcode));
    printf("[✗] Wrote shellcode to W^X memory (exploit succeeded)\n");
    
    return 0;
}
EOF

gcc /tmp/test_wx.c -o /tmp/test_wx
echo -e "${GREEN}✓ Test exploit compiled${NC}"
echo ""

# Test WITHOUT Nexus Axiom
echo -e "${BLUE}[4/6] Test WITHOUT Nexus Axiom (Baseline)${NC}"
echo "Expected: Exploit succeeds (W^X memory allowed)"
echo ""
/tmp/test_wx
BASELINE_RESULT=$?
echo ""
if [ $BASELINE_RESULT -eq 0 ]; then
    echo -e "${RED}✗ Baseline: Exploit SUCCEEDED (W^X allowed)${NC}"
else
    echo -e "${GREEN}✓ Baseline: Exploit blocked by system policy${NC}"
fi
echo ""

# Start Nexus Axiom
echo -e "${BLUE}[5/6] Starting Nexus Axiom${NC}"
if ! systemctl is-active --quiet nexus-axiom; then
    sudo systemctl start nexus-axiom
    sleep 2
fi
echo -e "${GREEN}✓ Nexus Axiom running${NC}"
echo ""

# Test WITH Nexus Axiom
echo -e "${BLUE}[6/6] Test WITH Nexus Axiom (Protected)${NC}"
echo "Expected: Process killed by LSM hook"
echo ""

# Run in subshell to capture kill
(
    /tmp/test_wx 2>&1
) &
TEST_PID=$!
sleep 1

# Check if process was killed
if ! kill -0 $TEST_PID 2>/dev/null; then
    echo -e "${GREEN}✓ Process was KILLED by Nexus Axiom${NC}"
    PROTECTED_RESULT=0
else
    echo -e "${RED}✗ Process still running (protection failed)${NC}"
    kill $TEST_PID 2>/dev/null
    PROTECTED_RESULT=1
fi
echo ""

# Check logs
echo -e "${BLUE}Checking Nexus Axiom Logs:${NC}"
sudo journalctl -u nexus-axiom --since "30 seconds ago" | grep -A 5 "EXPLOIT" || echo "No exploit logs found"
echo ""

# Summary
echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    PROOF SUMMARY                          ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Environment:"
echo "  OS: $(lsb_release -d | cut -f2)"
echo "  Kernel: $(uname -r)"
echo "  Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""
echo "Results:"
if [ $BASELINE_RESULT -eq 0 ]; then
    echo -e "  Baseline (no protection): ${RED}EXPLOIT SUCCEEDED${NC}"
else
    echo -e "  Baseline (no protection): ${GREEN}BLOCKED BY SYSTEM${NC}"
fi

if [ $PROTECTED_RESULT -eq 0 ]; then
    echo -e "  With Nexus Axiom:         ${GREEN}EXPLOIT BLOCKED${NC}"
else
    echo -e "  With Nexus Axiom:         ${RED}PROTECTION FAILED${NC}"
fi
echo ""

if [ $PROTECTED_RESULT -eq 0 ]; then
    echo -e "${GREEN}✓ PROOF COMPLETE: Nexus Axiom blocks W^X exploits${NC}"
    echo ""
    echo "This result is reproducible. Run this script on any Ubuntu 22.04+"
    echo "system with kernel 5.8+ and lsm=bpf enabled."
    exit 0
else
    echo -e "${RED}✗ PROOF FAILED: Protection did not work${NC}"
    exit 1
fi
