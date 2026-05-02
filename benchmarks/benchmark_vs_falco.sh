#!/bin/bash
# Comprehensive Performance Benchmark vs Falco
# Measures overhead, latency, throughput, and CPU usage

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "🚀 NEXUS AXIOM vs FALCO - PERFORMANCE COMPARISON"
echo "================================================"
echo ""

# Check root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Must run as root${NC}"
    exit 1
fi

# Check if tools exist
if [ ! -f "target/release/nexus-axiom" ]; then
    echo -e "${RED}Error: Build Nexus Axiom first${NC}"
    exit 1
fi

# Compile benchmarks
echo "📦 Compiling benchmark programs..."
gcc -O2 -o bench_mmap bench_mmap.c
gcc -O2 -o bench_baseline bench_baseline.c
echo ""

# Test 1: Baseline (no protection)
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 1: BASELINE (No Protection)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
./bench_baseline > /tmp/baseline.txt
BASELINE_OPS=$(grep "Operations/sec" /tmp/baseline.txt | awk '{print $2}')
BASELINE_LATENCY=$(grep "Average latency" /tmp/baseline.txt | awk '{print $3}')
cat /tmp/baseline.txt
echo ""

# Test 2: With Nexus Axiom
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 2: WITH NEXUS AXIOM${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Start Nexus Axiom
./target/release/nexus-axiom start > /dev/null 2>&1 &
AXIOM_PID=$!
sleep 2

# Measure CPU usage
CPU_BEFORE=$(ps -p $AXIOM_PID -o %cpu= | tr -d ' ')

./bench_mmap > /tmp/axiom.txt
AXIOM_OPS=$(grep "Operations/sec" /tmp/axiom.txt | awk '{print $2}')
AXIOM_LATENCY=$(grep "Average latency" /tmp/axiom.txt | awk '{print $3}')
cat /tmp/axiom.txt

# Measure CPU usage after
CPU_AFTER=$(ps -p $AXIOM_PID -o %cpu= | tr -d ' ')

kill $AXIOM_PID 2>/dev/null || true
wait $AXIOM_PID 2>/dev/null || true
echo ""

# Test 3: With Falco (if installed)
FALCO_INSTALLED=false
if command -v falco &> /dev/null; then
    FALCO_INSTALLED=true
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Test 3: WITH FALCO${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    falco > /dev/null 2>&1 &
    FALCO_PID=$!
    sleep 2
    
    ./bench_mmap > /tmp/falco.txt
    FALCO_OPS=$(grep "Operations/sec" /tmp/falco.txt | awk '{print $2}')
    FALCO_LATENCY=$(grep "Average latency" /tmp/falco.txt | awk '{print $3}')
    cat /tmp/falco.txt
    
    kill $FALCO_PID 2>/dev/null || true
    wait $FALCO_PID 2>/dev/null || true
    echo ""
fi

# Calculate results
AXIOM_OVERHEAD=$(echo "scale=2; (($BASELINE_OPS - $AXIOM_OPS) / $BASELINE_OPS) * 100" | bc)
AXIOM_LATENCY_INCREASE=$(echo "scale=2; $AXIOM_LATENCY - $BASELINE_LATENCY" | bc)

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}📊 RESULTS SUMMARY${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo "┌─────────────────────┬──────────────┬──────────────┬──────────────┐"
echo "│ Metric              │ Baseline     │ Nexus Axiom  │ Difference   │"
echo "├─────────────────────┼──────────────┼──────────────┼──────────────┤"
printf "│ Operations/sec      │ %-12s │ %-12s │ %-12s │\n" "$BASELINE_OPS" "$AXIOM_OPS" "-"
printf "│ Latency (ns)        │ %-12s │ %-12s │ +%-11s │\n" "$BASELINE_LATENCY" "$AXIOM_LATENCY" "$AXIOM_LATENCY_INCREASE"
printf "│ Overhead            │ %-12s │ %-12s │ %-12s │\n" "0%" "${AXIOM_OVERHEAD}%" "${AXIOM_OVERHEAD}%"
printf "│ CPU Usage           │ %-12s │ %-12s │ %-12s │\n" "0%" "${CPU_AFTER}%" "${CPU_AFTER}%"
echo "└─────────────────────┴──────────────┴──────────────┴──────────────┘"
echo ""

if [ "$FALCO_INSTALLED" = true ]; then
    FALCO_OVERHEAD=$(echo "scale=2; (($BASELINE_OPS - $FALCO_OPS) / $BASELINE_OPS) * 100" | bc)
    
    echo "┌─────────────────────┬──────────────┬──────────────┐"
    echo "│ Comparison          │ Nexus Axiom  │ Falco        │"
    echo "├─────────────────────┼──────────────┼──────────────┤"
    printf "│ Overhead            │ %-12s │ %-12s │\n" "${AXIOM_OVERHEAD}%" "${FALCO_OVERHEAD}%"
    printf "│ Operations/sec      │ %-12s │ %-12s │\n" "$AXIOM_OPS" "$FALCO_OPS"
    echo "└─────────────────────┴──────────────┴──────────────┘"
    echo ""
    
    IMPROVEMENT=$(echo "scale=2; $FALCO_OVERHEAD / $AXIOM_OVERHEAD" | bc)
    echo -e "${GREEN}🏆 Nexus Axiom is ${IMPROVEMENT}x faster than Falco${NC}"
fi

# Pass/Fail
echo ""
if (( $(echo "$AXIOM_OVERHEAD < 1.0" | bc -l) )); then
    echo -e "${GREEN}✅ PASSED: Overhead < 1%${NC}"
else
    echo -e "${YELLOW}⚠️  WARNING: Overhead >= 1%${NC}"
fi

if (( $(echo "$AXIOM_LATENCY_INCREASE < 100" | bc -l) )); then
    echo -e "${GREEN}✅ PASSED: Latency increase < 100ns${NC}"
else
    echo -e "${YELLOW}⚠️  WARNING: Latency increase >= 100ns${NC}"
fi

echo ""
echo "🎯 Target: <1% overhead, <100ns latency increase"
echo "📝 Results saved to /tmp/benchmark_results.txt"

# Save results
cat > /tmp/benchmark_results.txt << EOF
Nexus Axiom Performance Benchmark Results
==========================================

Date: $(date)
Kernel: $(uname -r)
CPU: $(lscpu | grep "Model name" | cut -d: -f2 | xargs)

Baseline:
- Operations/sec: $BASELINE_OPS
- Latency: $BASELINE_LATENCY ns

With Nexus Axiom:
- Operations/sec: $AXIOM_OPS
- Latency: $AXIOM_LATENCY ns
- Overhead: $AXIOM_OVERHEAD%
- CPU Usage: $CPU_AFTER%

$(if [ "$FALCO_INSTALLED" = true ]; then
    echo "With Falco:"
    echo "- Operations/sec: $FALCO_OPS"
    echo "- Latency: $FALCO_LATENCY ns"
    echo "- Overhead: $FALCO_OVERHEAD%"
    echo ""
    echo "Improvement: ${IMPROVEMENT}x faster than Falco"
fi)
EOF

echo ""
echo "✅ Benchmark complete!"
