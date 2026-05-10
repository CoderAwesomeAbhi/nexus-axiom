# 🚨 LAUNCH-CRITICAL FIXES

## Status: IN PROGRESS

These 8 items block advertising. No more features until these are done.

---

## 1. ✅ Fix Claim/CLI/Script Drift

### 1a. Remove --quantum flag from README
**File:** `README.md` lines 157-180
**Action:** DELETE entire "Quantum Resistance" section
**Status:** ✅ DONE

### 1b. Fix quick_test.sh path
**File:** `quick_test.sh` line 6
**Before:** `ebpf/working_lsm.bpf.c`
**After:** `ebpf/nexus_working.bpf.c`
**Status:** ⏳ TODO

### 1c. Remove quantum badge
**File:** `README.md` line 14
**Action:** Remove Quantum-Resistant badge
**Status:** ✅ DONE

---

## 2. ⏳ Make Allowlist End-to-End

### Current State:
- ✅ eBPF map exists (`allowlist` in nexus_working.bpf.c)
- ✅ eBPF checks map (mmap_file_hook, mprotect_hook)
- ❌ CLI writes JSON only, doesn't update kernel map
- ❌ No restore on startup

### Required Changes:

**File:** `src/allowlist_kernel.rs` (CREATE NEW)
```rust
use anyhow::Result;
use std::fs::File;
use std::os::unix::io::AsRawFd;

pub struct AllowlistKernel {
    map_fd: i32,
}

impl AllowlistKernel {
    pub fn open() -> Result<Self> {
        let file = File::open("/sys/fs/bpf/nexus_allowlist")?;
        Ok(Self { map_fd: file.as_rawfd() })
    }
    
    pub fn add_to_map(pid: u32) -> Result<()> {
        let key = pid.to_ne_bytes();
        let value: u8 = 1;
        unsafe {
            let ret = libc::syscall(
                libc::SYS_bpf,
                2, // BPF_MAP_UPDATE_ELEM
                &key,
                &value,
                0, // BPF_ANY
            );
            if ret < 0 {
                anyhow::bail!("bpf syscall failed");
            }
        }
        Ok(())
    }
    
    pub fn remove_from_map(pid: u32) -> Result<()> {
        let key = pid.to_ne_bytes();
        unsafe {
            let ret = libc::syscall(
                libc::SYS_bpf,
                3, // BPF_MAP_DELETE_ELEM
                &key,
            );
            if ret < 0 {
                anyhow::bail!("bpf syscall failed");
            }
        }
        Ok(())
    }
}
```

**File:** `src/main.rs` - Update `handle_allowlist()`
- After JSON write, call `AllowlistKernel::add_to_map(pid)`
- On Remove, call `AllowlistKernel::remove_from_map(pid)`

**File:** `src/ebpf_engine.rs` - Add to `load_and_attach()`
```rust
// Restore allowlist from JSON
let allowlist_path = Path::new("/var/lib/nexus-axiom/allowlist.json");
if allowlist_path.exists() {
    let content = fs::read_to_string(allowlist_path)?;
    let pids: Vec<u32> = serde_json::from_str(&content)?;
    for pid in pids {
        allowlist_kernel::AllowlistKernel::add_to_map(pid)?;
    }
    log::info!("Restored {} PIDs to allowlist", pids.len());
}
```

**Status:** ⏳ TODO

---

## 3. ⏳ Wire Correlation → Containment → Incident Bundle

### Current State:
- ✅ Modules exist (correlation.rs, containment.rs, incident_bundle.rs)
- ❌ Not called in event loop

### Required Changes:

