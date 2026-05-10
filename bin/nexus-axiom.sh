#!/bin/bash
# Nexus Axiom NPM wrapper

set -e

INSTALL_DIR="/usr/local/bin"
BINARY_NAME="nexus-axiom"

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ Nexus Axiom only supports Linux (kernel 5.8+)"
    exit 1
fi

# Check if binary is installed
if [ ! -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    echo "⚠️  Nexus Axiom binary not found. Installing..."
    echo "📥 Downloading installer..."
    curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash
    exit 0
fi

# Pass all arguments to the actual binary
sudo "$INSTALL_DIR/$BINARY_NAME" "$@"
