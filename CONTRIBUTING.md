# Contributing to Nexus Axiom

Thank you for your interest in contributing! 🎉

## How to Contribute

### Reporting Bugs
- Use GitHub Issues
- Include: OS version, kernel version, steps to reproduce
- Attach logs if possible

### Suggesting Features
- Open a GitHub Issue with the `enhancement` label
- Describe the use case and expected behavior

### Submitting Code

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make your changes**
4. **Test thoroughly**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
5. **Commit with clear messages**
   ```bash
   git commit -m "Add amazing feature"
   ```
6. **Push and create a Pull Request**

## Development Setup

### Prerequisites
```bash
# Ubuntu/Debian
sudo apt-get install clang llvm libbpf-dev linux-headers-$(uname -r)

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build
```bash
cargo build
```

### Test
```bash
cargo test
sudo ./target/debug/nexus-axiom start --audit
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Pass all clippy lints (`cargo clippy`)
- Add tests for new features
- Document public APIs

## Areas We Need Help

- [ ] **ARM64 support** - Port eBPF programs to ARM
- [ ] **BTF generation** - Support older kernels without BTF
- [ ] **Kubernetes operator** - Deploy as DaemonSet
- [ ] **Prometheus metrics** - Export security metrics
- [ ] **More CVE tests** - Add test cases for recent CVEs
- [ ] **Documentation** - Improve docs and examples
- [ ] **Performance** - Optimize hot paths

## Pull Request Guidelines

- Keep PRs focused (one feature/fix per PR)
- Update documentation if needed
- Add tests for new functionality
- Ensure CI passes
- Respond to review feedback

## Community

- **Discord**: [Join our server](https://discord.gg/nexus-axiom)
- **Twitter**: [@nexusaxiom](https://twitter.com/nexusaxiom)
- **Email**: security@nexus-axiom.dev

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.

---

**Thank you for making Nexus Axiom better!** ⭐
