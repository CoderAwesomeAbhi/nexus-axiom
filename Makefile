.PHONY: all clean ebpf rust install test run

CLANG ?= clang
CARGO ?= cargo

# eBPF compilation flags
BPF_CFLAGS = -O2 -target bpf -g -Wall -Werror
ARCH := $(shell uname -m)
ARCH_INC := /usr/include/$(ARCH)-linux-gnu
BPF_INCLUDES = -I/usr/include $(if $(wildcard $(ARCH_INC)),-I$(ARCH_INC),)

# Directories
EBPF_DIR = ebpf
BUILD_DIR = target

all: ebpf rust

# Compile eBPF programs
ebpf:
	@echo "🔧 Compiling eBPF LSM program..."
	@mkdir -p $(BUILD_DIR)/bpf
	$(CLANG) $(BPF_CFLAGS) $(BPF_INCLUDES) \
		-c $(EBPF_DIR)/nexus_working.bpf.c \
		-o $(BUILD_DIR)/bpf/nexus_working.bpf.o
	@echo "✅ eBPF LSM compiled: $(BUILD_DIR)/bpf/nexus_working.bpf.o"
	@echo "🔧 Compiling eBPF XDP program..."
	$(CLANG) $(BPF_CFLAGS) $(BPF_INCLUDES) \
		-c $(EBPF_DIR)/nexus_net.bpf.c \
		-o $(BUILD_DIR)/bpf/nexus_net.bpf.o
	@echo "✅ eBPF XDP compiled: $(BUILD_DIR)/bpf/nexus_net.bpf.o"

# Compile Rust userspace
rust:
	@echo "🔧 Compiling Rust userspace..."
	$(CARGO) build --release
	@echo "✅ Binary: target/release/nexus-axiom"

# Install to system
install: all
	@echo "📦 Installing Nexus Axiom..."
	sudo cp target/release/nexus-axiom /usr/local/bin/
	sudo mkdir -p /usr/lib/nexus-axiom
	sudo cp $(BUILD_DIR)/bpf/nexus_working.bpf.o /usr/lib/nexus-axiom/
	@echo "✅ Installed to /usr/local/bin/nexus-axiom"

# End-to-end: compile + load + test (requires root)
run:
	@echo "🚀 Running full integration test..."
	sudo bash run.sh

# Run tests
test: all
	@echo "🧪 Running tests..."
	$(CARGO) test
	@echo "✅ Tests passed"

# Clean build artifacts
clean:
	$(CARGO) clean
	rm -rf $(BUILD_DIR)/bpf
	@echo "✅ Cleaned"

# Quick demo
demo: all
	@echo "🎬 Running demo..."
	@chmod +x demo.sh
	@./demo.sh
