#!/bin/bash
# WSL2 Setup Script for Nexus Axiom

echo "🛡️  Nexus Axiom - WSL2 Setup"
echo "============================"
echo ""

# Install dependencies (skip kernel headers for WSL2)
echo "📦 Installing dependencies..."
sudo apt-get install -y clang llvm libbpf-dev build-essential

echo ""
echo "✅ Dependencies installed"
echo ""

# Check if we have the files we need
if [ ! -f "ebpf/nexus_real.bpf.c" ]; then
    echo "❌ Error: ebpf/nexus_real.bpf.c not found"
    exit 1
fi

echo "🔧 Building Nexus Axiom..."
make clean
make

echo ""
echo "✅ Build complete!"
echo ""
echo "Next steps:"
echo "  sudo ./demo.sh              # Run demo"
echo "  cd cve_tests && make        # Build test exploits"
echo "  ./test_pwnkit               # Test without protection"
