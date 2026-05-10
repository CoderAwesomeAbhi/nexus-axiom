# Benchmark Results

## ⚠️ Status: Pending Real Hardware Testing

**These benchmarks have not yet been run on production hardware.**

The numbers below are **placeholders** and should not be used for evaluation.

## How to Run Benchmarks

```bash
cd benchmarks
sudo ./run_full_benchmark.sh
```

This will generate real measurements for:
- mmap() latency overhead
- mprotect() latency overhead  
- open() latency overhead
- Process spawn overhead
- Memory usage
- CPU usage under load

## Planned Benchmark Scenarios

### 1. **Performance Overhead**
- Baseline system performance (no Nexus Axiom)
- With Nexus Axiom in audit mode
- With Nexus Axiom in enforce mode
- CPU, memory, and latency impact

### 2. **Exploit Detection**
- Test against known CVE exploits
- Measure detection rate
- Measure false positive rate
- Measure detection latency

### 3. **Comparison Benchmarks**
- Nexus Axiom vs Falco
- Nexus Axiom vs Tetragon
- Nexus Axiom vs SELinux
- Feature comparison and performance

### 4. **Stress Testing**
- High event rate (10K+ events/sec)
- Long-running stability (24+ hours)
- Memory leak detection
- Resource exhaustion scenarios

### 5. **Network Performance (XDP)**
- Packet processing throughput
- Blocked IP lookup latency
- Impact on network bandwidth

## Contributing Benchmark Results

If you run benchmarks on your hardware, please contribute results:

1. Run: `sudo ./benchmarks/run_full_benchmark.sh`
2. Save output to a file
3. Open GitHub Issue with:
   - Hardware specs (CPU, RAM, disk)
   - Kernel version
   - OS distribution
   - Workload description
   - Raw benchmark output

## Expected Results (Estimates)

Based on similar eBPF security tools, we expect:

- **CPU Overhead**: <1% under normal load
- **Memory Usage**: 10-20 MB for daemon
- **Latency Impact**: <10% for hooked syscalls
- **Detection Rate**: >99% for W^X exploits
- **False Positives**: <0.1% with proper allowlist

**These are estimates only. Real benchmarks will be published here once validated.**

---

## Why No Results Yet?

This project is in active development. Benchmark infrastructure exists but needs:
1. Testing on multiple hardware configurations
2. Validation of measurement methodology
3. Peer review of results
4. Comparison with baseline measurements

**We prioritize honest reporting over marketing claims.**

If you'd like to help run benchmarks, please open a GitHub Issue or Discussion.

---

*Last Updated: 2026-05-05*  
*Status: Awaiting real hardware testing*