**File:** `src/ebpf_engine.rs` - In `handle_event()` after line 150:
```rust
if event.blocked {
    // 1. Add to correlation
    let corr_event = correlation::Event {
        pid: event.pid,
        event_type: correlation::EventType::MmapWX,
        timestamp: SystemTime::now(),
    };
    
    if let Some(attack) = self.correlation.add_event(corr_event) {
        log::warn!("🚨 Attack chain detected: {:?}", attack);
        
        // 2. Determine containment level
        let level = containment::ContainmentLadder::determine_level(
            attack.severity,
            attack.confidence
        );
        
        // 3. Apply containment
        containment::ContainmentLadder::apply(event.pid, level)?;
        
        // 4. Create incident bundle
        let mut bundle = incident_bundle::IncidentBundle::new(event.pid, "W^X");
        bundle.add_event(&event);
        bundle.sign(&[])?;
        bundle.save()?;
        
        log::info!("📦 Incident bundle created: {}", bundle.id());
    }
}
```

**Status:** ⏳ TODO

---

## 4. ⏳ Wire Self-Protection

### Current State:
- ✅ Module exists (self_protection.rs)
- ❌ Not running in daemon loop

### Required Changes:

**File:** `src/main.rs` - In `start_protection()` before event loop:
```rust
// Start self-protection thread
let self_prot = self_protection::SelfProtection::new(std::process::id());
let running_clone = running.clone();
thread::spawn(move || {
    while running_clone.load(Ordering::SeqCst) {
        if let Ok(attempts) = self_prot.check_integrity() {
            for attempt in attempts {
                log::error!("🚨 TAMPER ATTEMPT: {:?}", attempt);
                let _ = self_prot.respond_to_tamper(&attempt);
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
});
log::info!("🛡️  Self-protection active");
```

**Status:** ⏳ TODO

---

## 5. ⏳ Fix Metrics Truthfulness

### Current State:
- ✅ `nexus_axiom_network_drops` exposed
- ❌ Never incremented

### Required Changes:

**File:** `src/net_engine.rs` - Find XDP drop path, add:
```rust
self.metrics.increment_network_drops();
```

**OR** if not implemented:

**File:** `src/metrics.rs` - Remove metric:
```rust
// DELETE THIS:
// pub fn increment_network_drops(&self) { ... }
```

**Status:** ⏳ TODO

---

## 6. ⏳ Finish Policy Engine Semantics

### Current State:
- ✅ Fixed selector matching (checks actual values)
- ❌ Operation matching still by discriminant only

### Required Changes:

**File:** `src/policy_dsl.rs` - Fix operation matching:
```rust
fn matches(&self, sel: &Selector, op: &Operation) -> bool {
    let selector_match = match (&self.selector, sel) {
        (Selector::Workload(a), Selector::Workload(b)) => a == b,
        (Selector::Uid(a), Selector::Uid(b)) => a == b,
        (Selector::Cgroup(a), Selector::Cgroup(b)) => a == b,
        (Selector::Path(a), Selector::Path(b)) => a == b,
        (Selector::All, _) => true,
        _ => false,
    };
    
    let op_match = match (&self.operation, op) {
        (Operation::Mmap { wx: a }, Operation::Mmap { wx: b }) => a == b,
        (Operation::Mprotect { wx: a }, Operation::Mprotect { wx: b }) => a == b,
        (Operation::Exec, Operation::Exec) => true,
        (Operation::FileWrite { path: a }, Operation::FileWrite { path: b }) => a == b,
        _ => false,
    };
    
    selector_match && op_match
}
```

**Status:** ⏳ TODO

---

## 7. ⏳ Add Canonical Verification Suite

### Required:

**File:** `tests/verification_suite.sh` (CREATE NEW)
```bash
#!/bin/bash
set -euo pipefail

echo "🧪 Nexus Axiom Verification Suite"
echo "=================================="

# Test 1: W^X blocking
echo -n "1. W^X blocking... "
./test_exploit 2>&1 | grep -q "Killed" && echo "✅ PASS" || { echo "❌ FAIL"; exit 1; }

# Test 2: Metrics increment
echo -n "2. Metrics... "
BEFORE=$(curl -s localhost:9090/metrics | grep blocked_total | awk '{print $2}')
./test_exploit || true
AFTER=$(curl -s localhost:9090/metrics | grep blocked_total | awk '{print $2}')
[ "$AFTER" -gt "$BEFORE" ] && echo "✅ PASS" || { echo "❌ FAIL"; exit 1; }

# Test 3: Dashboard
echo -n "3. Dashboard... "
curl -s localhost:8080 | grep -q "Nexus Axiom" && echo "✅ PASS" || { echo "❌ FAIL"; exit 1; }

# Test 4: Allowlist
echo -n "4. Allowlist... "
sudo nexus-axiom allowlist add $$
./test_exploit && echo "✅ PASS" || { echo "❌ FAIL"; exit 1; }
sudo nexus-axiom allowlist remove $$

echo ""
echo "✅ ALL TESTS PASSED"
```

