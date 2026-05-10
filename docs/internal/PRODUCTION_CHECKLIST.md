# Production Validation Checklist

## Pre-Deployment
- [ ] Kernel 5.8+ with BPF LSM enabled
- [ ] BTF available at /sys/kernel/btf/vmlinux
- [ ] All tests pass: `cargo test`
- [ ] Benchmarks run successfully

## Staging Tests
- [ ] W^X blocking works (test_wx_memory killed)
- [ ] Metrics endpoint responds (localhost:9090)
- [ ] Dashboard loads (localhost:8080)
- [ ] Allowlist commands work
- [ ] Graceful shutdown works (Ctrl+C)

## Production Deployment
- [ ] Start in audit mode first
- [ ] Monitor for 24h for false positives
- [ ] Add JIT processes to allowlist if needed
- [ ] Switch to enforce mode
- [ ] Configure Prometheus/Grafana
- [ ] Set up alerts for exploits blocked

## Rollback Plan
1. `sudo systemctl stop nexus-axiom`
2. Review logs: `journalctl -u nexus-axiom`
3. Fix issues in staging
4. Re-deploy

**Approved by**: _______________  
**Date**: _______________
