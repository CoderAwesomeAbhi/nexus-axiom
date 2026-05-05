#!/bin/bash
# Test install.sh on fresh system

set -e

echo "🧪 Testing install.sh"
echo "====================="

# This should be run on a FRESH Ubuntu 26.04 VM
echo "⚠️  This test should be run on a FRESH Ubuntu 26.04 VM"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

# Run installer
echo "Running installer..."
curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash

# Test if command is available
echo ""
echo "Testing nexus-axiom command..."
if command -v nexus-axiom &> /dev/null; then
    echo "✅ Command installed"
else
    echo "❌ Command not found"
    exit 1
fi

# Test if service is enabled
echo ""
echo "Testing systemd service..."
if systemctl is-enabled nexus-axiom &> /dev/null; then
    echo "✅ Service enabled"
else
    echo "❌ Service not enabled"
    exit 1
fi

# Start service
echo ""
echo "Starting service..."
sudo systemctl start nexus-axiom
sleep 3

# Check if running
if systemctl is-active nexus-axiom &> /dev/null; then
    echo "✅ Service running"
else
    echo "❌ Service not running"
    exit 1
fi

# Test exploit blocking
echo ""
echo "Testing exploit blocking..."
cd /opt/nexus-axiom/cve_tests
if sudo ./test_pwnkit 2>&1 | grep -q "BLOCKED"; then
    echo "✅ Exploit blocking works"
else
    echo "❌ Exploit blocking failed"
    exit 1
fi

# Check dashboard
echo ""
echo "Testing dashboard..."
if curl -s http://localhost:8080 | grep -q "Nexus Axiom"; then
    echo "✅ Dashboard accessible"
else
    echo "❌ Dashboard not accessible"
    exit 1
fi

# Check metrics
echo ""
echo "Testing metrics..."
if curl -s http://localhost:9090/metrics | grep -q "nexus_axiom"; then
    echo "✅ Metrics endpoint working"
else
    echo "❌ Metrics endpoint not working"
    exit 1
fi

echo ""
echo "====================="
echo "✅ Install Script Test PASSED"
echo "====================="
echo ""
echo "All checks passed:"
echo "  • Command installed: YES"
echo "  • Service enabled: YES"
echo "  • Service running: YES"
echo "  • Exploit blocking: YES"
echo "  • Dashboard: YES"
echo "  • Metrics: YES"
echo ""
echo "One-line installer is PRODUCTION READY!"
