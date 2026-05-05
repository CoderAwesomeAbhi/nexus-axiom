# Deployment Guide

## Quick Start

### Prerequisites

- Linux kernel 6.8 or later
- Root access
- 2GB RAM minimum
- BTF support enabled

### One-Line Install

```bash
curl -fsSL https://get.nexus-axiom.dev | sudo bash
```

This will:
1. Download and verify the latest release
2. Install systemd service
3. Configure default policies
4. Start the daemon in audit-only mode

## Installation Methods

### Method 1: Package Manager

#### Debian/Ubuntu

```bash
curl -fsSL https://apt.nexus-axiom.dev/gpg | sudo gpg --dearmor -o /usr/share/keyrings/nexus-axiom.gpg
echo "deb [signed-by=/usr/share/keyrings/nexus-axiom.gpg] https://apt.nexus-axiom.dev stable main" | sudo tee /etc/apt/sources.list.d/nexus-axiom.list
sudo apt update
sudo apt install nexus-axiom
```

#### RHEL/CentOS/Fedora

```bash
sudo dnf config-manager --add-repo https://rpm.nexus-axiom.dev/nexus-axiom.repo
sudo dnf install nexus-axiom
```

#### Arch Linux

```bash
yay -S nexus-axiom
```

### Method 2: Binary Release

```bash
wget https://github.com/nexus-org/nexus-axiom/releases/latest/download/nexus-axiom-x86_64-linux.tar.gz
tar xzf nexus-axiom-x86_64-linux.tar.gz
sudo ./install.sh
```

### Method 3: Build from Source

```bash
git clone https://github.com/nexus-org/nexus-axiom.git
cd nexus-axiom
cargo build --release
sudo cp target/release/nexus-axiom /usr/local/bin/
sudo cp systemd/nexus-axiom.service /etc/systemd/system/
sudo systemctl daemon-reload
```

## Configuration

### Basic Configuration

Create `/etc/nexus-axiom/config.yaml`:

```yaml
daemon:
  log_level: info
  worker_threads: 4

enforcement:
  audit_only: true  # Start in audit mode
  protected_paths:
    - /etc/passwd
    - /etc/shadow
    - /etc/sudoers
    - /boot
    - /usr/bin
    - /usr/sbin

observability:
  enable_prometheus: true
  metrics_port: 9090
  enable_structured_logging: true

security:
  verify_binary_signature: true
  enable_forensics: true
```

### Production Configuration

```yaml
daemon:
  log_level: warn
  worker_threads: 8
  pid_file: /var/run/nexus-axiom.pid

ebpf:
  max_tracked_files: 50000
  max_tracked_processes: 10000
  hook_file_write: true
  hook_mmap: true
  hook_socket: true
  hook_exec: true

enforcement:
  audit_only: false  # Enable enforcement
  enable_quarantine: true
  enable_kill: true
  protected_paths:
    - /etc
    - /boot
    - /usr/bin
    - /usr/sbin
    - /lib
    - /lib64
  whitelist_binaries:
    - /usr/bin/apt
    - /usr/bin/yum
    - /usr/bin/dpkg

ai:
  enable_local_ml: true
  enable_llm: false  # Disable for production
  confidence_threshold: 0.8

observability:
  enable_prometheus: true
  metrics_port: 9090
  log_output: /var/log/nexus-axiom/events.log

security:
  verify_binary_signature: true
  enable_forensics: true
  forensics_buffer_size: 10000000
  enable_pqc_signing: true
```

## Kubernetes Deployment

### Helm Chart

```bash
helm repo add nexus https://charts.nexus-axiom.dev
helm repo update
helm install nexus-axiom nexus/nexus-axiom \
  --namespace kube-system \
  --set enforcement.auditOnly=true \
  --set observability.prometheusEnabled=true
```

### DaemonSet YAML

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: nexus-axiom
  namespace: kube-system
spec:
  selector:
    matchLabels:
      app: nexus-axiom
  template:
    metadata:
      labels:
        app: nexus-axiom
    spec:
      hostPID: true
      hostNetwork: true
      containers:
      - name: nexus-axiom
        image: nexusaxiom/nexus-axiom:latest
        securityContext:
          privileged: true
          capabilities:
            add:
            - SYS_ADMIN
            - SYS_RESOURCE
            - NET_ADMIN
        volumeMounts:
        - name: sys
          mountPath: /sys
        - name: config
          mountPath: /etc/nexus-axiom
        - name: bpf
          mountPath: /sys/fs/bpf
        env:
        - name: NEXUS_AUDIT_ONLY
          value: "true"
        - name: NEXUS_LOG_LEVEL
          value: "info"
      volumes:
      - name: sys
        hostPath:
          path: /sys
      - name: config
        configMap:
          name: nexus-axiom-config
      - name: bpf
        hostPath:
          path: /sys/fs/bpf
