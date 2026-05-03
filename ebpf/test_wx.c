// test_wx.c — Tests that W^X mmap/mprotect is blocked by the eBPF LSM.
//
// Compile: gcc -O0 -o test_wx test_wx.c
// Run:     ./test_wx
//
// Expected output with LSM loaded:
//   [PASS] mmap PROT_WRITE|PROT_EXEC blocked: Operation not permitted
//   [PASS] mprotect RW->RWX blocked: Operation not permitted
//
// Expected output WITHOUT LSM:
//   [FAIL] mmap PROT_WRITE|PROT_EXEC was NOT blocked
//   [FAIL] mprotect RW->RWX was NOT blocked

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/mman.h>

int main(void)
{
    int all_pass = 1;

    // ── Test 1: mmap with PROT_WRITE|PROT_EXEC ──────────────────────────────
    void *p = mmap(NULL, 4096,
                   PROT_READ | PROT_WRITE | PROT_EXEC,
                   MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (p == MAP_FAILED && errno == EPERM) {
        printf("[PASS] mmap PROT_WRITE|PROT_EXEC blocked: %s\n", strerror(errno));
    } else {
        printf("[FAIL] mmap PROT_WRITE|PROT_EXEC was NOT blocked\n");
        all_pass = 0;
        if (p != MAP_FAILED) munmap(p, 4096);
    }

    // ── Test 2: mprotect RW → RWX ────────────────────────────────────────────
    void *q = mmap(NULL, 4096,
                   PROT_READ | PROT_WRITE,
                   MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (q == MAP_FAILED) {
        printf("[SKIP] mmap RW failed unexpectedly: %s\n", strerror(errno));
    } else {
        int rc = mprotect(q, 4096, PROT_READ | PROT_WRITE | PROT_EXEC);
        if (rc == -1 && errno == EPERM) {
            printf("[PASS] mprotect RW->RWX blocked: %s\n", strerror(errno));
        } else {
            printf("[FAIL] mprotect RW->RWX was NOT blocked\n");
            all_pass = 0;
        }
        munmap(q, 4096);
    }

    return all_pass ? 0 : 1;
}
