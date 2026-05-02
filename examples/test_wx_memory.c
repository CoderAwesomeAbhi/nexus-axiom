// Test W^X memory allocation (simulates exploit behavior)
// This will be BLOCKED by Nexus Axiom

#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <unistd.h>
#include <string.h>

int main() {
    printf("[*] Testing W^X memory allocation...\n");
    printf("[*] This simulates exploit behavior (JIT spraying, ROP chains, etc.)\n\n");
    
    // Attempt to allocate W^X memory (what exploits do)
    void *addr = mmap(NULL, 4096, 
                      PROT_WRITE | PROT_EXEC,  // ← This is what Nexus Axiom blocks
                      MAP_PRIVATE | MAP_ANONYMOUS, 
                      -1, 0);
    
    if (addr == MAP_FAILED) {
        printf("[✓] SAFE: W^X memory allocation blocked\n");
        printf("[✓] Nexus Axiom is protecting your system\n");
        return 0;
    }
    
    printf("[✗] VULNERABLE: Got W^X memory at %p\n", addr);
    printf("[✗] System is vulnerable to exploits\n");
    printf("[✗] Install Nexus Axiom to protect your system\n");
    
    // Write shellcode (simulated)
    unsigned char shellcode[] = {
        0x48, 0x31, 0xc0,  // xor rax, rax
        0xc3               // ret
    };
    memcpy(addr, shellcode, sizeof(shellcode));
    
    printf("[✗] Shellcode written to W^X memory\n");
    printf("[✗] Exploit would succeed on this system\n");
    
    munmap(addr, 4096);
    return 1;
}
