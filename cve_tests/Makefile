CC = gcc
CFLAGS = -Wall -O2

all: test_pwnkit

test_pwnkit: test_pwnkit.c
	$(CC) $(CFLAGS) -o test_pwnkit test_pwnkit.c

clean:
	rm -f test_pwnkit

.PHONY: all clean
