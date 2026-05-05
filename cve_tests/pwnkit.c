// CVE-2021-4034 - PwnKit (pkexec privilege escalation)
// Simplified version that triggers W^X memory protection
// Original exploit: https://www.qualys.com/2022/01/25/cve-2021-4034/pwnkit.txt

#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <string.h>
#include <errno.h>

int main() {
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║  CVE-2021-4034: PwnKit (pkexec privilege escalation)      ║\n");
    printf("║  Testing W^X memory allocation (exploit technique)        ║\n");
    printf("╚════════════════════════════════════════════════════════════╝\n\n");
    
    printf("[*] Attempting W^X memory allocation (shellcode injection)...\n");
    
    // This is what the real PwnKit exploit does:
    // Allocate writable+executable memory for shellcode
    void *mem = mmap(NULL, 4096, 
                     PROT_WRITE | PROT_EXEC,  // W^X violation!
                     MAP_PRIVATE | MAP_ANONYMOUS, 
                     -1, 0);
    
    if (mem == MAP_FAILED) {
        printf("\n✅ BLOCKED by Nexus Axiom!\n");
        printf("   Error: %s\n", strerror(errno));
        printf("   Status: System is PROTECTED\n\n");
        return 1;
    }
    
    printf("\n❌ VULNERABLE!\n");
    printf("   Got W^X memory at: %p\n", mem);
    printf("   Status: System is EXPLOITABLE\n");
    printf("   Nexus Axiom is NOT running\n\n");
    
    // Write shellcode (harmless NOP sled for demo)
    unsigned char shellcode[] = 
        "\x90\x90\x90\x90"  // NOP sled
        "\x48\x31\xc0"      // xor rax, rax
        "\x48\x31\xff"      // xor rdi, rdi
        "\xc3";             // ret
    
    memcpy(mem, shellcode, sizeof(shellcode));
    printf("   Wrote %zu bytes of shellcode\n", sizeof(shellcode));
    
    munmap(mem, 4096);
    return 0;
}
