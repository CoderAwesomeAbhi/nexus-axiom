use libbpf_cargo::SkeletonBuilder;
use std::env;
use std::path::PathBuf;

const SRC_LSM: &str = "ebpf/nexus_real.bpf.c";
const SRC_XDP: &str = "ebpf/nexus_net.bpf.c";

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set"));

    // Build LSM
    let mut out_lsm = out.clone();
    out_lsm.push("nexus_real.skel.rs");
    SkeletonBuilder::new()
        .source(SRC_LSM)
        .build_and_generate(&out_lsm)
        .unwrap();
    println!("cargo:rerun-if-changed={}", SRC_LSM);

    // Build XDP
    let mut out_xdp = out.clone();
    out_xdp.push("nexus_net.skel.rs");
    SkeletonBuilder::new()
        .source(SRC_XDP)
        .build_and_generate(&out_xdp)
        .unwrap();
    println!("cargo:rerun-if-changed={}", SRC_XDP);
}
