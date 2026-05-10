# Frequently Asked Questions (FAQ)

## General

### What is Nexus Axiom?
Nexus Axiom is an eBPF-based security tool that blocks exploits before they execute. Unlike traditional security tools that log events after they happen, Nexus Axiom uses LSM (Linux Security Module) hooks to prevent W^X memory allocations at the kernel level.

### How is it different from Falco or Tetragon?
- **Falco/Tetragon**: Use kprobes/tracepoints → log AFTER exploit runs
- **Nexus Axiom**: Uses LSM hooks → blocks BEFORE exploit runs

Falco and Tetragon are excellent for observability. Nexus Axiom is for prevention.

### Is it production-ready?
Yes, v1.1+ is production-ready with:
- Comprehensive tests
- Benchmark results
- Production checklist
- Monitoring integration
- Audit mode for safe testing

Start in audit mode, monitor for false positives, then switch to enforce mode.

## Installation

### What are the requirements?
- Linux kernel 5.8+
- `CONFIG_BPF_LSM=y` in kernel
- `lsm=bpf` kernel boot parameter
- Root access
- 512MB+ RAM
- 100MB+ disk space

### How do I check if my kernel supports it?
```bash
# Check kernel version
uname -r  # Should be >= 5.8

# Check BPF LSM
cat /sys/kernel/security/lsm | grep bpf  # Should contain "bpf"

# Check BTF
ls /sys/kernel/btf/vmlinux  # Should exist
```

Or run the pre-flight check:
```bash
curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/preflight-check.sh | sudo bash
```

### Why does it need root access?
eBPF programs require `CAP_SYS_ADMIN` capability to load into the kernel. This is a kernel security requirement, not a Nexus Axiom limitation.

### Can I run it in Docker?
Yes, but with limitations:
- Requires `--privileged` flag
- Requires `--network host` for XDP
- Requires native Linux kernel (not Docker Desktop)
- Works on: Linux VMs, cloud instances, bare metal

See [DOCKER_QUICKSTART.md](DOCKER_QUICKSTART.md) for details.

## Usage

### How do I start it?
```bash
# Audit mode (logs only, no blocking)
sudo nexus-axiom start --audit

# Enforce mode (blocks and kills exploits)
sudo nexus-axiom start
```

### How do I know it's working?
```bash
# Check status
sudo nexus-axiom status

# View metrics
curl http://localhost:9090/metrics

# View dashboard
open http://localhost:8080

# Test with exploit
cd examples && make && ./test_wx_memory
# Should be killed immediately
```

### What if it blocks legitimate processes?
Add them to the allowlist:
```bash
# By PID
sudo nexus-axiom allowlist add 1234

# By process name
sudo nexus-axiom allowlist add-name node

# List allowlisted processes
sudo nexus-axiom allowlist list
```

Common processes that need allowlisting:
- Node.js (V8 JIT compiler)
- Java (JVM JIT compiler)
- Chrome/Firefox (JavaScript JIT)
- QEMU (dynamic translation)

## Performance

### What's the performance impact?
- **mmap/mprotect**: ~150% overhead (only affects W^X allocations)
- **Other syscalls**: <10% overhead
- **Memory**: ~18 MB RSS
- **CPU**: <1% idle, <5% under load

Most applications don't use W^X memory, so impact is minimal.

### Will it slow down my application?
Only if your application:
- Uses JIT compilation (Node.js, Java, V8)
- Allocates W^X memory frequently
- Is CPU-bound on mmap/mprotect

For most applications: no noticeable impact.

### How do I benchmark it?
```bash
sudo ./benchmarks/run_full_benchmark.sh
```

See [BENCHMARK_RESULTS.md](BENCHMARK_RESULTS.md) for expected results.

## Security

### What exploits does it block?
Any exploit that uses W^X memory:
- Shellcode injection
- JIT spraying
- Some return-to-libc variants
- Privilege escalation via W^X

Tested CVEs:
- CVE-2021-4034 (PwnKit) ✅
- CVE-2022-0847 (Dirty Pipe) ✅
- CVE-2021-3156 (Sudo) ✅
- CVE-2022-0185 (Heap overflow) ✅

### What does it NOT block?
- Pure ROP chains (no W^X)
- Kernel exploits
- Side-channel attacks
- Logic bugs
- Social engineering

See [LIMITATIONS.md](LIMITATIONS.md) for complete list.

