#!/bin/bash
set -e

echo "🔧 Generating vmlinux.h..."

if [ ! -f /sys/kernel/btf/vmlinux ]; then
    echo "❌ BTF not available. Kernel must be compiled with CONFIG_DEBUG_INFO_BTF=y"
    exit 1
fi

if ! command -v bpftool &> /dev/null; then
    echo "📦 Installing bpftool..."
    sudo apt-get update && sudo apt-get install -y linux-tools-common linux-tools-generic || true
fi

bpftool btf dump file /sys/kernel/btf/vmlinux format c > ebpf/vmlinux.h

echo "✅ Generated vmlinux.h ($(wc -l < ebpf/vmlinux.h) lines)"
