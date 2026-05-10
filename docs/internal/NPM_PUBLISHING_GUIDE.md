# NPM Publishing Guide for Nexus Axiom

## Prerequisites

1. **NPM Account**
   - Create account at https://www.npmjs.com/signup
   - Verify your email address

2. **NPM CLI Login**
   ```bash
   npm login
   # Enter your username, password, and email
   ```

3. **Verify Login**
   ```bash
   npm whoami
   # Should display your NPM username
   ```

## Pre-Publishing Checklist

- [x] package.json created with all metadata
- [x] README.md is comprehensive and up-to-date
- [x] LICENSE file exists (GPL-3.0)
- [x] .npmignore or files array in package.json configured
- [x] bin scripts are executable
- [x] Version number is correct (1.0.0 for initial release)

## Publishing Steps

### 1. Test Package Locally

```bash
# Navigate to project directory
cd C:\Users\abhij\nexus-axiom-final

# Test package installation locally
npm pack
# This creates nexus-axiom-1.0.0.tgz

# Test installation from tarball
npm install -g ./nexus-axiom-1.0.0.tgz

# Verify it works
nexus-axiom --help
```

### 2. Check Package Contents

```bash
# See what will be published
npm publish --dry-run

# Verify the file list is correct
```

### 3. Publish to NPM

```bash
# Publish the package
npm publish

# For scoped packages (if needed):
# npm publish --access public
```

### 4. Verify Publication

```bash
# Check on NPM registry
npm view nexus-axiom

# Install from NPM to test
npm install -g nexus-axiom

# Verify it works
nexus-axiom --version
```

## Post-Publishing Tasks

### 1. Add NPM Badge to README

Add this to the top of README.md:

```markdown
[![npm version](https://badge.fury.io/js/nexus-axiom.svg)](https://www.npmjs.com/package/nexus-axiom)
[![npm downloads](https://img.shields.io/npm/dm/nexus-axiom.svg)](https://www.npmjs.com/package/nexus-axiom)
```

### 2. Update GitHub Repository

```bash
git add .
git commit -m "chore: publish v1.0.0 to NPM"
git tag v1.0.0
git push origin main --tags
```

### 3. Create GitHub Release

1. Go to https://github.com/CoderAwesomeAbhi/nexus-axiom/releases
2. Click "Create a new release"
3. Tag: v1.0.0
4. Title: "Nexus Axiom v1.0.0 - Initial Release"
5. Description:
   ```markdown
   ## 🎉 Initial Release
   
   Nexus Axiom is now available on NPM!
   
   ### Installation
   ```bash
   npm install -g nexus-axiom
   ```
   
   ### Features
   - ✅ W^X memory blocking using eBPF LSM hooks
   - ✅ Automatic exploit process termination
   - ✅ Tested against 4 major CVEs
   - ✅ Prometheus metrics + web dashboard
   - ✅ Kubernetes DaemonSet ready
   
   ### Links
   - NPM: https://www.npmjs.com/package/nexus-axiom
   - Documentation: https://github.com/CoderAwesomeAbhi/nexus-axiom#readme
   ```

### 4. Announce on Social Media

**Twitter/X:**
```
🚀 Nexus Axiom v1.0.0 is now on NPM!

eBPF security that actually KILLS exploits before execution.

Install: npm install -g nexus-axiom

✅ Blocks W^X memory exploits
✅ Tested against PwnKit, Dirty Pipe, Sudo CVEs
✅ Zero false positives

GitHub: https://github.com/CoderAwesomeAbhi/nexus-axiom
NPM: https://www.npmjs.com/package/nexus-axiom

#CyberSecurity #eBPF #Linux #OpenSource
```

**LinkedIn:**
Use the posts from LINKEDIN_SHOWCASE.md

**Reddit:**
- r/netsec
- r/linux
- r/rust
- r/kubernetes
- r/cybersecurity

### 5. Submit to Directories

- **Awesome Lists:**
  - https://github.com/zoidbergwill/awesome-ebpf
  - https://github.com/sbilly/awesome-security
  - https://github.com/qazbnm456/awesome-cve-poc

- **Product Hunt:**
  - https://www.producthunt.com/posts/new

- **Hacker News:**
  - https://news.ycombinator.com/submit

## Updating the Package

### For Bug Fixes (Patch Version)

```bash
npm version patch  # 1.0.0 -> 1.0.1
npm publish
git push origin main --tags
```

### For New Features (Minor Version)

```bash
npm version minor  # 1.0.0 -> 1.1.0
npm publish
git push origin main --tags
```

### For Breaking Changes (Major Version)

```bash
npm version major  # 1.0.0 -> 2.0.0
npm publish
git push origin main --tags
```

## Troubleshooting

### "Package name already exists"

If the name is taken, you can:
1. Use a scoped package: `@yourusername/nexus-axiom`
2. Choose a different name
3. Contact NPM support if you believe you have rights to the name

### "You must verify your email"

```bash
npm profile get
# Check if email is verified
# If not, check your email for verification link
```

### "403 Forbidden"

```bash
# Re-login to NPM
npm logout
npm login
```

### "ENEEDAUTH"

```bash
# Check authentication
npm whoami

# If not logged in:
npm login
```

## Security Best Practices

1. **Enable 2FA on NPM Account**
   ```bash
   npm profile enable-2fa auth-and-writes
   ```

2. **Use NPM Tokens for CI/CD**
   - Generate token: https://www.npmjs.com/settings/YOUR_USERNAME/tokens
   - Add to GitHub Secrets for automated publishing

3. **Sign Commits**
   ```bash
   git config --global commit.gpgsign true
   ```

## Monitoring

### Track Downloads

- NPM Stats: https://npm-stat.com/charts.html?package=nexus-axiom
- NPM Trends: https://www.npmtrends.com/nexus-axiom

### Monitor Issues

- GitHub Issues: https://github.com/CoderAwesomeAbhi/nexus-axiom/issues
- NPM Support: https://www.npmjs.com/support

## Success Metrics

Track these after publishing:

- **Week 1 Goals:**
  - 100+ downloads
  - 50+ GitHub stars
  - 5+ issues/questions

- **Month 1 Goals:**
  - 1,000+ downloads
  - 200+ GitHub stars
  - 10+ contributors

- **Month 3 Goals:**
  - 5,000+ downloads
  - 500+ GitHub stars
  - Featured in security newsletters

## Next Steps After Publishing

1. ✅ Publish to NPM
2. ✅ Create GitHub release
3. ✅ Post on LinkedIn (use LINKEDIN_SHOWCASE.md)
4. ✅ Post on Twitter/X
5. ✅ Submit to Hacker News
6. ✅ Submit to Reddit
7. ✅ Add to Awesome lists
8. ✅ Write blog post
9. ✅ Create demo video
10. ✅ Reach out to security influencers

## Resources

- NPM Documentation: https://docs.npmjs.com/
- Package.json Reference: https://docs.npmjs.com/cli/v9/configuring-npm/package-json
- NPM Publishing Guide: https://docs.npmjs.com/packages-and-modules/contributing-packages-to-the-registry
- Semantic Versioning: https://semver.org/

---

**Ready to publish?**

```bash
cd C:\Users\abhij\nexus-axiom-final
npm login
npm publish --dry-run  # Test first
npm publish            # Actually publish
```

Good luck! 🚀
