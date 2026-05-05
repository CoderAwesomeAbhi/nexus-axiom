// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom v2.0 - eBPF XDP Network Filter with Port Blocking & Rate Limiting

typedef unsigned char __u8;
typedef unsigned short __u16;
typedef unsigned int __u32;
typedef unsigned long long __u64;

#define SEC(name) __attribute__((section(name), used))
#define BPF_MAP_TYPE_HASH 1
#define BPF_MAP_TYPE_LRU_HASH 9
#define __uint(name, val) int (*name)[val]
#define __type(name, val) typeof(val) *name

char LICENSE[] SEC("license") = "GPL";

// XDP actions
#define XDP_ABORTED 0
#define XDP_DROP 1
#define XDP_PASS 2
#define XDP_TX 3

// Protocol types
#define ETH_P_IP  0x0800
#define IPPROTO_TCP 6
#define IPPROTO_UDP 17

// Rate limiting: 1000 packets per second per IP
#define RATE_LIMIT_PPS 1000
#define RATE_WINDOW_NS 1000000000ULL  // 1 second in nanoseconds

struct xdp_md {
    __u32 data;
    __u32 data_end;
    __u32 data_meta;
    __u32 ingress_ifindex;
    __u32 rx_queue_index;
    __u32 egress_ifindex;
};

struct ethhdr {
    unsigned char h_dest[6];
    unsigned char h_source[6];
    __u16 h_proto;
};

struct iphdr {
#if defined(__LITTLE_ENDIAN_BITFIELD)
    __u8 ihl:4, version:4;
#elif defined (__BIG_ENDIAN_BITFIELD)
    __u8 version:4, ihl:4;
#else
    __u8 ihl:4, version:4;
#endif
    __u8 tos;
    __u16 tot_len;
    __u16 id;
    __u16 frag_off;
    __u8 ttl;
    __u8 protocol;
    __u16 check;
    __u32 saddr;
    __u32 daddr;
};

struct tcphdr {
    __u16 source;
    __u16 dest;
    __u32 seq;
    __u32 ack_seq;
    __u16 res1:4, doff:4, fin:1, syn:1, rst:1, psh:1, ack:1, urg:1, ece:1, cwr:1;
    __u16 window;
    __u16 check;
    __u16 urg_ptr;
};

struct udphdr {
    __u16 source;
    __u16 dest;
    __u16 len;
    __u16 check;
};

struct rate_limit_info {
    __u64 last_reset;
    __u32 packet_count;
};

// Map of malicious IPs to drop
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 65536);
    __type(key, __u32);
    __type(value, __u8);
} blocklist_ipv4 SEC(".maps");

// Map of blocked ports (e.g., 22 for SSH brute force protection)
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1024);
    __type(key, __u16);
    __type(value, __u8);
} blocked_ports SEC(".maps");

// Rate limiting per source IP
struct {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    __uint(max_entries, 10000);
    __type(key, __u32);
    __type(value, struct rate_limit_info);
} rate_limit_map SEC(".maps");

static __inline __u16 bpf_ntohs(__u16 val) {
    return (val << 8) | (val >> 8);
}

static void *(*bpf_map_lookup_elem)(void *map, const void *key) = (void *) 1;
static long (*bpf_map_update_elem)(void *map, const void *key, const void *value, __u64 flags) = (void *) 2;
static __u64 (*bpf_ktime_get_ns)(void) = (void *) 5;

SEC("xdp")
int network_filter(struct xdp_md *ctx)
{
    void *data_end = (void *)(long)ctx->data_end;
    void *data = (void *)(long)ctx->data;

    struct ethhdr *eth = data;
    if ((void *)(eth + 1) > data_end)
        return XDP_PASS;

    if (eth->h_proto != bpf_ntohs(ETH_P_IP))
        return XDP_PASS;

    struct iphdr *ip = (void *)(eth + 1);
    if ((void *)(ip + 1) > data_end)
        return XDP_PASS;

    __u32 src_ip = ip->saddr;

    // Check IP blocklist
    __u8 *blocked = bpf_map_lookup_elem(&blocklist_ipv4, &src_ip);
    if (blocked && *blocked == 1)
        return XDP_DROP;

    // Rate limiting
    __u64 now = bpf_ktime_get_ns();
    struct rate_limit_info *rate_info = bpf_map_lookup_elem(&rate_limit_map, &src_ip);
    
    if (rate_info) {
        if (now - rate_info->last_reset > RATE_WINDOW_NS) {
            rate_info->last_reset = now;
            rate_info->packet_count = 1;
        } else {
            rate_info->packet_count++;
            if (rate_info->packet_count > RATE_LIMIT_PPS)
                return XDP_DROP;
        }
    } else {
        struct rate_limit_info new_info = {
            .last_reset = now,
            .packet_count = 1,
        };
        bpf_map_update_elem(&rate_limit_map, &src_ip, &new_info, 0);
    }

    // Port filtering for TCP
    if (ip->protocol == IPPROTO_TCP) {
        struct tcphdr *tcp = (void *)(ip + 1);
        if ((void *)(tcp + 1) > data_end)
            return XDP_PASS;

        __u16 dport = bpf_ntohs(tcp->dest);
        __u8 *port_blocked = bpf_map_lookup_elem(&blocked_ports, &dport);
        if (port_blocked && *port_blocked == 1)
            return XDP_DROP;
    }

    // Port filtering for UDP
    if (ip->protocol == IPPROTO_UDP) {
        struct udphdr *udp = (void *)(ip + 1);
        if ((void *)(udp + 1) > data_end)
            return XDP_PASS;

        __u16 dport = bpf_ntohs(udp->dest);
        __u8 *port_blocked = bpf_map_lookup_elem(&blocked_ports, &dport);
        if (port_blocked && *port_blocked == 1)
            return XDP_DROP;
    }
    
    return XDP_PASS;
}
