.PHONY: all clean test

CC = gcc
CFLAGS = -Wall -O2

TESTS = test_cve_3156 test_pwnkit

all: $(TESTS)

test_cve_3156: test_cve_3156.c
	$(CC) $(CFLAGS) -o $@ $<

test_pwnkit: test_pwnkit.c
	$(CC) $(CFLAGS) -o $@ $<

test: all
	@echo "🧪 Testing CVE exploits WITHOUT Nexus Axiom..."
	@echo ""
	@echo "Test 1: CVE-2021-3156 (Sudo)"
	@./test_cve_3156 || true
	@echo ""
	@echo "Test 2: CVE-2021-4034 (PwnKit)"
	@./test_pwnkit || true
	@echo ""
	@echo "⚠️  All tests should show VULNERABLE without Nexus Axiom"
	@echo "✅ With Nexus Axiom running, all should be BLOCKED"

clean:
	rm -f $(TESTS)
