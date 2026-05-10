#!/bin/bash
set -e

echo "🔬 Nexus Axiom Benchmark Suite"
echo "=============================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Must run as root"
    exit 1
fi

# Build benchmarks
cd benchmarks
make clean && make
cd ..

RESULTS_FILE="benchmark_results_$(date +%Y%m%d_%H%M%S).txt"

echo "📊 Running benchmarks..." | tee "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# 1. Baseline (no Nexus Axiom)
echo "1️⃣  Baseline (no protection)" | tee -a "$RESULTS_FILE"
echo "----------------------------" | tee -a "$RESULTS_FILE"
./benchmarks/bench_baseline | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# 2. Start Nexus Axiom
echo "2️⃣  Starting Nexus Axiom..." | tee -a "$RESULTS_FILE"
./target/release/nexus-axiom start --audit &
NEXUS_PID=$!
sleep 3

# 3. With Nexus Axiom (audit mode)
echo "3️⃣  With Nexus Axiom (audit mode)" | tee -a "$RESULTS_FILE"
echo "-----------------------------------" | tee -a "$RESULTS_FILE"
./benchmarks/bench_mmap | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# 4. Comprehensive benchmark
echo "4️⃣  Comprehensive syscall benchmark" | tee -a "$RESULTS_FILE"
echo "------------------------------------" | tee -a "$RESULTS_FILE"
./benchmarks/bench_comprehensive | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Stop Nexus Axiom
kill $NEXUS_PID 2>/dev/null || true
sleep 1

# 5. Calculate overhead
echo "📈 Performance Analysis" | tee -a "$RESULTS_FILE"
echo "=======================" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"
echo "Results saved to: $RESULTS_FILE" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"
echo "✅ Benchmark complete!" | tee -a "$RESULTS_FILE"
