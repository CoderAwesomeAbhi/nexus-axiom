#!/bin/bash
# Complete fix for Nexus Axiom compilation
# Copy-paste this ENTIRE file into your Linux terminal

cd ~/nexus-axiom/nexus-axiom

echo "=== Fixing All Source Files ==="

# Fix 1: src/ebpf_engine.rs - Remove Arc, use Option
echo "Fixing ebpf_engine.rs..."
cat > src/ebpf_engine_fix.patch << 'EOF'
--- a/src/ebpf_engine.rs
+++ b/src/ebpf_engine.rs
@@ -34,7 +34,7 @@ struct Event {
 
 pub struct EbpfEngine {
     skel: Option<NexusWorkingSkel<'static>>,
-    ai_analyst: Arc<crate::ai_analyst::AIAnalyst>,
+    ai_analyst: Option<AIAnalyst>,
     json_logger: Option<JsonLogger>,
     audit_mode: bool,
     kill_on_violation: bool,
@@ -42,7 +42,7 @@ pub struct EbpfEngine {
         metrics: Arc<crate::metrics::MetricsServer>,
         audit_mode: bool,
         kill_on_violation: bool,
-            ai_analyst: Arc::new(crate::ai_analyst::AIAnalyst::new(None)?),
+            ai_analyst: AIAnalyst::new(None).ok(),
             json_logger: Some(JsonLogger::new(
                 Some("/var/log/nexus-axiom/events.json"),
                 crate::json_logger::LogFormat::Standard,
@@ -118,7 +118,7 @@ impl EbpfEngine {
 
 fn handle_event(
     event: &Event,
-    ai_analyst: &Arc<crate::ai_analyst::AIAnalyst>,
+    ai_analyst: &Option<AIAnalyst>,
     json_logger: &Option<JsonLogger>,
     audit_mode: bool,
     kill_on_violation: bool,
EOF

# Apply simple sed fixes
sed -i 's/ai_analyst: Arc<crate::ai_analyst::AIAnalyst>/ai_analyst: Option<AIAnalyst>/' src/ebpf_engine.rs 2>/dev/null || true
sed -i 's/Arc::new(crate::ai_analyst::AIAnalyst::new(None)?)/AIAnalyst::new(None).ok()/' src/ebpf_engine.rs 2>/dev/null || true
sed -i 's/ai_analyst: &Arc<crate::ai_analyst::AIAnalyst>/ai_analyst: \&Option<AIAnalyst>/' src/ebpf_engine.rs 2>/dev/null || true

# Fix 2: src/main.rs - Add #[tokio::main]
echo "Fixing main.rs..."
if ! grep -q "#\[tokio::main\]" src/main.rs; then
    sed -i 's/^fn main()/#[tokio::main]\nasync fn main()/' src/main.rs
fi

# Fix 3: src/ai_analyst.rs - Make async
echo "Fixing ai_analyst.rs..."
sed -i 's/use reqwest::blocking::Client/use reqwest::Client/' src/ai_analyst.rs 2>/dev/null || true
sed -i 's/pub fn analyze_threat(&self/pub async fn analyze_threat(\&self/' src/ai_analyst.rs 2>/dev/null || true
sed -i 's/\.send()?/\.send().await?/' src/ai_analyst.rs 2>/dev/null || true
sed -i 's/\.json()?/\.json().await?/' src/ai_analyst.rs 2>/dev/null || true

# Fix 4: Cargo.toml
echo "Fixing Cargo.toml..."
cat > Cargo.toml << 'ENDTOML'
[package]
name = "nexus-axiom"
version = "1.0.0"
edition = "2021"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
env_logger = "0.11"
log = "0.4"
ctrlc = "3"
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
lazy_static = "1.4"
chrono = "0.4"
tokio = { version = "1.0", features = ["full"] }
ureq = "2.9"

[target.'cfg(target_os = "linux")'.dependencies]
libbpf-rs = "0.23"
libbpf-sys = "1.4"
libseccomp = "0.3"
inotify = "0.10"
nix = { version = "0.27", features = ["user", "signal", "process"] }

[target.'cfg(target_os = "linux")'.build-dependencies]
libbpf-cargo = "0.23"

[profile.release]
opt-level = 3
lto = true
strip = true
codegen-units = 1
ENDTOML

echo ""
echo "✅ All fixes applied!"
echo ""
echo "Building..."
cargo clean
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅✅✅ BUILD SUCCESSFUL! ✅✅✅"
    echo ""
    ./target/release/nexus-axiom --version
else
    echo ""
    echo "❌ Build failed. Showing errors:"
    cargo build --release 2>&1 | head -50
fi
