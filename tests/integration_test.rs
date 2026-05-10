use std::process::Command;
use std::time::Duration;

#[test]
fn test_binary_exists() {
    let output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to build");
    assert!(output.status.success(), "Build failed");
}

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to run help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Nexus Axiom"));
}

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to run version");
    assert!(output.status.success());
}

#[cfg(target_os = "linux")]
#[test]
fn test_ebpf_files_exist() {
    assert!(std::path::Path::new("ebpf/nexus_working.bpf.c").exists());
    assert!(std::path::Path::new("ebpf/nexus_net.bpf.c").exists());
}

#[test]
fn test_config_file_valid() {
    let config_path = "config.toml";
    if std::path::Path::new(config_path).exists() {
        let content = std::fs::read_to_string(config_path).unwrap();
        let _: toml::Value = toml::from_str(&content).expect("Invalid TOML");
    }
}
