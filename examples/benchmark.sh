#!/bin/bash

set -e

echo "🔬 NEXUS AXIOM PERFORMANCE BENCHMARK"
echo "====================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run as root (sudo ./benchmark.sh)"
    exit 1
fi

# Build benchmark program
cat > /tmp/bench_mmap.c << 'EOF'
#include <stdio.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>

#define ITERATIONS 1000000

int main() {
    struct timespec start, end;
    long total_ns = 0;
    
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    for (int i = 0; i < ITERATIONS; i++) {
        void *addr = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                          MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (addr != MAP_FAILED) {
            munmap(addr, 4096);
        }
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    total_ns = (end.tv_sec - start.tv_sec) * 1000000000L + 
               (end.tv_nsec - start.tv_nsec);
    
    printf("%ld\n", total_ns / ITERATIONS);
    
    return 0;
}
EOF

gcc -O2 -o /tmp/bench_mmap /tmp/bench_mmap.c

echo "📊 Test 1: mmap() Latency"
echo "-------------------------"
echo ""

# Baseline (without Nexus Axiom)
echo "Running baseline (1M iterations)..."
baseline=$(/tmp/bench_mmap)
echo "Baseline: ${baseline}ns per mmap()"

# With Nexus Axiom
echo ""
echo "Starting Nexus Axiom..."
./target/release/nexus-axiom start --audit &
NEXUS_PID=$!
sleep 2

echo "Running with Nexus Axiom (1M iterations)..."
with_nexus=$(/tmp/bench_mmap)
echo "With Nexus Axiom: ${with_nexus}ns per mmap()"

# Calculate overhead
overhead=$((with_nexus - baseline))
overhead_pct=$((overhead * 100 / baseline))

echo ""
echo "Overhead: +${overhead}ns (+${overhead_pct}%)"

# Stop Nexus Axiom
kill $NEXUS_PID 2>/dev/null || true
sleep 1

echo ""
echo "📊 Test 2: CPU Usage"
echo "--------------------"
echo ""

# Start Nexus Axiom
./target/release/nexus-axiom start --audit &
NEXUS_PID=$!
sleep 2

# Get PID and measure CPU
NEXUS_ACTUAL_PID=$(pgrep -f "nexus-axiom start")
echo "Measuring CPU usage for 10 seconds..."

# Run some load
for i in {1..10}; do
    /tmp/bench_mmap > /dev/null &
done

sleep 10

# Get CPU usage
cpu_usage=$(ps -p $NEXUS_ACTUAL_PID -o %cpu= | awk '{print $1}')
echo "CPU Usage: ${cpu_usage}%"

# Stop everything
killall bench_mmap 2>/dev/null || true
kill $NEXUS_PID 2>/dev/null || true

echo ""
echo "📊 Test 3: Memory Usage"
echo "-----------------------"
echo ""

./target/release/nexus-axiom start --audit &
NEXUS_PID=$!
sleep 2

NEXUS_ACTUAL_PID=$(pgrep -f "nexus-axiom start")
mem_usage=$(ps -p $NEXUS_ACTUAL_PID -o rss= | awk '{print $1/1024}')
echo "Memory Usage: ${mem_usage}MB"

kill $NEXUS_PID 2>/dev/null || true

echo ""
echo "📊 Summary"
echo "=========="
echo ""
echo "| Metric          | Baseline | With Nexus Axiom | Overhead |"
echo "|-----------------|----------|------------------|----------|"
echo "| mmap latency    | ${baseline}ns | ${with_nexus}ns | +${overhead_pct}% |"
echo "| CPU usage       | 0%       | ${cpu_usage}% | ${cpu_usage}% |"
echo "| Memory          | 0MB      | ${mem_usage}MB | ${mem_usage}MB |"
echo ""
echo "✅ Benchmark complete!"

# Cleanup
rm -f /tmp/bench_mmap /tmp/bench_mmap.c
