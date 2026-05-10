# Changelog

All notable changes to Nexus Axiom will be documented in this file.

## [1.1.0] - 2026-05-05

### Added
- **Allowlist Management**: CLI commands to manage process allowlist
  - `nexus-axiom allowlist add <pid>` - Add process by PID
  - `nexus-axiom allowlist add-name <name>` - Add by process name
  - `nexus-axiom allowlist remove <pid>` - Remove from allowlist
  - `nexus-axiom allowlist list` - Show all allowlisted processes
  - `nexus-axiom allowlist clear` - Clear entire allowlist
- **Docker Support**: Quick-start with Dockerfile and docker-compose
- **Grafana Integration**: Pre-built dashboard and Prometheus config
- **Comprehensive Tests**: Unit and integration tests with CI validation
- **AI Threat Analysis**: Async background analysis with rule-based fallback
- **Better Error Messages**: Actionable guidance for common issues
- **Production Checklist**: Validation checklist for production deployments
- **Benchmark Suite**: Automated performance benchmarking scripts

### Fixed
- **vmlinux.h Generation**: Auto-generate during build if missing
- **CI eBPF Compilation**: CI now actually compiles eBPF programs
- **Graceful Shutdown**: Proper cleanup on SIGTERM/SIGINT
- **XDP Metrics**: Real metrics from eBPF maps (wired up)

### Changed
- **Documentation**: Consolidated into clear structure with DOCS_INDEX.md
- **AI Analyst**: Enabled as async background task (no longer disabled)
- **Error Messages**: More helpful with specific troubleshooting steps

### Performance
- mmap() overhead: ~150% (measured)
- Memory usage: ~18 MB RSS
- CPU usage: <1% idle, <5% under load

## [1.0.0] - 2026-05-03

### Added
- Initial release
- W^X memory blocking via LSM hooks
- XDP network filtering
- Prometheus metrics endpoint
- Web dashboard
- JSON logging (multiple formats)
- Kubernetes DaemonSet and Helm chart
- File system protection
- Container awareness (cgroup tracking)
- Seccomp isolation for daemon
- CVE test suite (12+ exploits)

### Security
- Blocks CVE-2021-4034 (PwnKit)
- Blocks CVE-2022-0847 (Dirty Pipe)
- Blocks CVE-2021-3156 (Sudo heap overflow)
- Blocks CVE-2022-0185 (Heap overflow)

## [Unreleased]

### Planned
- ARM64 support and testing
- Real-world case studies
- Security audit by third party
- Performance optimizations
- Extended CVE test coverage
- Community Discord server
