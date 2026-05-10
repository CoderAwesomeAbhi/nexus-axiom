# Complete Setup Guide (From Scratch)

## Current Situation

- ✅ You have Linux VM: `nexus-axiom@nexus-axiom-VirtualBox`
- ✅ You're in: `~/nexus-axiom/nexus-axiom`
- ❌ Files I created are on Windows machine
- ❌ Need to transfer files to Linux VM

## Option 1: Transfer Files (Recommended)

### Step 1: On Windows, Create Archive

```powershell
# In PowerShell on Windows
cd C:\Users\abhij\nexus-axiom-final
tar -czf nexus-axiom-fixed.tar.gz *
```

### Step 2: Transfer to Linux VM

**Method A: Shared Folder (VirtualBox)**
1. VirtualBox → Settings → Shared Folders
2. Add folder: `C:\Users\abhij\nexus-axiom-final`
3. On Linux VM:
```bash
sudo mkdir /mnt/shared
sudo mount -t vboxsf nexus-axiom-final /mnt/shared
cp -r /mnt/shared/* ~/nexus-axiom/nexus-axiom/
```

**Method B: SCP (if SSH enabled)**
```bash
# On Linux VM
scp user@windows-ip:C:/Users/abhij/nexus-axiom-final/* ~/nexus-axiom/nexus-axiom/
```

**Method C: GitHub (Easiest)**
```bash
# On Windows
cd C:\Users\abhij\nexus-axiom-final
git add .
git commit -m "Fixed compilation issues"
git push

# On Linux VM
cd ~/nexus-axiom/nexus-axiom
git pull
```

## Option 2: Manual Fixes (Do This Now)

Since you can't access the files, I'll give you the exact commands to run:

### Step 1: Fix Cargo.toml

```bash
cd ~/nexus-axiom/nexus-axiom

# Backup original
cp Cargo.toml Cargo.toml.backup

# Fix reqwest line
sed -i 's/features = \["json", "blocking", "rustls-tls"\]/features = ["json", "rustls-tls"]/' Cargo.toml

# Add ureq dependency
echo 'ureq = "2.9"' >> Cargo.toml
```

### Step 2: Clean and Build

```bash
# Clean everything
cargo clean
rm -f Cargo.lock

# Update dependencies
cargo update

# Build
cargo build --release
```

### Step 3: If Build Fails, Check Errors

```bash
# Build and save errors
cargo build --release 2>&1 | tee build_errors.txt

# Show me the errors
cat build_errors.txt
```

## Option 3: Start Fresh (Nuclear Option)

If nothing works, start completely fresh:

```bash
# Go to home directory
cd ~

# Backup old code
mv nexus-axiom nexus-axiom-backup

# Clone fresh (if you have GitHub repo)
git clone https://github.com/YOUR_USERNAME/nexus-axiom.git
cd nexus-axiom

# Or create new directory
mkdir -p nexus-axiom-fresh
cd nexus-axiom-fresh
```

Then I'll help you set it up from scratch.

## What You Should Do RIGHT NOW

### Quick Fix (5 minutes)

```bash
cd ~/nexus-axiom/nexus-axiom

# 1. Fix Cargo.toml
nano Cargo.toml

# Find this line:
# reqwest = { version = "0.11", default-features = false, features = ["json", "blocking", "rustls-tls"] }

# Change to:
# reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }

# Add this line after tokio:
# ureq = "2.9"

# Save: Ctrl+X, Y, Enter

# 2. Clean build
cargo clean

# 3. Build
cargo build --release
```

### If That Works

```bash
# Test it
./target/release/nexus-axiom --version

# If you see version number, SUCCESS!
```

### If That Fails

**Copy the EXACT error message and send it to me.**

## Complete Manual Setup (If Starting Fresh)

### 1. Install Dependencies

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    clang \
    llvm \
    libelf-dev \
    linux-headers-$(uname -r) \
    pkg-config \
    libbpf-dev \
    bpftool \
    git

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Check Kernel

```bash
# Check kernel version (need 5.15+)
uname -r

# Check BPF LSM
cat /sys/kernel/security/lsm | grep bpf

# If "bpf" is NOT there:
sudo nano /etc/default/grub
# Add: lsm=bpf to GRUB_CMDLINE_LINUX
sudo update-grub
sudo reboot
```

### 3. Fix Your Current Code

```bash
cd ~/nexus-axiom/nexus-axiom

# Show me what's in this directory
ls -la

# Show me Cargo.toml
cat Cargo.toml | grep -A2 reqwest
cat Cargo.toml | grep -A2 tokio
```

## Tell Me

1. **What's in your current directory?**
   ```bash
   cd ~/nexus-axiom/nexus-axiom
   ls -la
   ```

2. **What's in Cargo.toml?**
   ```bash
   cat Cargo.toml
   ```

3. **What error do you get?**
   ```bash
   cargo build --release 2>&1 | head -50
   ```

Send me the output of these 3 commands and I'll give you exact fix.

## The Fastest Path to Success

### Right Now (2 minutes):

```bash
cd ~/nexus-axiom/nexus-axiom

# Edit Cargo.toml
nano Cargo.toml
```

**Find and change these lines:**

**BEFORE:**
```toml
reqwest = { version = "0.11", default-features = false, features = ["json", "blocking", "rustls-tls"] }
tokio = { version = "1.0", features = ["full"] }
```

**AFTER:**
```toml
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
tokio = { version = "1.0", features = ["full"] }
ureq = "2.9"
```

Save and exit (Ctrl+X, Y, Enter)

```bash
# Clean and build
cargo clean
cargo build --release
```

**If this works, you're done. If not, send me the error.**

## I Need From You

Run these and send output:

```bash
# 1. Where are you?
pwd

# 2. What's here?
ls -la

# 3. What's in Cargo.toml?
cat Cargo.toml | head -30

# 4. Try to build
cargo build --release 2>&1 | head -100
```

Then I'll give you the exact fix for your specific situation.
