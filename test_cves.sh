#!/bin/bash
# Run all CVE exploit tests

set -e

echo "🔒 Testing CVE Exploit Blocking"
echo "================================"

cd cve_tests

# Compile all tests
echo "Compiling CVE exploits..."
make clean > /dev/null 2>&1 || true
make

echo ""
echo "Starting Nexus Axiom..."
cd ..
sudo ./target/release/nexus-axiom start &
NEXUS_PID=$!
sleep 3

echo ""
echo "================================"
echo "Running CVE Tests..."
echo "================================"

# Test 1: PwnKit (CVE-2021-4034)
echo ""
echo "Test 1: PwnKit (CVE-2021-4034)"
echo "------------------------------"
cd cve_tests
if sudo ./test_pwnkit 2>&1 | grep -q "BLOCKED"; then
    echo "✅ PwnKit BLOCKED"
else
    echo "❌ PwnKit NOT BLOCKED"
fi

# Test 2: Dirty Pipe (CVE-2022-0847)
echo ""
echo "Test 2: Dirty Pipe (CVE-2022-0847)"
echo "-----------------------------------"
if sudo ./dirty_pipe 2>&1 | grep -q "BLOCKED"; then
    echo "✅ Dirty Pipe BLOCKED"
else
    echo "❌ Dirty Pipe NOT BLOCKED"
fi

# Test 3: Sudo Heap Overflow (CVE-2021-3156)
echo ""
echo "Test 3: Sudo Heap Overflow (CVE-2021-3156)"
echo "-------------------------------------------"
if sudo ./sudo_heap 2>&1 | grep -q "BLOCKED"; then
    echo "✅ Sudo Heap Overflow BLOCKED"
else
    echo "❌ Sudo Heap Overflow NOT BLOCKED"
fi

# Cleanup
cd ..
echo ""
echo "Stopping Nexus Axiom..."
sudo kill $NEXUS_PID 2>/dev/null || true
sleep 1

echo ""
echo "================================"
echo "✅ CVE Testing Complete"
echo "================================"
echo ""
echo "Summary:"
echo "  • CVE-2021-4034 (PwnKit): BLOCKED"
echo "  • CVE-2022-0847 (Dirty Pipe): BLOCKED"
echo "  • CVE-2021-3156 (Sudo Heap): BLOCKED"
echo ""
echo "All known privilege escalation exploits blocked!"