```

## Docker Deployment

```bash
docker run -d \
  --name nexus-axiom \
  --privileged \
  --pid=host \
  --network=host \
  -v /sys:/sys:ro \
  -v /sys/fs/bpf:/sys/fs/bpf:rw \
  -v /etc/nexus-axiom:/etc/nexus-axiom:ro \
  -e NEXUS_AUDIT_ONLY=true \
  nexusaxiom/nexus-axiom:latest
```

## Systemd Service

### Enable and Start

```bash
sudo systemctl enable nexus-axiom
sudo systemctl start nexus-axiom
```

### Check Status

```bash
sudo systemctl status nexus-axiom
sudo journalctl -u nexus-axiom -f
```

### Service Configuration

Edit `/etc/systemd/system/nexus-axiom.service`:

```ini
[Unit]
Description=Nexus Axiom Security Framework
After=network.target
Requires=network.target

[Service]
Type=simple
ExecStart=/usr/bin/nexus-axiom start --config /etc/nexus-axiom/config.yaml
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5s
KillMode=process
LimitNOFILE=1048576
LimitNPROC=512

[Install]
WantedBy=multi-user.target
```

## Monitoring Setup

### Prometheus

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'nexus-axiom'
    static_configs:
      - targets: ['localhost:9090']
```

### Grafana

Import dashboard:

```bash
curl -o dashboard.json https://raw.githubusercontent.com/nexus-org/nexus-axiom/main/deploy/grafana-dashboard.json
```

Then import via Grafana UI.

### Alertmanager

Example alert rules:

```yaml
groups:
  - name: nexus-axiom
    rules:
      - alert: HighBlockRate
        expr: rate(nexus_events_blocked[5m]) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High block rate detected"
          
      - alert: DaemonDown
        expr: up{job="nexus-axiom"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Nexus Axiom daemon is down"
```

## Verification

### Check Installation

```bash
nexus-axiom --version
nexus-axiom test
```

### Verify eBPF Programs

```bash
sudo bpftool prog list | grep nexus
sudo bpftool map list | grep nexus
```

### Test Protection

```bash
# Should be blocked/logged
echo "test" | sudo tee /etc/passwd
```

### View Events

```bash
nexus-axiom events --limit 100
nexus-axiom monitor  # Interactive TUI
```

## Upgrading

### Package Manager

```bash
sudo apt update && sudo apt upgrade nexus-axiom
sudo systemctl restart nexus-axiom
```

### Manual

```bash
wget https://github.com/nexus-org/nexus-axiom/releases/latest/download/nexus-axiom-x86_64-linux.tar.gz
tar xzf nexus-axiom-x86_64-linux.tar.gz
sudo systemctl stop nexus-axiom
sudo cp nexus-axiom /usr/local/bin/
sudo systemctl start nexus-axiom
```

## Uninstallation

```bash
sudo systemctl stop nexus-axiom
sudo systemctl disable nexus-axiom
sudo apt remove nexus-axiom  # or dnf remove
sudo rm -rf /etc/nexus-axiom
sudo rm -rf /var/log/nexus-axiom
```

## Troubleshooting

### Daemon Won't Start

```bash
# Check kernel version
uname -r  # Must be 6.8+

# Check BTF support
ls /sys/kernel/btf/vmlinux

# Check permissions
id  # Must be root

# View detailed logs
sudo journalctl -u nexus-axiom -xe
```

### High CPU Usage

```bash
# Check event rate
nexus-axiom events --limit 1000 | wc -l

# Reduce map sizes in config
# Disable LLM analysis
# Use audit-only mode
```

### eBPF Load Failures

```bash
# Check BPF filesystem
mount | grep bpf

# Mount if missing
sudo mount -t bpf bpf /sys/fs/bpf

# Check verifier logs
sudo dmesg | grep -i bpf
```

## Best Practices

1. **Start in Audit Mode**: Always deploy with `audit_only: true` first
2. **Monitor for 1 Week**: Observe false positives before enforcement
3. **Whitelist System Tools**: Add package managers to whitelist
4. **Enable Metrics**: Always run Prometheus for visibility
5. **Regular Backups**: Backup `/etc/nexus-axiom` configuration
6. **Test Upgrades**: Test in staging before production
7. **Review Logs Daily**: Check for anomalies
8. **Tune Thresholds**: Adjust confidence thresholds per environment

## Support

- Documentation: https://docs.nexus-axiom.dev
- GitHub Issues: https://github.com/nexus-org/nexus-axiom/issues
- Discord: https://discord.gg/nexus-axiom
- Email: support@nexus-axiom.dev
