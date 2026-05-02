// Benchmark: mmap operations with Nexus Axiom active
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>

#define ITERATIONS 100000

int main() {
    struct timespec start, end;
    long total_ns = 0;
    int success = 0;
    
    printf("Running %d mmap operations...\n", ITERATIONS);
    
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    for (int i = 0; i < ITERATIONS; i++) {
        // Try to allocate normal (non-W^X) memory
        void *addr = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                         MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        
        if (addr != MAP_FAILED) {
            success++;
            munmap(addr, 4096);
        }
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    total_ns = (end.tv_sec - start.tv_sec) * 1000000000L + 
               (end.tv_nsec - start.tv_nsec);
    
    double avg_ns = (double)total_ns / ITERATIONS;
    double ops_per_sec = 1000000000.0 / avg_ns;
    
    printf("Completed: %d/%d\n", success, ITERATIONS);
    printf("Total time: %.2f ms\n", total_ns / 1000000.0);
    printf("Average latency: %.2f ns\n", avg_ns);
    printf("Operations/sec: %.0f\n", ops_per_sec);
    
    return 0;
}
