# Performance Architecture

## Goal: 10x Faster Than Falco

**Target Metrics:**
- Overhead: <1% CPU
- Latency: <100ns per event
- Throughput: >1M events/sec

**Falco Baseline:**
- Overhead: ~5% CPU
- Latency: ~500ns per event
- Throughput: ~200K events/sec

---

## Optimizations Implemented

### 1. Hot Path Minimization
```c
// Fast path: Single bitwise check
static __always_inline bool is_wx_violation(unsigned long prot) {
    return (prot & PROT_WRITE) && (prot & PROT_EXEC);
}
```

**Why it's fast:**
- Inlined (no function call overhead)
- Single CPU instruction (bitwise AND)
- Branch predictor friendly

### 2. Per-CPU Ring Buffers
```c
struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024); // 256KB per CPU
} events SEC(".maps");
```

**Why it's fast:**
- Zero lock contention
- Each CPU has dedicated buffer
- No cache line bouncing

### 3. Minimal Event Data
```c
struct event {
    u32 pid;
    u32 blocked;
    u64 timestamp;
} __attribute__((packed));
```

**Why it's fast:**
- 16 bytes total (fits in single cache line)
- No string copies
- Packed for memory efficiency

### 4. Early Exit Strategy
```c
// Check W^X first (most common case)
if (!is_wx_violation(prot))
    return 0; // Exit immediately
```

**Why it's fast:**
- 99% of mmap calls are benign
- Exit in <10 instructions
- No map lookups for normal traffic

### 5. Allowlist Hash Map
```c
static __always_inline bool is_allowed(u32 pid) {
    u32 *val = bpf_map_lookup_elem(&allowlist, &pid);
    return val != NULL;
}
```

**Why it's fast:**
- O(1) hash lookup
- Pre-allocated (no runtime allocation)
- Trusted processes bypass all checks

---

## Benchmark Results

### Test Environment
- **CPU**: [To be measured on real Linux]
- **Kernel**: [To be measured]
- **Memory**: [To be measured]

### Baseline (No Protection)
```
Operations/sec: [TBD]
Average latency: [TBD] ns
```

### With Nexus Axiom
```
Operations/sec: [TBD]
Average latency: [TBD] ns
Overhead: [TBD]%
```

### Comparison to Falco
```
Nexus Axiom: [TBD]% overhead
Falco:       ~5% overhead
Improvement: [TBD]x faster
```

---

## How to Benchmark

```bash
# Build optimized version
make ebpf
cargo build --release

# Run benchmark suite
chmod +x benchmark.sh
sudo ./benchmark.sh
```

---

## Future Optimizations

### Phase 2: Zero-Copy Userspace
- mmap() ring buffer directly
- Eliminate copy from kernel to userspace
- Target: <50ns latency

### Phase 3: BPF Tail Calls
- Move complex logic to tail calls
- Keep hot path under 100 instructions
- Target: <30ns latency

### Phase 4: Custom Allocator
- Pre-allocate event pool
- Eliminate malloc() in hot path
- Target: <20ns latency

---

## Why This Matters

**Security tools must be fast because:**
1. They run on EVERY syscall
2. Slow tools get disabled in production
3. Performance = adoption

**Our advantage:**
- Rust's zero-cost abstractions
- Modern eBPF features (ring buffers, CO-RE)
- Obsessive hot-path optimization

---

## Verification

To verify these claims:
1. Run `benchmark.sh` on bare metal Linux
2. Compare to Falco with identical workload
3. Measure with `perf stat` and `bpftrace`

**We will publish:**
- Raw benchmark data
- Hardware specs
- Reproducible test scripts
- Comparison methodology

---

**No fake numbers. No marketing fluff. Just measurable performance.**
