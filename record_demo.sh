#!/bin/bash

# Script to record the W^X exploit demo for asciinema
# This creates a clean, professional recording

echo "🎬 Nexus Axiom Demo Recording Script"
echo "====================================="
echo ""
echo "This script will record a demo showing:"
echo "  1. W^X exploit succeeding without protection"
echo "  2. Same exploit being blocked with Nexus Axiom"
echo ""
echo "Prerequisites:"
echo "  • asciinema installed (apt install asciinema)"
echo "  • Nexus Axiom compiled (cargo build --release)"
echo "  • Running as root"
echo ""

if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run as root: sudo ./record_demo.sh"
    exit 1
fi

if ! command -v asciinema &> /dev/null; then
    echo "❌ asciinema not found. Install it:"
    echo "   apt install asciinema"
    exit 1
fi

if [ ! -f "./target/release/nexus-axiom" ]; then
    echo "❌ Nexus Axiom binary not found. Build it:"
    echo "   cargo build --release"
    exit 1
fi

echo ""
echo "Press Enter to start recording..."
read

# Record the demo
asciinema rec -t "Nexus Axiom - W^X Exploit Blocking Demo" \
    --overwrite \
    exploit-demo.cast \
    -c "./demo.sh"

echo ""
echo "✅ Recording saved to: exploit-demo.cast"
echo ""
echo "To upload to asciinema.org:"
echo "  asciinema upload exploit-demo.cast"
echo ""
echo "To play locally:"
echo "  asciinema play exploit-demo.cast"
