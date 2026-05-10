# COPY-PASTE THIS INTO YOUR LINUX VM

## Step 1: Go to your code directory
```bash
cd ~/nexus-axiom/nexus-axiom
```

## Step 2: Backup Cargo.toml
```bash
cp Cargo.toml Cargo.toml.backup
```

## Step 3: Fix Cargo.toml (copy-paste this entire block)
```bash
cat > Cargo.toml << 'ENDOFFILE'
[package]
name = "nexus-axiom"
version = "1.0.0"
edition = "2021"
description = "Real eBPF security that actually kills exploits"
license = "GPL-3.0"
repository = "https://github.com/CoderAwesomeAbhi/nexus-axiom"
keywords = ["security", "ebpf", "lsm", "exploit", "kernel"]
categories = ["os::linux-apis", "development-tools"]

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
ENDOFFILE
```

## Step 4: Clean and build
```bash
cargo clean
cargo build --release
```

## Step 5: Test
```bash
./target/release/nexus-axiom --version
```

---

# If You Get Errors

## Error: "cannot find ai_analyst"
**You're not on Linux. This MUST be run on Linux VM.**

## Error: "failed to resolve"
**Missing dependencies. Run:**
```bash
sudo apt update
sudo apt install -y build-essential clang llvm libelf-dev linux-headers-$(uname -r) pkg-config libbpf-dev
```

## Error: "linking with cc failed"
**Missing system libraries. Run:**
```bash
sudo apt install -y libbpf-dev libelf-dev
```

## Still Failing?
**Send me the output of:**
```bash
cargo build --release 2>&1 | head -100
```
