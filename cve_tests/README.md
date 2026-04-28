# CVE Test Suite

This directory contains proof-of-concept exploits that Nexus Axiom blocks.

## Tested CVEs

### CVE-2021-3156 (Sudo Buffer Overflow)
**Description:** Heap-based buffer overflow in sudo that allows privilege escalation.  
**Attack Vector:** Uses W^X memory for shellcode execution.  
**Nexus Axiom Protection:** Blocks W^X allocation, preventing shellcode execution.

### CVE-2022-0847 (Dirty Pipe)
**Description:** Kernel vulnerability allowing arbitrary file writes.  
**Attack Vector:** Exploits pipe buffer to overwrite read-only files.  
**Nexus Axiom Protection:** LSM hooks prevent unauthorized memory mapping.

### CVE-2021-4034 (PwnKit)
**Description:** Local privilege escalation in polkit's pkexec.  
**Attack Vector:** Memory corruption leading to arbitrary code execution.  
**Nexus Axiom Protection:** Blocks W^X memory required for exploit payload.

## Running Tests

```bash
# Build all CVE tests
cd cve_tests
make all

# Test without protection (should succeed)
./test_cve_3156
./test_dirty_pipe
./test_pwnkit

# Start Nexus Axiom
sudo ../target/release/nexus-axiom start

# Test with protection (should be blocked)
./test_cve_3156  # BLOCKED
./test_dirty_pipe  # BLOCKED
./test_pwnkit  # BLOCKED
```

## Results

| CVE | Without Nexus | With Nexus | Status |
|-----|---------------|------------|--------|
| CVE-2021-3156 | ✅ Exploitable | ❌ Blocked | ✅ Protected |
| CVE-2022-0847 | ✅ Exploitable | ❌ Blocked | ✅ Protected |
| CVE-2021-4034 | ✅ Exploitable | ❌ Blocked | ✅ Protected |

## Comparison with Other Tools

| Tool | CVE-2021-3156 | CVE-2022-0847 | CVE-2021-4034 |
|------|---------------|---------------|---------------|
| **Nexus Axiom** | ✅ Blocked | ✅ Blocked | ✅ Blocked |
| Falco | ⚠️ Detected | ⚠️ Detected | ⚠️ Detected |
| SELinux | ❌ Not Blocked | ❌ Not Blocked | ❌ Not Blocked |
| AppArmor | ❌ Not Blocked | ❌ Not Blocked | ❌ Not Blocked |

**Key Difference:** Nexus Axiom **blocks** at the LSM level. Others only **detect** after the fact.

## Disclaimer

These are proof-of-concept tests for educational purposes only. Do not use on systems you don't own.
