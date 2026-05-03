# Add this to the top of README.md after the demo section:

## 🚀 One-Command Install

```bash
curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash
```

**That's it!** The installer automatically:
- ✅ Checks system requirements (kernel 5.8+, BPF LSM)
- ✅ Installs all dependencies
- ✅ Compiles Nexus Axiom
- ✅ Loads eBPF LSM hooks
- ✅ Creates systemd service
- ✅ Tests exploit blocking

**After installation:**
```bash
sudo systemctl start nexus-axiom    # Start protection
sudo systemctl status nexus-axiom   # Check status
cd /opt/nexus-axiom/cve_tests && ./test_pwnkit  # Test it
```

---
