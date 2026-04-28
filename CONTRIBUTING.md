# Contributing to Nexus Axiom

Thank you for your interest in contributing!

## How to Contribute

### Reporting Bugs
- Check if the issue already exists
- Include kernel version, distro, and hardware specs
- Provide steps to reproduce
- Include relevant logs

### Suggesting Features
- Open an issue with [Feature Request] tag
- Explain the use case
- Consider performance impact

### Pull Requests
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test on real Linux (not WSL2)
5. Run `cargo fmt` and `cargo clippy`
6. Submit PR with clear description

## Development Setup

```bash
# Install dependencies
sudo apt-get install clang llvm libbpf-dev linux-headers-$(uname -r)

# Build
make all

# Test
make test
```

## Code Style

- Rust: Follow `rustfmt` defaults
- C: Follow Linux kernel style
- Comments: Explain *why*, not *what*

## Testing

- Test on multiple kernel versions (5.7, 5.10, 5.15, 6.0+)
- Verify no performance regression
- Check for memory leaks

## Areas We Need Help

- [ ] Testing on different distros
- [ ] More LSM hooks (file access, network)
- [ ] BTF/CO-RE support
- [ ] Documentation improvements
- [ ] Performance optimizations

## Questions?

Open a discussion or issue. We're here to help!
