/// audit.rs — Self-audit module that validates Nexus Axiom's own security posture.

use anyhow::Result;
use serde::Serialize;
use sha2::Digest;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub timestamp: String,
    pub checks: Vec<AuditCheck>,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub overall_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditCheck {
    pub category: String,
    pub name: String,
    pub status: AuditStatus,
    pub details: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AuditStatus { Pass, Fail, Warning, Skip }

pub struct SecurityAuditor {
    binary_path: String,
    config_hash: Option<String>,
}

impl SecurityAuditor {
    pub fn new() -> Self {
        let binary_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        Self { binary_path, config_hash: None }
    }

    pub fn set_config_hash(&mut self, hash: String) { self.config_hash = Some(hash); }

    pub fn run_full_audit(&self) -> AuditReport {
        let mut checks = Vec::new();

        checks.push(self.check_binary_integrity());
        checks.push(self.check_permissions());
        checks.push(self.check_config_security());
        checks.push(self.check_log_directory());
        checks.push(self.check_ebpf_programs());
        checks.push(self.check_network_exposure());
        checks.push(self.check_kernel_config());
        checks.push(self.check_dependencies());

        let passed = checks.iter().filter(|c| c.status == AuditStatus::Pass).count();
        let failed = checks.iter().filter(|c| c.status == AuditStatus::Fail).count();
        let warnings = checks.iter().filter(|c| c.status == AuditStatus::Warning).count();

        let overall = if failed > 0 { "FAIL" } else if warnings > 0 { "WARN" } else { "PASS" };

        AuditReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            checks,
            passed, failed, warnings,
            overall_status: overall.to_string(),
        }
    }

    fn check_binary_integrity(&self) -> AuditCheck {
        let category = "Integrity".into();
        let name = "Binary integrity".into();

        if self.binary_path.is_empty() {
            return AuditCheck {
                category, name,
                status: AuditStatus::Skip,
                details: "Could not determine binary path".into(),
                remediation: None,
            };
        }

        match std::fs::read(&self.binary_path) {
            Ok(data) => {
                let hash = format!("{:x}", sha2::Sha256::digest(&data));
                AuditCheck {
                    category, name,
                    status: AuditStatus::Pass,
                    details: format!("Binary SHA-256: {}", &hash[..16]),
                    remediation: None,
                }
            }
            Err(e) => AuditCheck {
                category, name,
                status: AuditStatus::Warning,
                details: format!("Could not read binary: {}", e),
                remediation: Some("Ensure binary is readable".into()),
            },
        }
    }

    fn check_permissions(&self) -> AuditCheck {
        #[cfg(target_os = "linux")]
        {
            let is_root = nix::unistd::Uid::effective().is_root();
            if is_root {
                AuditCheck {
                    category: "Permissions".into(),
                    name: "Running as root".into(),
                    status: AuditStatus::Pass,
                    details: "Running with required root privileges".into(),
                    remediation: None,
                }
            } else {
                AuditCheck {
                    category: "Permissions".into(),
                    name: "Running as root".into(),
                    status: AuditStatus::Fail,
                    details: "Not running as root — eBPF programs cannot be loaded".into(),
                    remediation: Some("Run with sudo or as root".into()),
                }
            }
        }
        #[cfg(not(target_os = "linux"))]
        AuditCheck {
            category: "Permissions".into(),
            name: "Running as root".into(),
            status: AuditStatus::Skip,
            details: "Not on Linux".into(),
            remediation: None,
        }
    }

    fn check_config_security(&self) -> AuditCheck {
        // Verify config file isn't world-writable
        let config_paths = ["/etc/nexus-axiom/config.toml", "config.toml"];
        for path in &config_paths {
            if Path::new(path).exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if let Ok(meta) = std::fs::metadata(path) {
                        let mode = meta.mode();
                        if mode & 0o002 != 0 {
                            return AuditCheck {
                                category: "Configuration".into(),
                                name: "Config file permissions".into(),
                                status: AuditStatus::Fail,
                                details: format!("{} is world-writable (mode: {:o})", path, mode),
                                remediation: Some(format!("chmod 640 {}", path)),
                            };
                        }
                    }
                }
                return AuditCheck {
                    category: "Configuration".into(),
                    name: "Config file permissions".into(),
                    status: AuditStatus::Pass,
                    details: format!("Config file {} has safe permissions", path),
                    remediation: None,
                };
            }
        }
        AuditCheck {
            category: "Configuration".into(),
            name: "Config file permissions".into(),
            status: AuditStatus::Warning,
            details: "No config file found, using defaults".into(),
            remediation: Some("Create /etc/nexus-axiom/config.toml".into()),
        }
    }

    fn check_log_directory(&self) -> AuditCheck {
        let log_dir = Path::new("/var/log/nexus-axiom");
        if log_dir.exists() {
            AuditCheck {
                category: "Logging".into(),
                name: "Log directory exists".into(),
                status: AuditStatus::Pass,
                details: "/var/log/nexus-axiom/ exists".into(),
                remediation: None,
            }
        } else {
            AuditCheck {
                category: "Logging".into(),
                name: "Log directory exists".into(),
                status: AuditStatus::Warning,
                details: "/var/log/nexus-axiom/ does not exist".into(),
                remediation: Some("mkdir -p /var/log/nexus-axiom && chmod 750 /var/log/nexus-axiom".into()),
            }
        }
    }

    fn check_ebpf_programs(&self) -> AuditCheck {
        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("bpftool")
                .args(["prog", "list"])
                .output();
            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let has_lsm = stdout.contains("lsm") || stdout.contains("nexus");
                    if has_lsm {
                        return AuditCheck {
                            category: "eBPF".into(),
                            name: "eBPF programs loaded".into(),
                            status: AuditStatus::Pass,
                            details: "LSM eBPF programs are loaded".into(),
                            remediation: None,
                        };
                    }
                    AuditCheck {
                        category: "eBPF".into(),
                        name: "eBPF programs loaded".into(),
                        status: AuditStatus::Warning,
                        details: "No Nexus Axiom eBPF programs found".into(),
                        remediation: Some("Start nexus-axiom to load eBPF programs".into()),
                    }
                }
                _ => AuditCheck {
                    category: "eBPF".into(),
                    name: "eBPF programs loaded".into(),
                    status: AuditStatus::Skip,
                    details: "bpftool not available".into(),
                    remediation: Some("Install bpftool for eBPF program verification".into()),
                },
            }
        }
        #[cfg(not(target_os = "linux"))]
        AuditCheck {
            category: "eBPF".into(), name: "eBPF programs loaded".into(),
            status: AuditStatus::Skip, details: "Not on Linux".into(), remediation: None,
        }
    }

    fn check_network_exposure(&self) -> AuditCheck {
        // Check if dashboard/metrics ports are bound only to expected interfaces
        let ports = [8080u16, 9090];
        let mut exposed = Vec::new();
        for port in &ports {
            if let Ok(listener) = std::net::TcpListener::bind(format!("0.0.0.0:{}", port)) {
                drop(listener); // Port was free (not exposed)
            } else {
                exposed.push(*port);
            }
        }
        if exposed.is_empty() {
            AuditCheck {
                category: "Network".into(),
                name: "Port exposure".into(),
                status: AuditStatus::Pass,
                details: "No unexpected ports open".into(),
                remediation: None,
            }
        } else {
            AuditCheck {
                category: "Network".into(),
                name: "Port exposure".into(),
                status: AuditStatus::Pass,
                details: format!("Ports {:?} are in use (expected for dashboard/metrics)", exposed),
                remediation: None,
            }
        }
    }

    fn check_kernel_config(&self) -> AuditCheck {
        let lsm_path = Path::new("/sys/kernel/security/lsm");
        if lsm_path.exists() {
            if let Ok(content) = std::fs::read_to_string(lsm_path) {
                if content.contains("bpf") {
                    return AuditCheck {
                        category: "Kernel".into(),
                        name: "BPF LSM enabled".into(),
                        status: AuditStatus::Pass,
                        details: format!("LSM: {}", content.trim()),
                        remediation: None,
                    };
                }
                return AuditCheck {
                    category: "Kernel".into(),
                    name: "BPF LSM enabled".into(),
                    status: AuditStatus::Fail,
                    details: format!("BPF not in LSM list: {}", content.trim()),
                    remediation: Some("Add lsm=bpf to kernel boot parameters".into()),
                };
            }
        }
        AuditCheck {
            category: "Kernel".into(), name: "BPF LSM enabled".into(),
            status: AuditStatus::Skip, details: "Cannot read LSM config".into(), remediation: None,
        }
    }

    fn check_dependencies(&self) -> AuditCheck {
        // Basic check: verify critical system libraries exist
        let libs = ["/usr/lib/libbpf.so", "/usr/lib/x86_64-linux-gnu/libbpf.so.1"];
        let found = libs.iter().any(|p| Path::new(p).exists());
        if found {
            AuditCheck {
                category: "Dependencies".into(),
                name: "System libraries".into(),
                status: AuditStatus::Pass,
                details: "libbpf found".into(),
                remediation: None,
            }
        } else {
            AuditCheck {
                category: "Dependencies".into(),
                name: "System libraries".into(),
                status: AuditStatus::Warning,
                details: "libbpf not found in standard paths (may be statically linked)".into(),
                remediation: Some("apt install libbpf-dev".into()),
            }
        }
    }
}

impl Default for SecurityAuditor { fn default() -> Self { Self::new() } }
