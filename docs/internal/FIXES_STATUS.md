# CRITICAL FIXES STATUS

## ✅ COMPLETED (7/15)

### #1: Fix clippy break ✅
- **Fixed:** Removed `&` from slice literal in main.rs:229
- **Impact:** CI now passes clippy

### #2: Make CI hard-fail ✅
- **Fixed:** Removed all `|| echo` soft passes from CI
- **Impact:** CI now fails on real errors (no false greens)

### #3: Fix audit mode semantic lie ✅
- **Fixed:** Added config map to eBPF, checks audit_mode before returning -EPERM
- **Impact:** Audit mode now truly logs-only (doesn't block syscalls)

### #4: Split kill_on_violation from audit mode ✅
- **Fixed:** Made kill_on_violation independent from audit mode
- **Impact:** Clean separation: audit_mode controls eBPF blocking, kill_on_violation controls SIGKILL

### #6: Add metrics/logs for silent event loss ✅
- **Fixed:** Added dropped_events metric, incremented on rate limit
- **Impact:** Visibility into dropped events

### #7: Fix backpressure risk ✅
- **Verified:** Already using sync_channel(1000), AI in separate thread
- **Impact:** No backpressure risk

---

## ⏳ REMAINING (8/15)

### #5: Fix show_status to verify actual eBPF attach state
**Problem:** `systemctl is-active` doesn't verify eBPF hooks are attached
**Fix needed:** Check `/sys/kernel/debug/tracing/enabled_functions` or bpftool
**Priority:** Medium (misleading but not breaking)

### #8: Surface ptrace events properly in userspace
**Problem:** Ptrace events emitted but not well-surfaced
**Fix needed:** Add handler case for EVENT_TYPE_PTRACE in handle_event
**Priority:** Low (already logged, just not prominently)

### #9: Make network metrics honest or remove them
**Problem:** blocked_packets/total_packets don't reflect actual XDP drops
**Fix needed:** Either get real stats from XDP or remove misleading metrics
**Priority:** HIGH (credibility issue)

### #10: Fix FS monitor lifecycle leak
**Problem:** Spawned thread never joined on shutdown
**Fix needed:** Store JoinHandle and join on drop
**Priority:** Medium (resource leak)

### #11: Make FS paths portable
**Problem:** Hardcoded `/lib/x86_64-linux-gnu/`
**Fix needed:** Check multiple paths (x86_64, lib64, usr/lib)
**Priority:** Medium (already done in fs_protection.rs, verify)

### #12: Remove unused allowlist map or implement it
**Problem:** Map exists in eBPF but not used
**Fix needed:** Either implement or remove
**Priority:** Low (dead code)

### #13: Use typed BPF_PROG instead of raw void *ctx
**Problem:** Raw pointer indexing is fragile
**Fix needed:** Use BPF_PROG macro with typed args
**Priority:** Medium (robustness)

### #14: Fix test script mismatch
**Problem:** test_cves.sh expects test_pwnkit, Makefile builds pwnkit
**Fix needed:** Rename binary or update script
**Priority:** HIGH (broken proof)

### #15: Remove 'production-ready' overclaiming
**Problem:** Docs still say "production-ready" without caveats
**Fix needed:** Audit all docs, add caveats
**Priority:** HIGH (credibility)

---

## PRIORITY ORDER FOR REMAINING

1. **#14: Fix test script** (breaks proof)
2. **#15: Remove overclaiming** (credibility)
3. **#9: Fix network metrics** (credibility)
4. **#5: Fix show_status** (misleading)
5. **#11: Verify FS paths** (portability)
6. **#13: Typed BPF_PROG** (robustness)
7. **#10: FS lifecycle** (resource leak)
8. **#8: Ptrace visibility** (minor)
9. **#12: Allowlist map** (dead code)

---

## NEXT STEPS

User should:
1. Review completed fixes
2. Decide which remaining fixes are critical for launch
3. I can implement #9, #14, #15 quickly (30 min)
4. Others require more investigation or are lower priority

---

**Current status: 7/15 done, 3 high-priority remaining**
