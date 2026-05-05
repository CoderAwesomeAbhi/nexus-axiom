# Makefile for CVE exploit tests

CC = gcc
CFLAGS = -Wall -O2

TARGETS = pwnkit dirty_pipe sudo_heap

all: $(TARGETS)

pwnkit: pwnkit.c
	$(CC) $(CFLAGS) -o pwnkit pwnkit.c

dirty_pipe: dirty_pipe.c
	$(CC) $(CFLAGS) -o dirty_pipe dirty_pipe.c

sudo_heap: sudo_heap.c
	$(CC) $(CFLAGS) -o sudo_heap sudo_heap.c

clean:
	rm -f $(TARGETS)

test: all
	@echo "Testing CVE exploits against Nexus Axiom..."
	@echo ""
	@echo "=== CVE-2021-4034 (PwnKit) ==="
	@sudo ./pwnkit || true
	@echo ""
	@echo "=== CVE-2022-0847 (Dirty Pipe) ==="
	@sudo ./dirty_pipe || true
	@echo ""
	@echo "=== CVE-2021-3156 (Sudo Heap) ==="
	@sudo ./sudo_heap || true

.PHONY: all clean test
