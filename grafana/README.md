# Grafana Dashboard Setup

## Quick Start with Docker

```bash
# 1. Start Prometheus
docker run -d \
  --name prometheus \
  -p 9091:9090 \
  -v $(pwd)/grafana/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus

# 2. Start Grafana
docker run -d \
  --name grafana \
  -p 3000:3000 \
  grafana/grafana

# 3. Configure Grafana
# - Open http://localhost:3000 (admin/admin)
# - Add Prometheus data source: http://prometheus:9090
# - Import dashboard from grafana/dashboard.json
```

## Manual Setup

### 1. Install Prometheus

```bash
# Ubuntu/Debian
sudo apt-get install prometheus

# Configure scrape target
sudo nano /etc/prometheus/prometheus.yml
# Add:
#   - job_name: 'nexus-axiom'
#     static_configs:
#       - targets: ['localhost:9090']

sudo systemctl restart prometheus
```

### 2. Install Grafana

```bash
# Ubuntu/Debian
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install grafana

sudo systemctl start grafana-server
sudo systemctl enable grafana-server
```

### 3. Import Dashboard

1. Open Grafana: http://localhost:3000
2. Login (default: admin/admin)
3. Add Prometheus data source:
   - Configuration → Data Sources → Add data source
   - Select Prometheus
   - URL: http://localhost:9091
   - Save & Test
4. Import dashboard:
   - Create → Import
   - Upload `grafana/dashboard.json`
   - Select Prometheus data source
   - Import

## Available Metrics

- `nexus_axiom_events_total` - Total security events
- `nexus_axiom_blocked_total` - Total exploits blocked
- `nexus_axiom_mmap_events` - W^X mmap violations
- `nexus_axiom_mprotect_events` - W^X mprotect violations
- `nexus_axiom_exec_events` - Execution control events
- `nexus_axiom_file_events` - File access events
- `nexus_axiom_network_drops` - Network packets dropped
- `nexus_axiom_uptime_seconds` - Daemon uptime

## Alerting

Example Prometheus alert rules:

```yaml
groups:
  - name: nexus_axiom
    rules:
      - alert: ExploitDetected
        expr: increase(nexus_axiom_blocked_total[5m]) > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Exploit blocked by Nexus Axiom"
          description: "{{ $value }} exploits blocked in the last 5 minutes"
      
      - alert: HighEventRate
        expr: rate(nexus_axiom_events_total[5m]) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High security event rate"
          description: "{{ $value }} events/sec for 5 minutes"
```

## Troubleshooting

**Grafana can't connect to Prometheus**
- Check Prometheus is running: `curl http://localhost:9091/metrics`
- Check firewall rules
- Use correct URL in Grafana data source

**No data in dashboard**
- Verify Nexus Axiom is running: `systemctl status nexus-axiom`
- Check metrics endpoint: `curl http://localhost:9090/metrics`
- Verify Prometheus is scraping: http://localhost:9091/targets
