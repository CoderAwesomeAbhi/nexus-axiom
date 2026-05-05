// Test mprotect() to W^X (another exploit technique)

#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <string.h>

int main() {
    printf("[*] Testing mprotect() to W^X...\n");
    printf("[*] This simulates ROP chain execution\n\n");
    
    // Step 1: Allocate RW memory
    void *addr = mmap(NULL, 4096,
                      PROT_READ | PROT_WRITE,
                      MAP_PRIVATE | MAP_ANONYMOUS,
                      -1, 0);
    
    if (addr == MAP_FAILED) {
        perror("mmap");
        return 1;
    }
    
    printf("[*] Allocated RW memory at %p\n", addr);
    
    // Step 2: Write shellcode
    unsigned char shellcode[] = { 0x90, 0x90, 0x90, 0xc3 }; // nop; nop; nop; ret
    memcpy(addr, shellcode, sizeof(shellcode));
    printf("[*] Wrote shellcode to memory\n");
    
    // Step 3: Try to make it executable (W^X)
    printf("[*] Attempting mprotect() to W^X...\n");
    
    if (mprotect(addr, 4096, PROT_WRITE | PROT_EXEC) == -1) {
        printf("[✓] BLOCKED: Nexus Axiom prevented mprotect() to W^X\n");
        printf("[✓] ROP chain execution would FAIL\n");
        munmap(addr, 4096);
        return 0;
    }
    
    printf("[✗] VULNERABLE: mprotect() to W^X succeeded\n");
    printf("[✗] ROP chain could execute on this system\n");
    
    munmap(addr, 4096);
    return 1;
}
