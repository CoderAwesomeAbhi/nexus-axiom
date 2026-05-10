#!/bin/bash
# This script applies ALL fixes to your source code
# Run this on your Linux VM

set -e

echo "=== Applying All Source Code Fixes ==="

# 1. Fix src/ebpf_engine.rs - Change Arc<AIAnalyst> to Option<AIAnalyst>
echo "Fixing src/ebpf_engine.rs..."
sed -i 's/ai_analyst: Arc<crate::ai_analyst::AIAnalyst>/ai_analyst: Option<AIAnalyst>/' src/ebpf_engine.rs
sed -i 's/ai_analyst: Arc::new(crate::ai_analyst::AIAnalyst::new(None)?)/ai_analyst: AIAnalyst::new(None).ok()/' src/ebpf_engine.rs
sed -i 's/fn handle_event.*Arc<crate::ai_analyst::AIAnalyst>/fn handle_event(event: \&Event, ai_analyst: \&Option<AIAnalyst>/' src/ebpf_engine.rs

# 2. Fix src/metrics.rs - Ensure start() returns Result
echo "Fixing src/metrics.rs..."
if ! grep -q "pub fn start(&self, port: u16) -> Result<(), String>" src/metrics.rs; then
    sed -i 's/pub fn start(&self, port: u16)/pub fn start(\&self, port: u16) -> Result<(), String>/' src/metrics.rs
fi

# 3. Fix src/dashboard.rs - Ensure start() returns Result  
echo "Fixing src/dashboard.rs..."
if ! grep -q "pub fn start(&self, port: u16) -> Result<(), String>" src/dashboard.rs; then
    sed -i 's/pub fn start(&self, port: u16)/pub fn start(\&self, port: u16) -> Result<(), String>/' src/dashboard.rs
fi

# 4. Fix src/ai_analyst.rs - Change to async
echo "Fixing src/ai_analyst.rs..."
sed -i 's/use reqwest::blocking::Client/use reqwest::Client/' src/ai_analyst.rs
sed -i 's/pub fn analyze_threat(&self/pub async fn analyze_threat(\&self/' src/ai_analyst.rs
sed -i 's/\.send()/\.send().await/' src/ai_analyst.rs
sed -i 's/\.json()/\.json().await/' src/ai_analyst.rs

# 5. Add #[tokio::main] to main.rs
echo "Fixing src/main.rs..."
if ! grep -q "#\[tokio::main\]" src/main.rs; then
    sed -i 's/^fn main()/\#[tokio::main]\nasync fn main()/' src/main.rs
fi

echo ""
echo "✅ All fixes applied!"
echo ""
echo "Now run:"
echo "  cargo clean"
echo "  cargo build --release"
