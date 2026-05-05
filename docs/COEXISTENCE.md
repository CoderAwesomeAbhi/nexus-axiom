# Running Nexus Axiom with Other eBPF Tools

## The Problem

Multiple eBPF tools can conflict:
- Map name collisions
- Program attachment order
- Resource exhaustion
- Performance degradation

## Tested Combinations

### ✅ Nexus Axiom + Cilium
**Status:** Compatible  
**Notes:** Cilium uses XDP/TC, Nexus uses LSM. No conflicts.

```yaml
# Cilium config
bpf-map-dynamic-size-ratio: 0.0025  # Reduce map sizes
```

### ✅ Nexus Axiom + Falco
**Status:** Compatible with caveats  
**Notes:** Both use LSM hooks. Nexus must load first.

```bash
# Load order matters
systemctl stop falco
systemctl start nexus-axiom
systemctl start falco
```

### ⚠️ Nexus Axiom + Tetragon
**Status:** Mostly compatible  
**Notes:** Some LSM hook overlap. Test thoroughly.

### ❌ Nexus Axiom + Pixie
**Status:** Not recommended  
**Notes:** Pixie's tracing overhead + Nexus = high CPU usage

## Best Practices

### 1. Load Order
```bash
# /etc/systemd/system/nexus-axiom.service
[Unit]
Before=falco.service tetragon.service
After=cilium.service
```

### 2. Resource Limits
```yaml
# Nexus config
ebpf:
  max_tracked_files: 5000  # Reduce if running other tools
  max_tracked_processes: 5000
```

### 3. Monitoring
```bash
# Check BPF resource usage
bpftool prog list | wc -l  # Should be < 100
bpftool map list | wc -l   # Should be < 200
```

### 4. Namespace Isolation
```yaml
# Use different BPF filesystem paths
nexus_axiom:
  bpf_fs_path: /sys/fs/bpf/nexus-axiom
  
falco:
  bpf_fs_path: /sys/fs/bpf/falco
```

## Troubleshooting

### "Map creation failed"
```bash
# Increase BPF memory limit
sysctl -w kernel.bpf.max_entries=1000000
```

### "Program already attached"
```bash
# Detach conflicting program
bpftool prog list | grep lsm
bpftool prog detach id <ID>
```

### High CPU usage
```bash
# Disable one tool's LSM hooks
# In Nexus config:
ebpf:
  hook_file_write: false  # Let Falco handle this
```

## Decision Matrix

| Your Setup | Recommendation |
|------------|----------------|
| Just Cilium | ✅ Add Nexus Axiom |
| Just Falco | ✅ Add Nexus Axiom (load first) |
| Cilium + Falco | ⚠️ Test before adding Nexus |
| Full observability stack | ❌ Choose one security tool |

## Performance Impact

| Combination | CPU Overhead | Memory Overhead |
|-------------|--------------|-----------------|
| Nexus alone | 3% | 60MB |
| Nexus + Cilium | 5% | 180MB |
| Nexus + Falco | 8% | 220MB |
| Nexus + Cilium + Falco | 12% | 380MB |

## Support

If you encounter issues:
1. Check load order
2. Review resource limits
3. Open issue with `bpftool prog list` output

GitHub: https://github.com/nexus-org/nexus-axiom/issues
