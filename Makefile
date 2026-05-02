.PHONY: all clean ebpf rust install test

CLANG ?= clang
CARGO ?= cargo

# eBPF compilation flags
BPF_CFLAGS = -O2 -target bpf -g -Wall -Werror
BPF_INCLUDES = -I/usr/include -I/usr/include/x86_64-linux-gnu

# Directories
EBPF_DIR = ebpf
BUILD_DIR = target

all: ebpf rust

# Compile eBPF program
ebpf:
	@echo "🔧 Compiling eBPF LSM program..."
	@mkdir -p $(BUILD_DIR)/bpf
	$(CLANG) $(BPF_CFLAGS) $(BPF_INCLUDES) \
		-c $(EBPF_DIR)/nexus_real.bpf.c \
		-o $(BUILD_DIR)/bpf/nexus_real.bpf.o
	@echo "✅ eBPF compiled: $(BUILD_DIR)/bpf/nexus_real.bpf.o"

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
	sudo cp $(BUILD_DIR)/bpf/nexus_real.bpf.o /usr/lib/nexus-axiom/
	@echo "✅ Installed to /usr/local/bin/nexus-axiom"

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
