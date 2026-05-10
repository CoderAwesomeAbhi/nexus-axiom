#!/bin/bash

echo "=== Fixing Compilation Issues ==="

# Clean build artifacts
echo "1. Cleaning build artifacts..."
cargo clean

# Ensure we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "ERROR: This must be compiled on Linux"
    exit 1
fi

# Check Rust version
echo "2. Checking Rust version..."
rustc --version

# Add tokio dependency if missing
echo "3. Checking Cargo.toml..."
if ! grep -q "tokio.*features.*full" Cargo.toml; then
    echo "ERROR: tokio dependency missing or incomplete in Cargo.toml"
    echo "Add: tokio = { version = \"1\", features = [\"full\"] }"
    exit 1
fi

# Build
echo "4. Building..."
cargo build --release 2>&1 | tee build.log

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed. Check build.log for details"
    exit 1
fi
