#!/bin/bash
# Nexus Axiom Performance Benchmarks

set -e

echo "🔬 NEXUS AXIOM PERFORMANCE BENCHMARKS"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if Nexus Axiom is running
if ! pgrep -x "nexus-axiom" > /dev/null; then
    echo "⚠️  Nexus Axiom is not running. Starting it..."
    if [ -f ../target/release/nexus-axiom ]; then
        sudo ../target/release/nexus-axiom start > /dev/null 2>&1 &
        sleep 3
    else
        echo "❌ Nexus Axiom binary not found. Run: cargo build --release"
        exit 1
    fi
fi

echo "📊 Benchmark 1: Memory Usage"
echo "─────────────────────────────"
mem_kb=$(ps aux | grep nexus-axiom | grep -v grep | awk '{print $6}' | head -1)
if [ -n "$mem_kb" ]; then
    mem_mb=$((mem_kb / 1024))
    echo "RSS Memory: ${mem_mb} MB"
    echo -e "${GREEN}✓ Memory usage: ${mem_mb} MB${NC}"
else
    echo "⚠️  Could not measure memory (process not found)"
fi
echo ""

echo "📊 Benchmark 2: Event Processing Latency"
echo "─────────────────────────────────────────"

# Create latency test
cat > /tmp/bench_latency.c << 'EOF'
#include <stdio.h>
#include <time.h>
#include <sys/mman.h>

int main() {
    struct timespec start, end;
    long total_ns = 0;
    int iterations = 10000;
    
    for (int i = 0; i < iterations; i++) {
        clock_gettime(CLOCK_MONOTONIC, &start);
        
        void *mem = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                         MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
        if (mem != MAP_FAILED) {
            munmap(mem, 4096);
        }
        
        clock_gettime(CLOCK_MONOTONIC, &end);
        total_ns += (end.tv_sec - start.tv_sec) * 1000000000 +
                    (end.tv_nsec - start.tv_nsec);
    }
    
    printf("Average latency: %.2f microseconds\n", 
           (double)total_ns / iterations / 1000.0);
    return 0;
}
EOF

gcc /tmp/bench_latency.c -o /tmp/bench_latency 2>/dev/null
if [ -f /tmp/bench_latency ]; then
    /tmp/bench_latency
    rm /tmp/bench_latency /tmp/bench_latency.c
else
    echo "⚠️  Could not compile benchmark"
fi
echo ""

echo "📊 Benchmark 3: Event Throughput"
echo "───────────────────────────"
echo "Testing event processing throughput..."

# Get current event count
before=$(curl -s http://localhost:9090/metrics 2>/dev/null | grep "nexus_axiom_events_total" | awk '{print $2}')

if [ -n "$before" ]; then
    # Generate load
    for i in {1..1000}; do
        /bin/true &
    done
    wait
    
    sleep 1
    
    # Get new event count
    after=$(curl -s http://localhost:9090/metrics 2>/dev/null | grep "nexus_axiom_events_total" | awk '{print $2}')
    
    events=$((after - before))
    echo "Processed $events events in 1 second"
    echo -e "${GREEN}✓ Throughput: ${events} events/second${NC}"
else
    echo "⚠️  Metrics endpoint not available"
    echo "   Make sure Nexus Axiom is running with metrics enabled"
fi
echo ""

echo "======================================"
echo "📈 SUMMARY"
echo "======================================"
echo ""
echo "These are MEASURED results from actual system calls."
echo "Performance varies based on:"
echo "  • Kernel version"
echo "  • System load"
echo "  • CPU architecture"
echo "  • Number of active eBPF hooks"
echo ""
echo "Benchmarked on: $(uname -r), $(nproc) cores, $(free -h | grep Mem | awk '{print $2}') RAM"
