# Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        USER APPLICATIONS                            │
│                                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │  Node.js │  │   Java   │  │  Exploit │  │  Normal  │          │
│  │   (JIT)  │  │   (JIT)  │  │  Process │  │   App    │          │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘          │
│       │             │              │             │                 │
│       └─────────────┴──────────────┴─────────────┘                 │
│                          │                                          │
│                          ▼ syscall (mmap/mprotect/open/...)        │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      LINUX KERNEL                                   │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                  LSM HOOK LAYER  (eBPF)                     │   │
│   │                                                             │   │
│   │  lsm/mmap_file ──────► W^X check ──► -EPERM (block)        │   │
│   │  lsm/file_mprotect ──► W^X check ──► -EPERM (block)        │   │
│   │  lsm/file_open ──────► path check ──► event emitted        │   │
│   │  lsm/bprm_check ─────► exec check ──► event emitted        │   │
│   │  lsm/ptrace_access ──► ptrace block ► -EPERM (block)       │   │
│   │                                                             │   │
│   │  Ring Buffer (1 MB) ◄──────────────── all events           │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │               XDP HOOK  (eBPF, per NIC)                     │   │
│   │                                                             │   │
│   │  Packet in ──► IP blocklist check ──► XDP_DROP             │   │
│   │            ──► Port blocklist check ─► XDP_DROP            │   │
│   │            ──► Rate limit (1000 PPS) ─► XDP_DROP           │   │
│   │            ──► XDP_PASS (allow)                            │   │
│   └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────┬──────────────────────────────────────┘
                               │  ring buffer poll (100ms)
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   NEXUS AXIOM DAEMON  (Rust)                        │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ EbpfEngine   │  │  NetEngine   │  │     SeccompEngine        │  │
│  │              │  │              │  │  (hardens daemon itself)  │  │
│  │ • poll ring  │  │ • XDP attach │  └──────────────────────────┘  │
│  │ • SIGKILL    │  │ • IP/port    │                                 │
│  │ • AI analyst │  │   blocklist  │  ┌──────────────────────────┐  │
│  └──────────────┘  └──────────────┘  │      FsProtection        │  │
│                                      │  • critical path set     │  │
│  ┌──────────────┐  ┌──────────────┐  │  • inode tracking        │  │
│  │MetricsServer │  │  Dashboard   │  └──────────────────────────┘  │
│  │  :9090       │  │  :8080       │                                 │
│  │  8 metrics   │  │  auto-refresh│  ┌──────────────────────────┐  │
│  └──────────────┘  └──────────────┘  │      JsonLogger          │  │
│                                      │  Standard/Splunk/ELK/DD  │  │
│                                      └──────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                               │
                    ┌──────────┴──────────┐
                    ▼                     ▼
           ┌──────────────┐     ┌──────────────────┐
           │  Prometheus  │     │  Splunk / ELK /  │
           │  + Grafana   │     │  Datadog / SIEM  │
           └──────────────┘     └──────────────────┘