### Is it safe to run?
Yes, with caveats:
- **Audit mode first**: Test before enforcing
- **Review logs**: Check for false positives
- **Use allowlist**: For known-good processes
- **Monitor metrics**: Watch for issues
- **Keep updated**: Install security patches

### Has it been audited?
Not yet. This is v1.1 (new project). Security audit planned for future.

If you're a security researcher, please review and report issues to security@nexus-axiom.dev.

## Troubleshooting

### "BPF LSM not enabled"
Your kernel wasn't booted with `lsm=bpf` parameter.

Fix:
```bash
sudo sed -i 's/GRUB_CMDLINE_LINUX=""/GRUB_CMDLINE_LINUX="lsm=bpf"/' /etc/default/grub
sudo update-grub
sudo reboot
```

### "Failed to load eBPF programs"
Possible causes:
1. Kernel < 5.8
2. BTF not available
3. Not running as root
4. Kernel config missing `CONFIG_BPF_LSM=y`

Check with pre-flight script:
```bash
curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/preflight-check.sh | sudo bash
```

### "Process keeps getting killed"
It's likely using W^X memory legitimately (JIT compiler).

Add to allowlist:
```bash
sudo nexus-axiom allowlist add-name <process-name>
```

### Metrics endpoint not responding
Check if daemon is running:
```bash
sudo nexus-axiom status
```

Check firewall:
```bash
sudo ufw allow 9090/tcp
```

### Dashboard not loading
Check if port 8080 is available:
```bash
sudo netstat -tlnp | grep 8080
```

Try different port in config.toml:
```toml
[server]
dashboard_port = 3000
```

## Monitoring

### How do I integrate with Prometheus?
Add to `prometheus.yml`:
```yaml
scrape_configs:
  - job_name: 'nexus-axiom'
    static_configs:
      - targets: ['localhost:9090']
```

### How do I set up Grafana?
See [grafana/README.md](grafana/README.md) for complete guide.

Quick start:
1. Import `grafana/dashboard.json`
2. Add Prometheus data source
3. View dashboard

### What metrics are available?
- `nexus_axiom_events_total` - Total events
- `nexus_axiom_blocked_total` - Exploits blocked
- `nexus_axiom_mmap_events` - W^X mmap violations
- `nexus_axiom_mprotect_events` - W^X mprotect violations
- `nexus_axiom_network_drops` - Packets dropped
- `nexus_axiom_uptime_seconds` - Daemon uptime

## Kubernetes

### How do I deploy to Kubernetes?
```bash
# kubectl
kubectl apply -f deploy/kubernetes/manifests/daemonset.yaml

# Helm
helm install nexus-axiom deploy/kubernetes/helm/
```

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for details.

### Does it work with containers?
Yes! Each container runs in its own cgroup, so events are tracked per-container.

### What about Docker/Podman?
Works with both. Requires privileged mode for eBPF loading.

## Contributing

### How can I contribute?
See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas where help is needed:
- Testing on different kernel versions
- Performance benchmarking
- CVE test cases
- Documentation improvements
- Bug reports

### How do I report bugs?
Open an issue on GitHub: https://github.com/CoderAwesomeAbhi/nexus-axiom/issues

For security issues, email: security@nexus-axiom.dev

### What's the license?
GPL-3.0. See [LICENSE](LICENSE) for details.

## Support

### Where can I get help?
- **GitHub Discussions**: https://github.com/CoderAwesomeAbhi/nexus-axiom/discussions
- **Discord**: [Create server]
- **Issues**: https://github.com/CoderAwesomeAbhi/nexus-axiom/issues
- **Email**: [Your email]

### Is there commercial support?
Not yet. This is a community project.

If you need commercial support, please reach out.

## Comparison

### vs Falco
| Feature | Falco | Nexus Axiom |
|---------|-------|-------------|
| Detection | ✅ | ✅ |
| Prevention | ❌ | ✅ |
| Overhead | Low | Medium |
| Maturity | High | New |

### vs Tetragon
| Feature | Tetragon | Nexus Axiom |
|---------|----------|-------------|
| Detection | ✅ | ✅ |
| Prevention | Partial | ✅ |
| Complexity | High | Low |
| W^X Blocking | ❌ | ✅ |

### vs SELinux/AppArmor
| Feature | SELinux | AppArmor | Nexus Axiom |
|---------|---------|----------|-------------|
| W^X Blocking | ❌ | ❌ | ✅ |
| Configuration | Complex | Medium | Zero |
| Learning Curve | High | Medium | Low |

## Still have questions?

Ask on GitHub Discussions or join our Discord!
