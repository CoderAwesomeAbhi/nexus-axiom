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

    // Try to build LSM skeleton
    let mut out_lsm = out.clone();
    out_lsm.push("nexus_working.skel.rs");
    match SkeletonBuilder::new()
        .source(SRC_LSM)
        .build_and_generate(&out_lsm)
    {
        Ok(_) => println!("cargo:warning=LSM skeleton generated successfully"),
        Err(e) => {
            println!("cargo:warning=Failed to generate LSM skeleton: {}", e);
            println!("cargo:warning=This is expected in CI without full BPF headers");
            // Create empty skeleton file so compilation can continue
            std::fs::write(&out_lsm, "// Skeleton generation failed\n").ok();
        }
    }
    println!("cargo:rerun-if-changed={}", SRC_LSM);

    // Try to build XDP skeleton
    let mut out_xdp = out.clone();
    out_xdp.push("nexus_net.skel.rs");
    match SkeletonBuilder::new()
        .source(SRC_XDP)
        .build_and_generate(&out_xdp)
    {
        Ok(_) => println!("cargo:warning=XDP skeleton generated successfully"),
        Err(e) => {
            println!("cargo:warning=Failed to generate XDP skeleton: {}", e);
            println!("cargo:warning=This is expected in CI without full BPF headers");
            // Create empty skeleton file so compilation can continue
            std::fs::write(&out_xdp, "// Skeleton generation failed\n").ok();
        }
    }
    println!("cargo:rerun-if-changed={}", SRC_XDP);
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
