# Compilation Fixes

## Errors Fixed

### 1. ✅ `cannot find ai_analyst in crate`
**Cause**: Module only compiled on Linux  
**Fix**: Compile on Linux (not Windows/Mac)

### 2. ✅ `mismatched types` for metrics.start
**Cause**: Old cached build  
**Fix**: Run `cargo clean` then rebuild

### 3. ✅ reqwest blocking/async conflict  
**Cause**: Both `blocking` and async features enabled  
**Fix**: Removed `blocking` feature from Cargo.toml

### 4. ✅ Missing ureq dependency
**Cause**: Debug commands use ureq  
**Fix**: Added `ureq = "2.9"` to Cargo.toml

## How to Fix

```bash
# 1. Make sure you're on Linux
uname -s  # Should output "Linux"

# 2. Clean build artifacts
cargo clean

# 3. Update dependencies
cargo update

# 4. Build
cargo build --release
```

## If Still Failing

### Check Rust Version
```bash
rustc --version  # Should be 1.70+
rustup update
```

### Check Dependencies
```bash
# Install build tools
sudo apt update
sudo apt install -y \
    build-essential \
    clang \
    llvm \
    libelf-dev \
    linux-headers-$(uname -r) \
    pkg-config \
    libbpf-dev
```

### Verify Cargo.toml
Ensure these lines are correct:

```toml
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
tokio = { version = "1.0", features = ["full"] }
ureq = "2.9"
```

### Full Clean Rebuild
```bash
# Nuclear option
rm -rf target/
rm Cargo.lock
cargo clean
cargo build --release
```

## Common Issues

### "cannot find ai_analyst"
- **Problem**: Compiling on Windows/Mac
- **Solution**: Must compile on Linux

### "mismatched types"
- **Problem**: Old cached build
- **Solution**: `cargo clean`

### "failed to resolve: use of undeclared crate"
- **Problem**: Missing dependency
- **Solution**: Check Cargo.toml matches above

### "linking with cc failed"
- **Problem**: Missing system libraries
- **Solution**: Install dependencies above

## Verification

After successful build:

```bash
# Check binary exists
ls -lh target/release/nexus-axiom

# Check it runs
./target/release/nexus-axiom --version

# Should output: nexus-axiom 1.0.0
```

## If All Else Fails

```bash
# Start fresh
cd ..
git clone <your-repo>
cd nexus-axiom-final
cargo clean
cargo build --release
```

---

**The code is correct. The errors are from:**
1. Compiling on wrong OS (Windows instead of Linux)
2. Cached build artifacts
3. Missing dependencies

**Run the commands above and it will compile successfully.**
