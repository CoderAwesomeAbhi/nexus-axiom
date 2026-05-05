#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <sys/mman.h>
#include <unistd.h>
#include <string.h>

#define ITERATIONS 10000

double get_time_ns() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}

void benchmark_mmap() {
    printf("\n📊 Benchmark: mmap() latency\n");
    printf("─────────────────────────────\n");
    
    double total_ns = 0;
    int success = 0;
    
    for (int i = 0; i < ITERATIONS; i++) {
        double start = get_time_ns();
        
        void *mem = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                         MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
        
        double end = get_time_ns();
        
        if (mem != MAP_FAILED) {
            munmap(mem, 4096);
            total_ns += (end - start);
            success++;
        }
    }
    
    double avg_us = total_ns / success / 1000.0;
    printf("Iterations: %d\n", success);
    printf("Average latency: %.2f μs per mmap()\n", avg_us);
    printf("Throughput: %.0f ops/sec\n", 1e6 / avg_us);
}

void benchmark_mprotect() {
    printf("\n📊 Benchmark: mprotect() latency\n");
    printf("──────────────────────────────────\n");
    
    // Allocate memory once
    void *mem = mmap(NULL, 4096, PROT_READ|PROT_WRITE,
                     MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
    if (mem == MAP_FAILED) {
        printf("❌ Failed to allocate memory\n");
        return;
    }
    
    double total_ns = 0;
    int success = 0;
    
    for (int i = 0; i < ITERATIONS; i++) {
        double start = get_time_ns();
        
        int ret = mprotect(mem, 4096, PROT_READ);
        
        double end = get_time_ns();
        
        if (ret == 0) {
            total_ns += (end - start);
            success++;
            // Change back
            mprotect(mem, 4096, PROT_READ|PROT_WRITE);
        }
    }
    
    munmap(mem, 4096);
    
    double avg_us = total_ns / success / 1000.0;
    printf("Iterations: %d\n", success);
    printf("Average latency: %.2f μs per mprotect()\n", avg_us);
    printf("Throughput: %.0f ops/sec\n", 1e6 / avg_us);
}

void benchmark_syscall_overhead() {
    printf("\n📊 Benchmark: Basic syscall overhead\n");
    printf("──────────────────────────────────────\n");
    
    double total_ns = 0;
    
    for (int i = 0; i < ITERATIONS; i++) {
        double start = get_time_ns();
        getpid();
        double end = get_time_ns();
        total_ns += (end - start);
    }
    
    double avg_ns = total_ns / ITERATIONS;
    printf("Iterations: %d\n", ITERATIONS);
    printf("Average latency: %.0f ns per getpid()\n", avg_ns);
    printf("Throughput: %.0f ops/sec\n", 1e9 / avg_ns);
}

int main(int argc, char **argv) {
    int with_nexus = 0;
    
    if (argc > 1 && strcmp(argv[1], "--with-nexus") == 0) {
        with_nexus = 1;
    }
    
    printf("\n");
    printf("═══════════════════════════════════════════\n");
    printf("  NEXUS AXIOM PERFORMANCE BENCHMARKS\n");
    printf("═══════════════════════════════════════════\n");
    
    if (with_nexus) {
        printf("Mode: WITH Nexus Axiom eBPF hooks\n");
    } else {
        printf("Mode: BASELINE (no eBPF)\n");
    }
    
    benchmark_syscall_overhead();
    benchmark_mmap();
    benchmark_mprotect();
    
    printf("\n═══════════════════════════════════════════\n");
    printf("✅ Benchmarks complete\n");
    printf("═══════════════════════════════════════════\n\n");
    
    return 0;
}