```

## Key Components

### 1. LSM Hooks (Kernel Space)
- **Location**: Linux kernel security decision path
- **Timing**: BEFORE syscall completes
- **Action**: Return -EPERM to block, or 0 to allow
- **Events**: Sent to ring buffer for userspace processing

### 2. XDP Hooks (Kernel Space)
- **Location**: Network driver, before sk_buff allocation
- **Timing**: First point packet enters kernel
- **Action**: XDP_DROP or XDP_PASS
- **Performance**: Line-rate filtering (10Gbps+)

### 3. Ring Buffer
- **Size**: 1 MB (configurable)
- **Type**: BPF_MAP_TYPE_RINGBUF
- **Delivery**: Zero-copy, ordered
- **Throughput**: 1M+ events/sec

### 4. Nexus Axiom Daemon
- **Language**: Rust (memory-safe)
- **Privileges**: Runs as root (required for eBPF)
- **Isolation**: Seccomp restricts syscalls
- **Threading**: Async I/O with tokio

### 5. Monitoring Stack
- **Metrics**: Prometheus format
- **Dashboard**: Built-in web UI
- **Logs**: JSON (multiple formats)
- **Visualization**: Grafana dashboards

## Data Flow

### Exploit Attempt
```
1. Exploit calls mmap(PROT_WRITE|PROT_EXEC)
2. Kernel invokes LSM hook (lsm/mmap_file)
3. eBPF program checks: (prot & WRITE) && (prot & EXEC)
4. eBPF returns -EPERM
5. Kernel fails syscall with EPERM
6. eBPF emits event to ring buffer
7. Daemon reads event
8. Daemon sends SIGKILL to process
9. Daemon increments metrics
10. Daemon logs JSON event
11. AI analyst analyzes (async)
```

### Normal Application
```
1. App calls mmap(PROT_READ|PROT_WRITE)
2. Kernel invokes LSM hook
3. eBPF checks: no EXEC bit
4. eBPF returns 0 (allow)
5. Kernel completes syscall normally
6. No event emitted (not suspicious)
7. App continues normally
```

## Design Decisions

### Why LSM over Kprobes?
- **LSM**: In security decision path, can block
- **Kprobes**: After decision, can only observe
- **Trade-off**: Higher overhead, but actual prevention

### Why Ring Buffer over Perf Buffer?
- **Ring Buffer**: Zero-copy, ordered, simpler API
- **Perf Buffer**: Per-CPU, requires merging
- **Trade-off**: Single producer, but easier to use

### Why Rust over C?
- **Rust**: Memory-safe, no GC pauses
- **C**: Lower-level control
- **Trade-off**: Safer, but larger binary

### Why SIGKILL from Userspace?
- **eBPF**: Can't send signals from kernel
- **Userspace**: Full process control
- **Trade-off**: Small race window, but more flexible

## Security Boundaries

### Trusted
- Linux kernel
- eBPF verifier
- Nexus Axiom daemon (runs as root)

### Untrusted
- All user applications
- Network packets
- File system events

### Threat Model
- **Protects against**: W^X exploits, shellcode injection
- **Does not protect**: Kernel exploits, pure ROP, side-channels
- **Assumes**: Kernel is trusted, eBPF verifier works correctly

## Performance Characteristics

### Latency
- **LSM hook**: ~0.7μs per mmap/mprotect
- **XDP**: ~50ns per packet
- **Ring buffer**: ~100ns per event

### Throughput
- **Events**: 1M+ events/sec
- **Network**: 10Gbps+ line-rate
- **Metrics**: 15s scrape interval

### Resource Usage
- **Memory**: ~18 MB RSS
- **CPU**: <1% idle, <5% under load
- **Disk**: Minimal (logs only)

## Failure Modes

### eBPF Load Failure
- **Cause**: Kernel incompatibility, BTF missing
- **Effect**: Daemon fails to start
- **Recovery**: Check pre-flight, fix kernel config

### Ring Buffer Full
- **Cause**: Event rate > processing rate
- **Effect**: Events dropped
- **Recovery**: Increase buffer size, rate limit

### Daemon Crash
- **Cause**: Bug, OOM, SIGKILL
- **Effect**: No new events processed, eBPF still loaded
- **Recovery**: Systemd restarts daemon

### False Positive
- **Cause**: JIT compiler uses W^X legitimately
- **Effect**: Process killed
- **Recovery**: Add to allowlist

## Monitoring Points

### Health Checks
- `/health` endpoint (TODO)
- Metrics endpoint responding
- eBPF programs loaded
- Ring buffer not full

### Key Metrics
- `blocked_total` - Exploits stopped
- `events_total` - Total activity
- `network_drops` - Packets filtered
- `uptime_seconds` - Daemon stability

### Alerts
- Exploit blocked (critical)
- High event rate (warning)
- Daemon down (critical)
- Ring buffer full (warning)

## See Also

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - Detailed technical deep dive
- [WHY_LSM.md](WHY_LSM.md) - LSM vs kprobes explanation
- [BENCHMARK_RESULTS.md](BENCHMARK_RESULTS.md) - Performance analysis
