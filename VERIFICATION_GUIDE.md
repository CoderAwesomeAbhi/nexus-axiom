# 🚀 Official Verification & Testing Guide

To prove that Nexus Axiom genuinely blocks kernel-level exploits with <1% overhead, you must run it in a real Linux environment with eBPF LSM enabled. 

## 💰 Choosing Your Environment

| Option | Cost | Setup Effort | Recommended For |
|--------|------|--------------|-----------------|
| **WSL2 (Windows)** | Free | Medium | Local developers on Windows |
| **Oracle Cloud** | Always Free | Medium | Persistent cloud testing (24GB RAM) |
| **Killercoda** | Free | Low | Quick 5-minute browser demo |
| **AWS EC2** | ~$10/mo | Low | Professional benchmarking |

---

## 💻 Option 1: WSL2 (Windows Subsystem for Linux) - FREE
If you are on Windows, you can test locally.
1. Ensure you have WSL2 installed (`wsl --install`).
2. Update to the latest kernel: `wsl --update`.
3. Enable BPF LSM by creating/editing `C:\Users\<YourUser>\.wslconfig`:
   ```ini
   [wsl2]
   kernelCommandLine=lsm=apparmor,bpf
   ```
4. Restart WSL: `wsl --shutdown`.

## ☁️ Option 2: Oracle Cloud "Always Free" - FREE
1. Sign up for [Oracle Cloud Free Tier](https://www.oracle.com/cloud/free/).
2. Launch an **Ampere A1** instance (Ubuntu 22.04).
3. These kernels typically support BPF LSM out of the box.

## 🌐 Option 3: Killercoda (Browser) - FREE
1. Open the [Killercoda Ubuntu Playground](https://killercoda.com/playgrounds/scenario/ubuntu).
2. You get an instant root terminal. Skip to "Install Dependencies".

---

## Step 1: Provision the Environment (Paid EC2 Alternative)
1. Log into AWS Console and launch an **EC2 t3.medium** instance.
2. Select **Ubuntu 22.04 LTS** or **Ubuntu 24.04 LTS**.
3. SSH into the instance.

## Step 2: Install Dependencies
Run the following to install the required eBPF and Rust toolchains:
```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y build-essential clang llvm libbpf-dev linux-headers-$(uname -r) pkg-config m4 gcc-multilib

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

## Step 3: Enable Kernel LSM Hooks
Ubuntu may need BPF LSM explicitly enabled in the boot parameters.
```bash
# Check if bpf is in the active LSMs
cat /sys/kernel/security/lsm
# If you don't see 'bpf', edit GRUB:
sudo nano /etc/default/grub
# Append to GRUB_CMDLINE_LINUX: lsm=lockdown,yama,integrity,apparmor,bpf
sudo update-grub
sudo reboot
```

## Step 4: Build and Run Nexus Axiom
```bash
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom.git
cd nexus-axiom
make ebpf
cargo build --release

# Run in blocking mode
sudo ./target/release/nexus-axiom start
```

## Step 5: Generate the Proof Video (Optional but highly recommended)
1. Install `asciinema` (`sudo apt install asciinema`).
2. Run `asciinema rec proof.cast`.
3. In one terminal, run `sudo ./target/release/nexus-axiom start`.
4. In another terminal, compile and run the test exploit:
   ```bash
   gcc -o test_exploit test_exploit.c
   ./test_exploit
   ```
5. You will see Nexus Axiom immediately terminate the exploit. Press `Ctrl+D` to stop recording.
6. Upload the cast to [asciinema.org](https://asciinema.org) or convert it to a GIF.
7. Put this GIF in your README!

## Step 6: Generate Real Benchmarks
Run the built-in benchmarking script to get real CPU/Latency metrics.
```bash
cd benchmarks
sudo ./benchmark_vs_falco.sh
```
Copy the output from this script and place it directly into the `BENCHMARKS.md` file.

---
**Why this matters:** Security engineers do not trust simulated videos or theoretical overhead. By following this guide, you prove beyond a shadow of a doubt that Nexus Axiom works exactly as advertised.
