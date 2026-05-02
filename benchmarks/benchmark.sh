#!/bin/bash
# NEXUS AXIOM PERFORMANCE BENCHMARK
# Measures overhead, latency, and throughput

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🚀 NEXUS AXIOM PERFORMANCE BENCHMARK"
echo "===================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Must run as root${NC}"
    exit 1
fi

# Check if Nexus Axiom is built
if [ ! -f "target/release/nexus-axiom" ]; then
    echo -e "${RED}Error: Build first with 'cargo build --release'${NC}"
    exit 1
fi

# Compile benchmark programs
echo "📦 Compiling benchmark programs..."
gcc -O2 -o bench_mmap bench_mmap.c
gcc -O2 -o bench_baseline bench_baseline.c
echo ""

# Benchmark 1: Baseline (no protection)
echo "📊 Benchmark 1: BASELINE (No Protection)"
echo "----------------------------------------"
./bench_baseline
BASELINE_OPS=$(./bench_baseline | grep "Operations/sec" | awk '{print $2}')
echo ""

# Benchmark 2: With Nexus Axiom
echo "📊 Benchmark 2: WITH NEXUS AXIOM"
echo "----------------------------------------"
./target/release/nexus-axiom start &
AXIOM_PID=$!
sleep 2

./bench_mmap
AXIOM_OPS=$(./bench_mmap | grep "Operations/sec" | awk '{print $2}')

kill $AXIOM_PID 2>/dev/null || true
echo ""

# Calculate overhead
OVERHEAD=$(echo "scale=2; (($BASELINE_OPS - $AXIOM_OPS) / $BASELINE_OPS) * 100" | bc)

echo "📈 RESULTS"
echo "=========="
echo -e "Baseline:     ${GREEN}${BASELINE_OPS}${NC} ops/sec"
echo -e "With Axiom:   ${GREEN}${AXIOM_OPS}${NC} ops/sec"
echo -e "Overhead:     ${YELLOW}${OVERHEAD}%${NC}"
echo ""

# Target: <1% overhead
if (( $(echo "$OVERHEAD < 1.0" | bc -l) )); then
    echo -e "${GREEN}✅ PASSED: Overhead < 1%${NC}"
else
    echo -e "${RED}❌ FAILED: Overhead >= 1%${NC}"
fi

echo ""
echo "🎯 TARGET: <1% overhead, <100ns latency"
echo "🏆 GOAL: 10x faster than Falco (~5% overhead)"
