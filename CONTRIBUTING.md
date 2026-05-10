# 🤝 Contributing to Nexus Axiom

**Thank you for your interest!** This project needs help from the community.

---

## 🎓 About This Project

This is built by an 8th grader learning eBPF and kernel security. **I need mentorship and help from experienced developers.**

---

## 🚀 How You Can Help

### 1. 🔍 Code Review
**Most needed:** Security engineers to review the code

**What to review:**
- eBPF code in `ebpf/`
- Rust code in `src/`
- Security implications
- Performance issues
- Best practices

**How to help:**
- Open issues with findings
- Comment on code
- Suggest improvements

### 2. 🧪 Testing
**Most needed:** People to test on different systems

**What to test:**
- Different Linux distros
- Different kernel versions
- Different workloads
- Edge cases

**How to help:**
- Report bugs
- Share benchmarks
- Document issues

### 3. 📖 Documentation
**Most needed:** Better docs and tutorials

**What to write:**
- Setup guides
- Troubleshooting docs
- Architecture explanations
- Use case examples

**How to help:**
- Improve README
- Write tutorials
- Create videos
- Fix typos

### 4. 💻 Code Contributions
**Most needed:** Bug fixes and small features

**Good first issues:**
- Documentation improvements
- Test coverage
- Error handling
- Performance optimization

**How to help:**
- Check [Issues](https://github.com/CoderAwesomeAbhi/nexus-axiom/issues)
- Look for "good first issue" label
- Submit pull requests

### 5. 🎓 Mentorship
**Most needed:** Experienced developers to guide me

**What I need help with:**
- eBPF best practices
- Rust patterns
- Security considerations
- Project management

**How to help:**
- Review my code
- Answer questions
- Suggest improvements
- Pair programming

---

## 🔧 Development Setup

### Prerequisites
```bash
# Ubuntu/Debian
sudo apt-get install -y \
    clang \
    llvm \
    libelf-dev \
    libbpf-dev \
    pkg-config \
    build-essential

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build
```bash
git clone https://github.com/CoderAwesomeAbhi/nexus-axiom
cd nexus-axiom
cargo build
```

### Test
```bash
cargo test
./tests/verify_claims.sh
```

---

## 📝 Pull Request Process

1. **Fork the repo**
2. **Create a branch:** `git checkout -b feature/your-feature`
3. **Make changes**
4. **Test:** `cargo test && cargo check`
5. **Commit:** `git commit -m "Add: your feature"`
6. **Push:** `git push origin feature/your-feature`
7. **Open PR** with description

### PR Guidelines
- ✅ Describe what you changed and why
- ✅ Include tests if applicable
- ✅ Update docs if needed
- ✅ Keep changes focused
- ✅ Be patient (I'm in school during the day)

---

## 🐛 Reporting Bugs

### Before Reporting
- Check existing issues
- Try latest version
- Verify it's reproducible

### What to Include
```markdown
**Environment:**
- OS: Ubuntu 22.04
- Kernel: 5.15.0
- Nexus Axiom version: 1.0.0

**Steps to Reproduce:**
1. Start nexus-axiom
2. Run exploit
3. See error

**Expected:** Process should be killed
**Actual:** Process continues running

**Logs:**
[paste logs here]
```

---

## 💡 Suggesting Features

### Before Suggesting
- Check existing issues
- Consider if it fits the project scope
- Think about implementation

### What to Include
```markdown
**Problem:** Current behavior is X
**Solution:** Proposed behavior is Y
**Use Case:** This helps with Z
**Alternatives:** Could also do A or B
```

---

## 🎯 Priority Areas

### High Priority
1. **Security review** - Most important
2. **Testing on different systems**
3. **Documentation improvements**
4. **Bug fixes**

### Medium Priority
1. **Performance optimization**
2. **More test coverage**
3. **Better error messages**
4. **CI/CD improvements**

### Low Priority
1. **New features**
2. **UI improvements**
3. **Integrations**

---

## 👥 Looking for Co-Maintainers

**I need 1-2 co-maintainers** with:
- Security background
- eBPF experience
- Rust knowledge
- Time to help

**Responsibilities:**
- Review pull requests
- Help with issues
- Guide project direction
- Mentor me

**Benefits:**
- Co-ownership of project
- Credit in all materials
- Learning opportunity
- Open source contribution

**Interested?** Email me or open an issue.

---

## 📚 Resources

### Learning eBPF
- [eBPF.io](https://ebpf.io)
- [BPF Performance Tools](http://www.brendangregg.com/bpf-performance-tools-book.html)
- [libbpf-bootstrap](https://github.com/libbpf/libbpf-bootstrap)

### Learning Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### Security
- [Falco](https://falco.org)
- [Tetragon](https://tetragon.io)
- [Linux Security Modules](https://www.kernel.org/doc/html/latest/security/lsm.html)

---

## 🙏 Thank You

**Every contribution helps:**
- ⭐ Star the repo
- 🐛 Report bugs
- 💻 Submit code
- 📖 Improve docs
- 💬 Answer questions
- 🎓 Mentor me

**I appreciate all help!**

---

## 📧 Contact

**GitHub:** [@CoderAwesomeAbhi](https://github.com/CoderAwesomeAbhi)  
**Issues:** [GitHub Issues](https://github.com/CoderAwesomeAbhi/nexus-axiom/issues)  
**Discussions:** [GitHub Discussions](https://github.com/CoderAwesomeAbhi/nexus-axiom/discussions)

**Response time:** Usually within 24 hours (I'm in school during the day)

---

## 📜 Code of Conduct

**Be kind and respectful.**

- ✅ Constructive feedback
- ✅ Helpful suggestions
- ✅ Patient with beginners
- ❌ Harassment
- ❌ Discrimination
- ❌ Trolling

**Remember:** I'm 13 and learning. Please be patient and helpful.

---

**Thank you for contributing!** 🚀
