# 🎯 4-WEEK PLAN TO "ADVERTISING-ONLY" LEVEL

## ✅ CURRENT STATUS: NOT READY

**Trust Blockers Still Present:**
1. ❌ CI doesn't fail (|| true everywhere)
2. ❌ Allowlist not enforced in kernel
3. ❌ Modules not wired to event loop
4. ❌ README claims don't match reality
5. ❌ Tests are simulations, not proofs
6. ❌ Metrics not incremented
7. ❌ Script drift (broken references)

**Bottom Line:** Close on vision, not on proof rigor.

---

## 📅 WEEK 1: TRUTH & FOUNDATION

### Day 1: Fix CI (2 hours)
```bash
# Remove all || true from .github/workflows/ci.yml
# Already done in code, verify it works:
git add .github/workflows/ci.yml
git commit -m "fix: CI now fails on errors"
git push
# Verify CI actually fails on errors
```

### Day 2: Fix README Claims (3 hours)
**Remove these claims:**
- ❌ `--quantum` flag (doesn't exist)
- ❌ "All features integrated" (they're not)
- ❌ Any feature marked ✅ that isn't wired

**Add honest status:**
```markdown
## ✅ Production-Ready (Wired & Tested):
1. W^X blocking (LSM hooks)
2. Process termination (SIGKILL)
3. Metrics (8 counters)
4. Dashboard (live UI)

## 🚧 Code Exists But Not Wired:
1. Policy engine
2. Correlation
3. Containment ladder
4. Self-protection
5. Attack wall
6. Incident bundles
```

### Day 3: Fix Script Drift (2 hours)
```bash
# Fix or remove broken scripts:
rm quick_test.sh  # References non-existent files
# Update all script references in docs
# Test every script actually runs
```

### Day 4: Wire Allowlist to eBPF (4 hours)
**File:** `ebpf/nexus_working.bpf.c`

Add allowlist lookup:
```c
// Add allowlist map
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1024);
    __type(key, u32);    // PID
    __type(value, u8);   // 1 = allowed
} allowlist SEC(".maps");

// In mmap_file hook:
u32 pid = bpf_get_current_pid_tgid() >> 32;
u8 *allowed = bpf_map_lookup_elem(&allowlist, &pid);
if (allowed && *allowed == 1) {
    return 0;  // Allow
}
```

**Test:**
```bash
# Add PID to allowlist
sudo nexus-axiom allowlist add 1234

# Verify it's in eBPF map
sudo bpftool map dump name allowlist

# Test process 1234 can do W^X
```

### Day 5: Verification Matrix (3 hours)
Create `VERIFICATION_MATRIX.md`:

| Feature | Test Command | Expected Output | CI Job |
|---------|-------------|-----------------|--------|
| W^X blocking | `./test_exploit` | "Killed" | ci.yml:L45 |
| Allowlist | `sudo nexus-axiom allowlist add $$; ./test_exploit` | Success | ci.yml:L52 |
| Metrics | `curl localhost:9090/metrics \| grep blocked` | `nexus_axiom_blocked_total 1` | ci.yml:L60 |

---

## 📅 WEEK 2: WIRE CORE MODULES

### Day 6-7: Wire Correlation (8 hours)
**File:** `src/ebpf_engine.rs`

In `handle_event()`:
```rust
// Add event to correlation engine
let corr_event = correlation::Event {
    pid: event.pid,
    event_type: correlation::EventType::MmapWX,
    timestamp: SystemTime::now(),
};

if let Some(attack_chain) = self.correlation.add_event(corr_event) {
    log::warn!("🚨 Attack chain: {:?}", attack_chain.pattern);
    // Use containment ladder
    let level = containment_ladder::determine_level(
        attack_chain.severity,
        attack_chain.confidence
    );
    containment_ladder::apply(event.pid, level)?;
}
```

**Test:**
```bash
# Run multiple exploits quickly
for i in {1..3}; do ./test_exploit & done

# Check logs for "Attack chain detected"
sudo journalctl -u nexus-axiom | grep "Attack chain"
```

### Day 8-9: Wire Incident Bundles (8 hours)
**File:** `src/ebpf_engine.rs`

On every block:
```rust
if event.blocked {
    // Create incident bundle
    let mut bundle = incident_bundle::IncidentBundle::new(
        event.pid,
        "W^X mmap"
    );
    bundle.add_event(/* ... */);
    bundle.sign(&signing_key);
    
    // Save to disk
    let json = serde_json::to_string(&bundle)?;
    fs::write(
        format!("/var/log/nexus-axiom/incidents/{}.json", bundle.id),
        json
    )?;
}
```

**Test:**
```bash
./test_exploit
ls /var/log/nexus-axiom/incidents/
cat /var/log/nexus-axiom/incidents/*.json | jq .
```

### Day 10: Fix Metrics (4 hours)
**File:** `src/net_engine.rs`

In XDP drop path:
```rust
// When dropping packet
self.metrics.increment_network_drops();
```

**Test:**
```bash
# Block an IP
sudo nexus-axiom network block 1.2.3.4

# Send traffic from that IP
# Check metric increments
curl localhost:9090/metrics | grep network_drops
```

---

## 📅 WEEK 3: PROOF-GRADE TESTS

### Day 11-12: Real CVE Reproductions (8 hours)
**Not simulations. Real exploits.**

**PwnKit (CVE-2021-4034):**
```bash
# Download real exploit
wget https://github.com/arthepsy/CVE-2021-4034/raw/main/cve-2021-4034.c
gcc -o pwnkit cve-2021-4034.c

# Test WITHOUT Nexus
./pwnkit
# Should succeed

# Test WITH Nexus
sudo nexus-axiom start &
./pwnkit
# Should be killed

# Verify in logs
sudo journalctl -u nexus-axiom | grep "Killed.*pwnkit"
```

### Day 13-14: Deterministic Test Harness (8 hours)
**File:** `tests/deterministic_test.sh`

```bash
#!/bin/bash
set -e

# Start Nexus
sudo nexus-axiom start &
sleep 3

# Test 1: W^X mmap blocked
./test_exploit 2>&1 | grep -q "Killed" || exit 1

# Test 2: Allowlist works
sudo nexus-axiom allowlist add $$
./test_exploit && echo "PASS: Allowlist works" || exit 1

# Test 3: Metrics increment
BEFORE=$(curl -s localhost:9090/metrics | grep blocked_total | awk '{print $2}')
./test_exploit || true
AFTER=$(curl -s localhost:9090/metrics | grep blocked_total | awk '{print $2}')
[ $AFTER -gt $BEFORE ] || exit 1

echo "ALL TESTS PASSED"
```

### Day 15: Golden Outputs (4 hours)
**File:** `tests/golden/`

```bash
# Capture expected outputs
./test_exploit 2>&1 | tee tests/golden/exploit_blocked.txt
curl localhost:9090/metrics > tests/golden/metrics.txt

# In CI, diff against golden
diff <(./test_exploit 2>&1) tests/golden/exploit_blocked.txt
```

---

## 📅 WEEK 4: POLISH & VERIFY

### Day 16-17: Integration Tests (8 hours)
**File:** `tests/integration_test.rs`

```rust
#[test]
fn test_end_to_end_blocking() {
    // Start daemon
    let daemon = start_daemon();
    
    // Run exploit
    let output = Command::new("./test_exploit").output().unwrap();
    
    // Verify killed
    assert!(!output.status.success());
    
    // Verify logged
    let logs = read_logs();
    assert!(logs.contains("Killed"));
    
    // Verify metric
    let metrics = fetch_metrics();
    assert!(metrics.blocked_total > 0);
}
```

### Day 18: Remove Unwired Features (4 hours)
```bash
# Move to experimental branch
git checkout -b experimental
git checkout main

# Remove from main:
rm src/autopilot.rs
rm src/policy_dsl.rs
# (Keep only what's wired)

# Update README to only list wired features
```

### Day 19: Final Verification (4 hours)
**Run through entire verification matrix:**

```bash
# Clone fresh
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom
cd nexus-axiom

# Build
cargo build --release

# Run tests
cargo test

# Run integration tests
./tests/deterministic_test.sh

# Verify CI is green
# Verify all claims in README work
```

### Day 20: Documentation Audit (4 hours)
**Check every claim:**
- [ ] Every feature in README has working test
- [ ] Every command in docs actually works
- [ ] Every metric is actually incremented
- [ ] No broken script references
- [ ] CI badge is green

---

## ✅ DONE CRITERIA

### You're ready to advertise when:
1. ✅ CI fails on errors (no || true)
2. ✅ Allowlist enforced in kernel
3. ✅ Correlation + containment wired
4. ✅ Incident bundles generated
5. ✅ All metrics increment
6. ✅ Real CVE tests pass
7. ✅ Deterministic test harness
8. ✅ Golden outputs match
9. ✅ Verification matrix complete
10. ✅ No claim/reality drift

### The Test:
**"Can a skeptical security engineer clone, run one command, see green CI, reproduce blocks, and verify claims without trust leaps?"**

If YES → Ready to advertise
If NO → Keep fixing

---

## 🎯 EXECUTION CHECKLIST

### Week 1:
- [ ] Remove || true from CI
- [ ] Fix README claims
- [ ] Fix script drift
- [ ] Wire allowlist to eBPF
- [ ] Create verification matrix

### Week 2:
- [ ] Wire correlation
- [ ] Wire containment
- [ ] Wire incident bundles
- [ ] Fix all metrics

### Week 3:
- [ ] Real CVE reproductions
- [ ] Deterministic test harness
- [ ] Golden outputs

### Week 4:
- [ ] Integration tests
- [ ] Remove unwired features
- [ ] Final verification
- [ ] Documentation audit

---

## 🏆 AFTER 4 WEEKS

**You'll have:**
- Internally undeniable codebase
- Proof-grade tests
- No trust leaps required
- Green CI
- Honest documentation

**Then you can advertise with confidence.**

**This is the path. Execute it.**
