# ✅ DO THIS TO LAUNCH (1 Hour Total)

## 🔴 ON UBUNTU VM (30 min)

```bash
# 1. Build
cd /path/to/nexus-axiom-final
cargo build --release

# 2. Test it works
sudo ./target/release/nexus-axiom start &
./test_exploit
# Should be killed!

# 3. Record demo
sudo apt install asciinema
chmod +x demo.sh record_demo.sh
sudo ./record_demo.sh

# 4. Upload video
asciinema upload exploit-demo.cast
# Copy the URL (e.g., https://asciinema.org/a/ABC123)
```

---

## 💻 ON WINDOWS (30 min)

```bash
cd C:\Users\abhij\nexus-axiom-final

# 1. Update website with video ID
# Edit website/index.html line 95
# Replace QEpVzhDNuDNDJyI7 with your video ID

# 2. Publish to NPM
npm login
npm publish

# 3. Push to GitHub
git add .
git commit -m "feat: v1.0.0 - quantum-resistant eBPF security"
git tag v1.0.0
git push origin main --tags

# 4. Enable GitHub Pages
# Go to: Settings → Pages → Source: main → Folder: /website

# 5. Post on LinkedIn
# Copy from QUANTUM_LINKEDIN_POST.md
```

---

## 🎯 THAT'S IT!

Everything else is ready:
- ✅ Code compiles
- ✅ NPM package valid
- ✅ Website ready
- ✅ Docs complete
- ✅ Marketing ready

**Just do the 5 steps above and you're live! 🚀**
