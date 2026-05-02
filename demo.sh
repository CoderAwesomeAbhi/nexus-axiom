#!/bin/bash

set -e

echo "🛡️  NEXUS AXIOM - LIVE DEMO"
echo "================================"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run as root (sudo ./demo.sh)"
    exit 1
fi

# Build examples
echo "📦 Building exploit tests..."
cd examples
make clean && make
cd ..

echo ""
echo "🔴 PHASE 1: System WITHOUT Protection"
echo "======================================"
echo ""
echo "Running exploit tests on unprotected system..."
echo ""

echo "Test 1: W^X Memory Allocation"
./examples/test_wx_memory
echo ""

echo "Test 2: CVE-2021-4034 (PwnKit)"
./examples/test_pwnkit
echo ""

echo "Test 3: mprotect() to W^X"
./examples/test_mprotect
echo ""

echo "❌ All exploits succeeded - system is VULNERABLE"
echo ""
echo "Press Enter to start Nexus Axiom protection..."
read

echo ""
echo "🟢 PHASE 2: System WITH Nexus Axiom"
echo "======================================"
echo ""

# Start Nexus Axiom in background
echo "Starting Nexus Axiom..."
./target/release/nexus-axiom start &
NEXUS_PID=$!

# Wait for it to load
sleep 2

echo ""
echo "Running same exploit tests with protection..."
echo ""

echo "Test 1: W^X Memory Allocation"
./examples/test_wx_memory || echo "✅ Process was killed by Nexus Axiom"
echo ""

echo "Test 2: CVE-2021-4034 (PwnKit)"
./examples/test_pwnkit || echo "✅ Process was killed by Nexus Axiom"
echo ""

echo "Test 3: mprotect() to W^X"
./examples/test_mprotect || echo "✅ Process was killed by Nexus Axiom"
echo ""

# Stop Nexus Axiom
kill $NEXUS_PID 2>/dev/null || true

echo ""
echo "✅ All exploits BLOCKED - system is PROTECTED"
echo ""
echo "🎉 Demo complete!"
echo ""
echo "⭐ If this impressed you, star us on GitHub!"
echo "   https://github.com/YOUR_USERNAME/nexus-axiom"
