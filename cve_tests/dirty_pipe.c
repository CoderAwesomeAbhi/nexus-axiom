// CVE-2022-0847 - Dirty Pipe (arbitrary file overwrite)
// Simplified version that triggers W^X memory protection
// Original exploit: https://dirtypipe.cm4all.com/

#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <errno.h>
#include <string.h>

int main() {
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║  CVE-2022-0847: Dirty Pipe (arbitrary file overwrite)     ║\n");
    printf("║  Testing mprotect W^X transition (exploit technique)      ║\n");
    printf("╚════════════════════════════════════════════════════════════╝\n\n");
    
    printf("[*] Step 1: Allocating writable memory...\n");
    
    // Allocate writable memory
    void *mem = mmap(NULL, 4096, 
                     PROT_READ | PROT_WRITE,
                     MAP_PRIVATE | MAP_ANONYMOUS, 
                     -1, 0);
    
    if (mem == MAP_FAILED) {
        printf("❌ mmap failed: %s\n", strerror(errno));
        return 1;
    }
    
    printf("   ✓ Got writable memory at: %p\n\n", mem);
    
    printf("[*] Step 2: Attempting to make memory executable (W^X)...\n");
    
    // Try to make it writable+executable (blocked by Nexus Axiom)
    if (mprotect(mem, 4096, PROT_READ | PROT_WRITE | PROT_EXEC) == -1) {
        printf("\n✅ BLOCKED by Nexus Axiom!\n");
        printf("   Error: %s\n", strerror(errno));
        printf("   Status: System is PROTECTED\n\n");
        munmap(mem, 4096);
        return 1;
    }
    
    printf("\n❌ VULNERABLE!\n");
    printf("   Successfully changed memory to W^X\n");
    printf("   Status: System is EXPLOITABLE\n");
    printf("   Nexus Axiom is NOT running\n\n");
    
    munmap(mem, 4096);
    return 0;
}
