CC = gcc
CFLAGS = -Wall -O2

all: test_wx_memory test_pwnkit test_mprotect

test_wx_memory: test_wx_memory.c
	$(CC) $(CFLAGS) -o test_wx_memory test_wx_memory.c

test_pwnkit: test_pwnkit.c
	$(CC) $(CFLAGS) -o test_pwnkit test_pwnkit.c

test_mprotect: test_mprotect.c
	$(CC) $(CFLAGS) -o test_mprotect test_mprotect.c

clean:
	rm -f test_wx_memory test_pwnkit test_mprotect

.PHONY: all clean
