// Nexus Axiom Client Library
// LD_PRELOAD this to connect apps to unprivileged Nexus daemon

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <linux/seccomp.h>
#include <linux/filter.h>
#include <sys/prctl.h>
#include <string.h>

static int nexus_sock = -1;

// Connect to Nexus daemon
static int connect_to_nexus(void) {
    struct sockaddr_un addr;
    int sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock < 0) {
        perror("socket");
        return -1;
    }
    
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, "/tmp/nexus-axiom.sock", sizeof(addr.sun_path) - 1);
    
    if (connect(sock, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        perror("connect to nexus");
        close(sock);
        return -1;
    }
    
    return sock;
}

// Install seccomp filter with user notification
static void install_seccomp_filter(void) {
    struct sock_filter filter[] = {
        // Load syscall number
        BPF_STMT(BPF_LD | BPF_W | BPF_ABS, offsetof(struct seccomp_data, nr)),
        
        // Check if mprotect (syscall 10)
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, 10, 0, 1),
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_USER_NOTIF),
        
        // Check if mmap (syscall 9)
        BPF_JUMP(BPF_JMP | BPF_JEQ | BPF_K, 9, 0, 1),
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_USER_NOTIF),
        
        // Allow everything else
        BPF_STMT(BPF_RET | BPF_K, SECCOMP_RET_ALLOW),
    };
    
    struct sock_fprog prog = {
        .len = sizeof(filter) / sizeof(filter[0]),
        .filter = filter,
    };
    
    if (prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) < 0) {
        perror("prctl NO_NEW_PRIVS");
        return;
    }
    
    if (prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog) < 0) {
        perror("prctl SECCOMP");
        return;
    }
    
    fprintf(stderr, "[Nexus] Seccomp filter installed\n");
}

// Constructor: runs before main()
__attribute__((constructor))
void nexus_init(void) {
    fprintf(stderr, "[Nexus] Initializing client library\n");
    
    // Connect to daemon
    nexus_sock = connect_to_nexus();
    if (nexus_sock < 0) {
        fprintf(stderr, "[Nexus] Warning: Could not connect to daemon\n");
        fprintf(stderr, "[Nexus] Start daemon with: nexus-axiom start --unprivileged\n");
        return;
    }
    
    fprintf(stderr, "[Nexus] Connected to daemon\n");
    
    // Install seccomp filter
    install_seccomp_filter();
    
    fprintf(stderr, "[Nexus] Protection active (unprivileged mode)\n");
}

// Destructor: cleanup
__attribute__((destructor))
void nexus_cleanup(void) {
    if (nexus_sock >= 0) {
        close(nexus_sock);
    }
}
