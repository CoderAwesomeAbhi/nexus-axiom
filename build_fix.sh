#!/bin/bash
set -e

echo "=== Nexus Axiom Compilation Fix ==="
echo ""

# Check OS
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ ERROR: Must be run on Linux"
    echo "   Current OS: $OSTYPE"
    exit 1
fi

echo "✅ Running on Linux"

# Clean
echo ""
echo "Cleaning build artifacts..."
cargo clean
rm -f Cargo.lock

# Update
echo ""
echo "Updating dependencies..."
cargo update

# Build
echo ""
echo "Building (this may take a few minutes)..."
cargo build --release

# Verify
if [ -f "target/release/nexus-axiom" ]; then
    echo ""
    echo "✅ BUILD SUCCESSFUL!"
    echo ""
    echo "Binary: target/release/nexus-axiom"
    echo "Size: $(du -h target/release/nexus-axiom | cut -f1)"
    echo ""
    echo "Test it:"
    echo "  ./target/release/nexus-axiom --version"
    echo ""
else
    echo ""
    echo "❌ BUILD FAILED"
    echo "Check errors above"
    exit 1
fi
