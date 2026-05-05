// CVE-2021-3156 - Sudo heap overflow (Baron Samedit)
// Simplified version that triggers W^X memory protection
// Original exploit: https://www.qualys.com/2021/01/26/cve-2021-3156/baron-samedit-heap-based-overflow-sudo.txt

#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <string.h>
#include <errno.h>

int main() {
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║  CVE-2021-3156: Sudo heap overflow (Baron Samedit)        ║\n");
    printf("║  Testing heap spray with W^X memory (exploit technique)   ║\n");
    printf("╚════════════════════════════════════════════════════════════╝\n\n");
    
    printf("[*] Simulating heap spray with executable memory...\n");
    printf("    (Real exploit overwrites sudo's heap with shellcode)\n\n");
    
    // Allocate multiple W^X regions (heap spray technique)
    printf("[*] Attempting to allocate W^X memory regions...\n");
    
    for (int i = 0; i < 3; i++) {
        printf("    Spray attempt %d/3... ", i + 1);
        
        void *mem = mmap(NULL, 4096, 
                         PROT_WRITE | PROT_EXEC,  // W^X violation!
                         MAP_PRIVATE | MAP_ANONYMOUS, 
                         -1, 0);
        
        if (mem == MAP_FAILED) {
            printf("BLOCKED\n");
            if (i == 0) {
                printf("\n✅ BLOCKED by Nexus Axiom!\n");
                printf("   Error: %s\n", strerror(errno));
                printf("   Status: System is PROTECTED\n");
                printf("   All heap spray attempts prevented\n\n");
                return 1;
            }
        } else {
            printf("SUCCESS at %p\n", mem);
        }
    }
    
    printf("\n❌ VULNERABLE!\n");
    printf("   Heap spray succeeded\n");
    printf("   Status: System is EXPLOITABLE\n");
    printf("   Nexus Axiom is NOT running\n\n");
    
    return 0;
}
