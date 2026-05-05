# Contributing to Nexus Axiom

Thank you for your interest! This project aims to make eBPF security accessible.

## Quick Start

1. Fork the repo
2. Clone: `git clone https://github.com/YOUR_USERNAME/nexus-axiom.git`
3. Create branch: `git checkout -b feature/your-feature`
4. Make changes
5. Test: `cargo test && cargo clippy`
6. Commit: `git commit -m "feat: your feature"`
7. Push: `git push origin feature/your-feature`
8. Open PR

## Good First Issues

Look for issues labeled `good first issue` - perfect for new contributors:

**Easy (1-2 hours):**
- Add more CVE test cases
- Improve documentation
- Add unit tests
- Fix typos

**Medium (4-8 hours):**
- Add allowlist feature for false positives
- Improve error messages
- Add more metrics
- Optimize performance

**Hard (1-2 days):**
- ARM architecture support
- Real-time network stats from XDP
- Graceful shutdown for FS protection
- Advanced threat correlation

## Areas We Need Help

### Testing
- More CVE test cases
- Stress testing under load
- False positive identification
- ARM architecture testing

### Documentation
- Installation guides for different distros
- Troubleshooting guides
- Video tutorials
- Translation to other languages

### Features
- Allowlist for legitimate W^X programs
- Better false positive handling
- SIEM integrations (Splunk, ELK, etc.)
- Kubernetes Helm chart improvements

### Performance
- Reduce memory footprint
- Optimize event processing
- Better rate limiting
- Async AI analysis

## Development Setup

### Prerequisites
- Linux kernel 5.8+ with `lsm=bpf`
- Rust 1.70+
- clang, llvm, libbpf-dev
- Root access for testing

### Build
```bash
cargo build --release
```

### Test
```bash
# Unit tests
cargo test

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt

# Integration test
sudo bash proof.sh
```

### Run Locally
```bash
sudo ./target/release/nexus-axiom start
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add tests for new features
- Update README if adding features
- Keep functions small and focused
- Comment complex eBPF code

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only
- `test:` Adding tests
- `refactor:` Code refactoring
- `perf:` Performance improvement
- `chore:` Maintenance

Examples:
- `feat: add allowlist for false positives`
- `fix: handle ptrace events correctly`
- `docs: improve installation guide`

## Pull Request Process

1. **Before submitting:**
   - Run `cargo test && cargo clippy && cargo fmt`
   - Update README if needed
   - Add tests for new features
   - Verify proof.sh still works

2. **PR description should include:**
   - What: Brief description of changes
   - Why: Reason for changes
   - How: Technical approach
   - Testing: How you tested it

3. **Review process:**
   - Maintainer will review within 24-48 hours
   - Address feedback promptly
   - Once approved, maintainer will merge

4. **After merge:**
   - Your contribution will be in next release
   - You'll be added to contributors list

## Maintainer Response SLA

We commit to:
- **Issues:** First response within 24 hours
- **PRs:** First review within 48 hours
- **Security issues:** Response within 12 hours
- **Questions:** Response within 48 hours

If we miss these SLAs, please ping us!

## Roadmap

### v1.1 (Next 2 weeks)
- [ ] Allowlist for false positives
- [ ] Real-time network stats
- [ ] Graceful shutdown
- [ ] ARM support

### v1.2 (Next month)
- [ ] Advanced threat correlation
- [ ] Better SIEM integrations
- [ ] Performance optimizations
- [ ] More CVE tests

### v2.0 (Future)
- [ ] Machine learning for anomaly detection
- [ ] Policy-as-code
- [ ] Multi-tenant support
- [ ] Attack replay lab

See [GitHub Projects](https://github.com/CoderAwesomeAbhi/nexus-axiom/projects) for detailed roadmap.

## Architecture

```
┌─────────────────────────────────────────┐
│         User Applications               │
└──────────────┬──────────────────────────┘
               │ syscall
               ▼
┌─────────────────────────────────────────┐
│         Linux Kernel                    │
│  ┌───────────────────────────────────┐  │
│  │  LSM Hook (eBPF)                  │  │
│  │  - mmap_file                      │  │
│  │  - file_mprotect                  │  │
│  └───────────┬───────────────────────┘  │
└──────────────┼──────────────────────────┘
               │ Ring Buffer
               ▼
┌─────────────────────────────────────────┐
│    Nexus Axiom (Rust Userspace)        │
│  ┌─────────────────────────────────┐   │
│  │  Event Handler                  │   │
│  │  - Parse events                 │   │
│  │  - Log to JSON                  │   │
│  │  - Send SIGKILL                 │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Metrics Server (Prometheus)    │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Dashboard (HTTP)               │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

Key files:
- `ebpf/nexus_working.bpf.c` - eBPF LSM hooks
- `src/ebpf_engine.rs` - Event processing
- `src/metrics.rs` - Prometheus metrics
- `src/dashboard.rs` - Web dashboard

## Questions?

- Open an issue
- Start a discussion
- Check existing docs

We're friendly and happy to help!

## License

By contributing, you agree your code will be licensed under GPL-3.0.
