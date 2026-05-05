#[cfg(target_os = "linux")]
use libbpf_cargo::SkeletonBuilder;
#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[cfg(target_os = "linux")]
const SRC_LSM: &str = "ebpf/nexus_working.bpf.c";
#[cfg(target_os = "linux")]
const SRC_XDP: &str = "ebpf/nexus_net.bpf.c";

#[cfg(target_os = "linux")]
fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set"));

    let mut out_lsm = out.clone();
    out_lsm.push("nexus_working.skel.rs");
    SkeletonBuilder::new()
        .source(SRC_LSM)
        .build_and_generate(&out_lsm)
        .expect("failed to generate LSM skeleton from ebpf/nexus_working.bpf.c");
    println!("cargo:rerun-if-changed={}", SRC_LSM);

    let mut out_xdp = out.clone();
    out_xdp.push("nexus_net.skel.rs");
    SkeletonBuilder::new()
        .source(SRC_XDP)
        .build_and_generate(&out_xdp)
        .expect("failed to generate XDP skeleton from ebpf/nexus_net.bpf.c");
    println!("cargo:rerun-if-changed={}", SRC_XDP);
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
