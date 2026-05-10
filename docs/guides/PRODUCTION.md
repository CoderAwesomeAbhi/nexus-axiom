# Production Deployment Checklist

## Pre-Deployment

### System Requirements
- [ ] Linux kernel 5.8 or higher (`uname -r`)
- [ ] BPF LSM enabled in kernel boot parameters
  ```bash
  # Check current LSM
  cat /sys/kernel/security/lsm
  # Should include 'bpf'
  
  # If not, add to /etc/default/grub:
  GRUB_CMDLINE_LINUX="lsm=bpf,..."
  sudo update-grub && sudo reboot
  ```
- [ ] Root access available
- [ ] x86_64 architecture (ARM untested)

### Capacity Planning
- [ ] Memory: Minimum 100MB free (Nexus Axiom uses ~18MB RSS)
- [ ] CPU: <1% overhead at idle, ~2-5% under load
- [ ] Disk: 50MB for binary + logs
- [ ] Network: Ports 8080 (dashboard) and 9090 (metrics) available

### Testing Environment
- [ ] Test in staging/dev environment first
- [ ] Run proof script: `sudo bash proof.sh`
- [ ] Verify exploit blocking works
- [ ] Check logs: `sudo journalctl -u nexus-axiom -f`

## Deployment

### Installation
- [ ] Run installer: `curl -sSL https://raw.githubusercontent.com/CoderAwesomeAbhi/nexus-axiom/main/install.sh | sudo bash`
- [ ] Verify service: `sudo systemctl status nexus-axiom`
- [ ] Check dashboard: `curl http://localhost:8080`
- [ ] Check metrics: `curl http://localhost:9090/metrics`

### Configuration
- [ ] Review `/etc/nexus-axiom/config.toml`
- [ ] Set mode: `enforce` (kills) or `audit` (logs only)
  ```toml
  [security]
  mode = "audit"  # Start with audit mode
  kill_on_violation = false
  ```
- [ ] Configure blocked IPs/ports if needed
  ```toml
  [network]
  blocked_ips = ["192.168.1.100"]
  blocked_ports = [22, 23]  # SSH, Telnet
  ```
- [ ] Set log level: `info` (default) or `debug`

### Gradual Rollout
- [ ] **Week 1:** Deploy in audit mode (logs only, no kills)
- [ ] Monitor logs for false positives
- [ ] Identify legitimate programs triggering blocks
- [ ] **Week 2:** Add allowlist for false positives (not yet implemented - use audit mode)
- [ ] **Week 3:** Switch to enforce mode on 10% of fleet
- [ ] **Week 4:** Roll out to 100% if no issues

## Monitoring

### Health Checks
- [ ] Service status: `systemctl is-active nexus-axiom`
- [ ] Process running: `ps aux | grep nexus-axiom`
- [ ] Memory usage: `ps aux | grep nexus-axiom | awk '{print $6/1024 " MB"}'`
- [ ] CPU usage: `top -p $(pgrep nexus-axiom)`

### Metrics (Prometheus)
- [ ] Scrape endpoint: `http://localhost:9090/metrics`
- [ ] Key metrics:
  - `nexus_axiom_events_total` - Total events processed
  - `nexus_axiom_blocked_events` - Exploits blocked
  - `nexus_axiom_mmap_events` - mmap() calls monitored
  - `nexus_axiom_mprotect_events` - mprotect() calls monitored

### Logs
- [ ] View live: `sudo journalctl -u nexus-axiom -f`
- [ ] Search blocks: `sudo journalctl -u nexus-axiom | grep BLOCKED`
- [ ] Export JSON: Logs written to `/var/log/nexus-axiom/events.json`

### Alerts
- [ ] Set up alerts for:
  - Service down: `systemctl is-active nexus-axiom != active`
  - High block rate: `nexus_axiom_blocked_events > threshold`
  - Memory spike: RSS > 100MB
  - Rate limit hits: Check logs for "Rate limit exceeded"

## Tuning

