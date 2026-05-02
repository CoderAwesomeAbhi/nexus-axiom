// SPDX-License-Identifier: GPL-2.0
// Nexus Axiom - eBPF XDP Network Filter

typedef unsigned char __u8;
typedef unsigned short __u16;
typedef unsigned int __u32;
typedef unsigned long long __u64;

#define SEC(name) __attribute__((section(name), used))
#define BPF_MAP_TYPE_HASH 1
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
    __u8 ihl:4, version:4; // fallback to little endian
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

// Map of malicious IPs to drop
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 65536);
    __type(key, __u32);   // IPv4 address
    __type(value, __u8);  // 1 = Drop
} blocklist_ipv4 SEC(".maps");

static __inline __u16 bpf_ntohs(__u16 val) {
    return (val << 8) | (val >> 8);
}

static void *(*bpf_map_lookup_elem)(void *map, const void *key) = (void *) 1;

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

    __u8 *blocked = bpf_map_lookup_elem(&blocklist_ipv4, &src_ip);
    if (blocked && *blocked == 1) {
        return XDP_DROP;
    }
    
    return XDP_PASS;
}
