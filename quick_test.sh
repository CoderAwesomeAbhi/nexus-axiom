#!/bin/bash
# Quick test script for Nexus Axiom

set -e

echo "🔧 Compiling minimal working eBPF..."
clang -O2 -target bpf -g -c ebpf/working_lsm.bpf.c -o /tmp/working_lsm.bpf.o -I/usr/include -I/usr/include/x86_64-linux-gnu

echo "✅ Compiled successfully!"
echo ""
echo "📦 Loading eBPF program..."
sudo bpftool prog load /tmp/working_lsm.bpf.o /sys/fs/bpf/nexus_test type lsm

echo "✅ Loaded successfully!"
echo ""
echo "🔗 Attaching to LSM hooks..."
PROG_ID=$(sudo bpftool prog show name mmap_file -j | jq '.[0].id')
sudo bpftool prog attach id $PROG_ID lsm mmap_file

PROG_ID2=$(sudo bpftool prog show name file_mprotect -j | jq '.[0].id')
sudo bpftool prog attach id $PROG_ID2 lsm file_mprotect

echo "✅ Attached successfully!"
echo ""
echo "🧪 Testing W^X blocking..."
echo ""

# Create test program
cat > /tmp/test_wx.c << 'EOF'
#include <stdio.h>
#include <sys/mman.h>
#include <string.h>

int main() {
    printf("[*] Attempting W^X memory allocation...\n");
    
    void *mem = mmap(NULL, 4096, PROT_READ | PROT_WRITE | PROT_EXEC,
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    
    if (mem == MAP_FAILED) {
        printf("[✅] BLOCKED! W^X allocation denied by Nexus Axiom\n");
        return 0;
    }
    
    printf("[❌] VULNERABLE! Got W^X memory at %p\n", mem);
    return 1;
}
EOF

gcc /tmp/test_wx.c -o /tmp/test_wx
/tmp/test_wx

echo ""
echo "🎉 Test complete!"