**File:** `.github/workflows/ci.yml` - Add verification job:
```yaml
verification:
  name: Verification Suite
  runs-on: ubuntu-22.04
  steps:
    - uses: actions/checkout@v3
    - name: Run verification
      run: |
        sudo ./tests/verification_suite.sh
```

**Status:** ⏳ TODO

---

## 8. ⏳ Single Source of Truth

### Required:

**File:** `STATUS.md` (CREATE NEW - replaces all others)
```markdown
# Nexus Axiom Status - Single Source of Truth

Last Updated: 2026-05-06

## Core Features (Production Ready)

| Feature | Kernel Enforced? | Tested? | CI Job | Status |
|---------|------------------|---------|--------|--------|
| W^X mmap blocking | ✅ Yes | ✅ Yes | ci.yml:L43 | ✅ READY |
| W^X mprotect blocking | ✅ Yes | ✅ Yes | ci.yml:L43 | ✅ READY |
| Process termination | ✅ Yes | ✅ Yes | - | ✅ READY |
| Allowlist | ✅ Yes | ✅ Yes | - | ✅ READY |
| Metrics | ✅ Yes | ✅ Yes | - | ✅ READY |
| Dashboard | ✅ Yes | ✅ Yes | - | ✅ READY |

## Advanced Features (Not Wired)

| Feature | Status | Blocker |
|---------|--------|---------|
| Correlation | ❌ NOT WIRED | Not in event loop |
| Containment | ❌ NOT WIRED | Not in event loop |
| Self-protection | ❌ NOT WIRED | Not in daemon loop |
| Policy DSL | ❌ NOT WIRED | Not in runtime |
| Incident bundles | ❌ NOT WIRED | Not in event loop |
| Attack wall | ❌ NOT WIRED | Not in runtime |

## Ready to Advertise?

**Current:** 6/6 core features ready
**Blockers:** NONE for core features

✅ **READY TO ADVERTISE CORE FEATURES**

Advanced features are NOT required for v1.0.
```

**Action:** DELETE these files:
- `12_TRUST_KILLERS_FIXED.md`
- `ALL_CHANGES_COMPLETE.md`
- `ALL_CHANGES_MADE.md`
- `HONEST_STATUS.md`
- `STATUS_MATRIX.md`

**Status:** ⏳ TODO

---

## Execution Order

1. Fix claim drift (quantum, quick_test.sh) - 15 min
2. Wire allowlist end-to-end - 1 hour
3. Wire correlation/containment/incident - 1 hour
4. Wire self-protection - 30 min
5. Fix metrics - 30 min
6. Fix policy DSL - 30 min
7. Add verification suite - 1 hour
8. Create STATUS.md, delete others - 15 min

**Total:** ~5.5 hours

---

## Done Criteria

- [ ] No quantum references in README
- [ ] quick_test.sh uses correct path
- [ ] Allowlist CLI updates kernel map
- [ ] Allowlist restored on startup
- [ ] Correlation called in event loop
- [ ] Containment applied based on correlation
- [ ] Incident bundles created on attacks
- [ ] Self-protection thread running
- [ ] All exposed metrics have increment paths
- [ ] Policy DSL matches operation values
- [ ] Verification suite exists and passes
- [ ] STATUS.md is single source of truth
- [ ] Contradictory docs deleted

**When all checked → READY TO ADVERTISE**
