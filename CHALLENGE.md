# 🎯 Break My Tool Challenge

**Bounty:** $100 USD (first 3 successful bypasses)  
**Status:** OPEN  
**Deadline:** None (ongoing)

---

## Challenge

**Can you bypass Nexus Axiom's W^X memory protection?**

If you can execute arbitrary code despite Nexus Axiom blocking W^X memory, you win.

---

## Rules

### What Counts as a Bypass

✅ **Valid bypass:**
- Exploit executes arbitrary code (e.g., spawns shell, reads /etc/shadow)
- Nexus Axiom is running in enforce mode
- No modifications to Nexus Axiom code or config
- Reproducible on clean Ubuntu 22.04 VM

❌ **Not a bypass:**
- ROP chains (we don't claim to block these - see LIMITATIONS.md)
- Kernel exploits (runs in kernel context, outside LSM scope)
- Side-channel attacks (Spectre, Meltdown, etc.)
- Exploits that don't use W^X memory
- Attacks on Nexus Axiom itself (report as security issue instead)
- Social engineering or physical access

### Test Environment

Must reproduce on:
- **OS:** Ubuntu 22.04.3 LTS (clean install)
- **Kernel:** 6.5.0-14-generic or later
- **Nexus Axiom:** Latest main branch
- **Config:** Default config.toml (enforce mode)

### Submission Requirements

1. **Exploit code** (C, Python, or shell script)
2. **Reproduction steps** (exact commands to run)
3. **Proof** (screenshot or video showing code execution)
4. **Explanation** (how the bypass works)
5. **Public report** (blog post, GitHub gist, or tweet)

---

## How to Participate

### Step 1: Set Up Test Environment

```bash
# Fresh Ubuntu 22.04 VM
# Enable LSM eBPF
sudo nano /etc/default/grub
# Add: GRUB_CMDLINE_LINUX="lsm=bpf,apparmor,yama"
sudo update-grub
sudo reboot

# Install Nexus Axiom
curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash

# Verify it's running
sudo systemctl status nexus-axiom

# Test baseline protection
sudo bash /opt/nexus-axiom/proof.sh
# Should show exploits being blocked
```

### Step 2: Develop Your Bypass

Try to execute arbitrary code despite W^X blocking. Ideas:
- JIT spray attacks
- File-backed executable mappings
- Shared memory tricks
- Race conditions
- Novel memory mapping techniques

### Step 3: Submit Your Bypass

Open a GitHub issue with:
- Title: `[BYPASS] Brief description`
- Label: `bypass-challenge`
- Content: All submission requirements above

### Step 4: Get Paid

If your bypass is valid:
1. We'll verify it within 48 hours
2. We'll fix the vulnerability
3. We'll send you $100 via PayPal/Venmo/crypto
4. We'll credit you in SECURITY.md

---

## Example Submission

**Title:** `[BYPASS] JIT spray via file-backed mmap`

**Exploit code:**
```c
// bypass_jit.c
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>

int main() {
    // Create file with shellcode
    int fd = open("/tmp/shellcode", O_RDWR | O_CREAT, 0600);
    char shellcode[] = "\x48\x31\xc0..."; // execve("/bin/sh")
    write(fd, shellcode, sizeof(shellcode));
    
    // Map as executable (file-backed, not anonymous)
    void *addr = mmap(NULL, 4096, PROT_READ | PROT_EXEC,
                      MAP_PRIVATE, fd, 0);
    
    // Execute
    ((void(*)())addr)();
    
    return 0;
}
```

**Reproduction:**
```bash
gcc bypass_jit.c -o bypass_jit
sudo systemctl start nexus-axiom
./bypass_jit
# Expected: Shell spawned despite Nexus Axiom running
```

**Proof:**
[Screenshot showing shell prompt]

**Explanation:**
Nexus Axiom only blocks anonymous W^X mappings. File-backed executable mappings are allowed because they're used by legitimate programs (e.g., loading .so files). This bypass writes shellcode to a file, then maps it as executable.

**Fix suggestion:**
Check file-backed mappings too, but allowlist known-good paths (/lib, /usr/lib, etc.).

---

## Current Bypasses

### None yet!

Be the first to find one.

---

## Known Non-Bypasses

These have been tried and don't work:

### 1. ROP Chains
**Why it doesn't work:** ROP uses existing executable memory (libc gadgets). Nexus Axiom doesn't claim to block this (see LIMITATIONS.md).

### 2. Kernel Exploits
**Why it doesn't work:** Kernel exploits run in kernel context, outside LSM scope. Nexus Axiom can't block kernel-level code execution.

### 3. Timing Attacks
**Why it doesn't work:** LSM hooks run synchronously before syscall completes. No TOCTOU gap.

### 4. Forking Before Kill
**Why it doesn't work:** LSM hook blocks the mmap/mprotect syscall itself. Process never gets W^X memory, so fork doesn't help.

---

## FAQ

**Q: Can I modify Nexus Axiom's config?**  
A: No. Must use default config.toml (enforce mode).

**Q: Can I use a different kernel version?**  
A: Yes, as long as it's 5.8+ and supports LSM eBPF.

**Q: What if I find a bug in Nexus Axiom itself?**  
A: Report it as a security issue (see SECURITY.md). Different from bypass challenge.

**Q: Can I submit multiple bypasses?**  
A: Yes! $100 for each unique bypass (first 3 total across all participants).

**Q: What if someone already submitted my bypass?**  
A: First valid submission wins. Check existing issues before submitting.

**Q: Can I collaborate with others?**  
A: Yes, but split the bounty among yourselves.

**Q: What if my bypass only works sometimes?**  
A: Must be reliably reproducible (>90% success rate).

**Q: Can I use 0-days?**  
A: Yes, but please be responsible. Don't publish exploits for unpatched vulnerabilities.

---

## Responsible Disclosure

If you find a security issue in Nexus Axiom itself (not a bypass):
- Email: [TO BE FILLED - create security@nexus-axiom.dev]
- Or: Open a private security advisory on GitHub
- We'll respond within 24 hours
- We'll credit you in SECURITY.md

---

## Leaderboard

| Rank | Researcher | Bypass | Date | Bounty |
|------|------------|--------|------|--------|
| 🥇 | TBD | TBD | TBD | $100 |
| 🥈 | TBD | TBD | TBD | $100 |
| 🥉 | TBD | TBD | TBD | $100 |

---

## Sponsor This Challenge

Want to increase the bounty? Contact us:
- GitHub: [@CoderAwesomeAbhi](https://github.com/CoderAwesomeAbhi)
- Twitter: [TO BE FILLED]

---

**Good luck! 🎯**

*Last updated: [TO BE FILLED]*
