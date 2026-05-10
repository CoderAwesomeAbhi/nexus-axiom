#!/bin/bash
# Nexus Axiom Performance Benchmark Suite
set -euo pipefail

CYAN='\033[0;36m'; GREEN='\033[0;32m'; NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  📊 Nexus Axiom Performance Benchmark${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# ── Syscall Overhead ──────────────────────────────────────────────
echo -e "${CYAN}[1/4] Syscall Overhead${NC}"

# Measure mmap baseline without Nexus Axiom
ITERATIONS=10000

cat > /tmp/bench_mmap.c <<'EOF'
#include <sys/mman.h>
#include <stdlib.h>
#include <time.h>
#include <stdio.h>
int main() {
    struct timespec start, end;
    int N = 10000;
    clock_gettime(CLOCK_MONOTONIC, &start);
    for (int i = 0; i < N; i++) {
        void *p = mmap(NULL, 4096, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
        if (p != MAP_FAILED) munmap(p, 4096);
    }
    clock_gettime(CLOCK_MONOTONIC, &end);
    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
    printf("%.2f ms per %d mmap+munmap cycles (%.2f us/call)\n", elapsed * 1000, N, elapsed / N * 1e6);
    return 0;
}
EOF

if gcc -O2 -o /tmp/bench_mmap /tmp/bench_mmap.c 2>/dev/null; then
    echo "  Baseline (no LSM):"
    echo -n "    "
    /tmp/bench_mmap
    rm -f /tmp/bench_mmap /tmp/bench_mmap.c
else
    echo "  Skipped (gcc not available)"
fi

echo ""

# ── Event Throughput ──────────────────────────────────────────────
echo -e "${CYAN}[2/4] Event Throughput${NC}"

if command -v nexus-axiom &>/dev/null && pgrep nexus-axiom &>/dev/null; then
    # Check metrics endpoint for current throughput
    METRICS=$(curl -s http://localhost:9090/metrics 2>/dev/null || echo "")
    if [ -n "$METRICS" ]; then
        TOTAL=$(echo "$METRICS" | grep "nexus_axiom_events_total" | awk '{print $2}')
        UPTIME=$(echo "$METRICS" | grep "nexus_axiom_uptime_seconds" | awk '{print $2}')
        if [ -n "$TOTAL" ] && [ -n "$UPTIME" ] && [ "$UPTIME" != "0" ]; then
            RATE=$(echo "scale=2; $TOTAL / $UPTIME" | bc 2>/dev/null || echo "N/A")
            echo "  Events/sec: $RATE"
            echo "  Total events: $TOTAL"
            echo "  Uptime: ${UPTIME}s"
        fi
    fi
else
    echo "  Nexus Axiom not running — skipped"
fi

echo ""

# ── Memory Usage ──────────────────────────────────────────────────
echo -e "${CYAN}[3/4] Memory Usage${NC}"

PID=$(pgrep nexus-axiom 2>/dev/null || echo "")
if [ -n "$PID" ]; then
    RSS=$(awk '/VmRSS/ {print $2}' /proc/$PID/status 2>/dev/null || echo "N/A")
    VSZ=$(awk '/VmSize/ {print $2}' /proc/$PID/status 2>/dev/null || echo "N/A")
    echo "  RSS: ${RSS} kB"
    echo "  VSZ: ${VSZ} kB"
else
    echo "  Nexus Axiom not running — skipped"
fi

echo ""

# ── XDP Performance ──────────────────────────────────────────────
echo -e "${CYAN}[4/4] XDP Network Filter${NC}"

if command -v bpftool &>/dev/null; then
    XDP_PROGS=$(bpftool prog list 2>/dev/null | grep -c "xdp" || echo "0")
    echo "  XDP programs loaded: $XDP_PROGS"
    if [ "$XDP_PROGS" -gt 0 ]; then
        echo "  XDP stats:"
        bpftool prog list 2>/dev/null | grep -A2 "xdp" | head -6 || true
    fi
else
    echo "  bpftool not available — skipped"
fi

echo ""
echo -e "${GREEN}✅ Benchmark complete${NC}"
