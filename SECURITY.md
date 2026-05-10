# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.1.x   | :white_check_mark: |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

**DO NOT** open a public GitHub issue for security vulnerabilities.

### How to Report

1. **Email**: security@nexus-axiom.dev (or create this email)
2. **Subject**: "Security Vulnerability in Nexus Axiom"
3. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Fix Timeline**: Critical issues within 30 days
- **Disclosure**: Coordinated disclosure after fix is released

### Scope

**In Scope**:
- eBPF program vulnerabilities
- Privilege escalation in daemon
- Bypass of W^X blocking
- Memory corruption bugs
- Denial of service attacks
- Information disclosure

**Out of Scope**:
- Social engineering
- Physical access attacks
- Vulnerabilities in dependencies (report to upstream)
- Issues requiring root access (tool requires root by design)

## Security Considerations

### Threat Model

**What Nexus Axiom Protects Against**:
- W^X memory exploits (shellcode injection)
- Privilege escalation via W^X
- Known CVEs using W^X patterns

**What Nexus Axiom Does NOT Protect Against**:
- Kernel exploits
- Pure ROP chains (no W^X)
- Side-channel attacks
- Social engineering
- Supply chain attacks

### Running Nexus Axiom Safely

1. **Review the code** before running as root
2. **Test in staging** before production
3. **Monitor logs** for unexpected behavior
4. **Use allowlist** for known-good processes
5. **Keep updated** to latest version

### Known Limitations

See [LIMITATIONS.md](LIMITATIONS.md) for complete list.

## Security Features

- **Seccomp isolation**: Daemon restricts its own syscalls
- **Minimal privileges**: Only requires CAP_SYS_ADMIN for eBPF
- **No network access**: Daemon doesn't make outbound connections (except optional AI)
- **Read-only eBPF**: Programs can't modify kernel memory
- **Audit mode**: Test without blocking

## Bug Bounty

Currently no formal bug bounty program. Security researchers who responsibly disclose vulnerabilities will be:
- Credited in CHANGELOG
- Mentioned in security advisory
- Given priority support

## Past Security Issues

None reported yet (project is new).

## Security Best Practices

### For Users
- Run in audit mode first
- Review logs before enforcing
- Test with your workload
- Keep kernel updated
- Monitor metrics

### For Contributors
- No unsafe Rust without justification
- All eBPF code must be reviewed
- Add tests for security-critical code
- Document security implications
- Follow secure coding guidelines

## Contact

- **Security Email**: security@nexus-axiom.dev
- **PGP Key**: [Add PGP key]
- **GitHub**: https://github.com/CoderAwesomeAbhi/nexus-axiom/security

## Acknowledgments

We thank the following security researchers for responsible disclosure:

(None yet - be the first!)
