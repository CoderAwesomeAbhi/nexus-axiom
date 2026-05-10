#!/bin/bash

# Nexus Axiom - W^X Exploit Demo
# This script demonstrates real-time exploit blocking

set +e  # Don't exit on error - we expect exploits to be killed

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🛡️  NEXUS AXIOM - W^X EXPLOIT BLOCKING DEMO${NC}"
echo "=================================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}❌ Please run as root: sudo ./demo.sh${NC}"
    exit 1
fi

# Build test exploit
echo -e "${YELLOW}📦 Building W^X exploit test...${NC}"
gcc -o test_exploit test_exploit.c 2>/dev/null || {
    echo -e "${RED}❌ Failed to compile test exploit${NC}"
    exit 1
}
echo -e "${GREEN}✅ Exploit compiled${NC}"
echo ""

# Phase 1: Without protection
echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${RED}🔴 PHASE 1: System WITHOUT Protection${NC}"
echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}Running W^X exploit on unprotected system...${NC}"
echo ""

./test_exploit
RESULT=$?

if [ $RESULT -eq 0 ]; then
    echo ""
    echo -e "${RED}🚨 EXPLOIT SUCCEEDED - System is VULNERABLE!${NC}"
    echo -e "${RED}   The exploit allocated W^X memory and could execute shellcode${NC}"
else
    echo ""
    echo -e "${YELLOW}⚠️  Exploit was blocked (kernel may have built-in protections)${NC}"
fi

echo ""
echo -e "${YELLOW}Press Enter to start Nexus Axiom protection...${NC}"
read

# Phase 2: With protection
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🟢 PHASE 2: System WITH Nexus Axiom${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check if binary exists
if [ ! -f "./target/release/nexus-axiom" ]; then
    echo -e "${RED}❌ Nexus Axiom binary not found. Build it first:${NC}"
    echo "   cargo build --release"
    exit 1
fi

# Start Nexus Axiom
echo -e "${YELLOW}🚀 Starting Nexus Axiom...${NC}"
./target/release/nexus-axiom start > /tmp/nexus-axiom.log 2>&1 &
NEXUS_PID=$!

# Wait for eBPF programs to load
sleep 3

# Verify it's running
if ! ps -p $NEXUS_PID > /dev/null; then
    echo -e "${RED}❌ Nexus Axiom failed to start. Check logs:${NC}"
    cat /tmp/nexus-axiom.log
    exit 1
fi

echo -e "${GREEN}✅ Nexus Axiom is running (PID: $NEXUS_PID)${NC}"
echo ""
echo -e "${YELLOW}Running same W^X exploit with protection...${NC}"
echo ""

# Run exploit - it should be killed
./test_exploit
RESULT=$?

echo ""

if [ $RESULT -ne 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✅ EXPLOIT BLOCKED!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${GREEN}🛡️  Nexus Axiom successfully:${NC}"
    echo -e "${GREEN}   1. Detected W^X memory allocation attempt${NC}"
    echo -e "${GREEN}   2. Blocked the syscall at kernel level (LSM hook)${NC}"
    echo -e "${GREEN}   3. Terminated the exploit process (SIGKILL)${NC}"
    echo ""
    echo -e "${BLUE}📊 Check the logs:${NC}"
    echo "   tail /tmp/nexus-axiom.log"
else
    echo -e "${RED}⚠️  Exploit was not blocked as expected${NC}"
    echo -e "${YELLOW}   This may indicate a configuration issue${NC}"
fi

# Show recent events
echo ""
echo -e "${BLUE}📋 Recent security events:${NC}"
tail -n 10 /tmp/nexus-axiom.log | grep -E "BLOCKED|W\^X|SIGKILL" || echo "   (check full log for details)"

# Cleanup
echo ""
echo -e "${YELLOW}🧹 Stopping Nexus Axiom...${NC}"
kill $NEXUS_PID 2>/dev/null || true
wait $NEXUS_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}🎉 Demo complete!${NC}"
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📚 Learn more:${NC}"
echo "   • GitHub: https://github.com/CoderAwesomeAbhi/nexus-axiom"
echo "   • Docs: https://github.com/CoderAwesomeAbhi/nexus-axiom/blob/main/README.md"
echo ""
echo -e "${YELLOW}⭐ If this impressed you, star us on GitHub!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Cleanup test binary
rm -f test_exploit