### Performance
- [ ] Adjust rate limit in code if needed (default: 10,000 events/sec)
- [ ] Monitor overhead: `perf top` during load
- [ ] If overhead too high: Consider audit mode or selective deployment

### False Positives
- [ ] Identify: Check logs for legitimate programs being blocked
- [ ] Common culprits:
  - Node.js (V8 JIT)
  - Java (JVM JIT)
  - Python (PyPy JIT)
  - Browsers (JavaScript JIT)
- [ ] Workaround: Run in audit mode or exclude these hosts
- [ ] Future: Allowlist feature (not yet implemented)

### False Negatives
- [ ] Test with known exploits: `cd /opt/nexus-axiom/cve_tests && make test`
- [ ] If exploit not blocked: Check if it uses W^X memory
- [ ] Remember: Only blocks W^X exploits (see LIMITATIONS.md)

## Rollback

### Emergency Rollback
```bash
# Stop service
sudo systemctl stop nexus-axiom

# Disable autostart
sudo systemctl disable nexus-axiom

# Verify stopped
sudo systemctl status nexus-axiom
```

### Uninstall
```bash
# Stop and disable
sudo systemctl stop nexus-axiom
sudo systemctl disable nexus-axiom

# Remove files
sudo rm -rf /opt/nexus-axiom
sudo rm /etc/systemd/system/nexus-axiom.service
sudo rm -rf /etc/nexus-axiom
sudo rm -rf /var/log/nexus-axiom

# Reload systemd
sudo systemctl daemon-reload
```

## Troubleshooting

### Service Won't Start
- [ ] Check LSM: `cat /sys/kernel/security/lsm` (need 'bpf')
- [ ] Check kernel: `uname -r` (need 5.8+)
- [ ] Check logs: `sudo journalctl -u nexus-axiom -n 50`
- [ ] Check permissions: Service must run as root

### High CPU Usage
- [ ] Check event rate: `curl http://localhost:9090/metrics | grep events_total`
- [ ] If >10K events/sec: Rate limiting kicking in
- [ ] Solution: Increase rate limit or use audit mode

### Dashboard Not Loading
- [ ] Check port: `sudo netstat -tlnp | grep 8080`
- [ ] Check firewall: `sudo ufw status`
- [ ] Try: `curl http://localhost:8080`

### Metrics Not Updating
- [ ] Check Prometheus endpoint: `curl http://localhost:9090/metrics`
- [ ] Verify service running: `systemctl is-active nexus-axiom`
- [ ] Check logs for errors

## Security Considerations

### Defense in Depth
- [ ] Nexus Axiom is ONE layer, not sole security
- [ ] Use with: SELinux/AppArmor, firewalls, IDS, etc.
- [ ] Regular patching still required

### Incident Response
- [ ] If exploit blocked:
  1. Check logs: `sudo journalctl -u nexus-axiom | grep BLOCKED`
  2. Identify process: PID, command, user
  3. Investigate: How did malicious code get there?
  4. Contain: Isolate affected system
  5. Remediate: Remove malicious code, patch vulnerability

### Updates
- [ ] Check for updates: `cd /opt/nexus-axiom && git pull`
- [ ] Rebuild: `cargo build --release`
- [ ] Restart: `sudo systemctl restart nexus-axiom`
- [ ] Verify: `sudo systemctl status nexus-axiom`

## Support

### Getting Help
- [ ] Documentation: https://github.com/CoderAwesomeAbhi/nexus-axiom
- [ ] Issues: https://github.com/CoderAwesomeAbhi/nexus-axiom/issues
- [ ] Verification: Run `sudo bash proof.sh`

### Reporting Bugs
- [ ] Include: Kernel version, OS, logs, config
- [ ] Reproduce: Provide steps to reproduce
- [ ] Logs: Attach relevant logs (redact sensitive info)

### Security Issues
- [ ] If you find a bypass: Open GitHub issue
- [ ] Include: Exploit code, kernel version, logs
- [ ] Be responsible: Don't publish 0-days publicly

---

**Remember:** Start with audit mode, monitor for false positives, roll out gradually.
