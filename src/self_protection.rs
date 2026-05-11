// Self-protection: Detect tampering attempts against Nexus Axiom itself.
// Includes binary integrity, config tampering, eBPF program verification.

use anyhow::Result;
use sha2::Digest;
use std::fs;
use std::path::Path;

pub struct SelfProtection {
    daemon_pid: u32,
    map_paths: Vec<String>,
    binary_hash: Option<String>,
    config_hash: Option<String>,
}

impl SelfProtection {
    pub fn new(daemon_pid: u32) -> Self {
        let binary_hash = Self::compute_binary_hash();
        Self {
            daemon_pid,
            map_paths: vec![
                "/sys/fs/bpf/nexus_mmap_events".to_string(),
                "/sys/fs/bpf/nexus_mprotect_events".to_string(),
            ],
            binary_hash,
            config_hash: None,
        }
    }

    /// Snapshot config file hash at startup.
    pub fn snapshot_config(&mut self, config_path: &str) {
        self.config_hash = Self::hash_file(config_path);
        log::info!(
            "🔐 Config hash snapshot: {}",
            self.config_hash.as_deref().unwrap_or("none")
        );
    }

    pub fn check_integrity(&self) -> Result<Vec<TamperAttempt>> {
        let mut attempts = Vec::new();

        // Check if daemon process still exists
        if !Path::new(&format!("/proc/{}", self.daemon_pid)).exists() {
            attempts.push(TamperAttempt::DaemonKilled);
        }

        // Check if eBPF maps still exist
        for map_path in &self.map_paths {
            if !Path::new(map_path).exists() {
                attempts.push(TamperAttempt::MapDeleted(map_path.clone()));
            }
        }

        // Check binary integrity
        if let Some(ref original_hash) = self.binary_hash {
            if let Some(current_hash) = Self::compute_binary_hash() {
                if current_hash != *original_hash {
                    attempts.push(TamperAttempt::BinaryModified {
                        expected: original_hash.clone(),
                        actual: current_hash,
                    });
                }
            }
        }

        // Check config integrity
        if let Some(ref original_hash) = self.config_hash {
            let config_paths = ["config.toml", "/etc/nexus-axiom/config.toml"];
            for path in &config_paths {
                if let Some(current_hash) = Self::hash_file(path) {
                    if current_hash != *original_hash {
                        attempts.push(TamperAttempt::ConfigModified(path.to_string()));
                    }
                    break;
                }
            }
        }

        // Check for unauthorized bpf syscalls
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(pid_str) = entry.file_name().into_string() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        if pid != self.daemon_pid && self.is_using_bpf(pid) {
                            attempts.push(TamperAttempt::UnauthorizedBpfSyscall(pid));
                        }
                    }
                }
            }
        }

        // Check eBPF program pinning
        if !Path::new("/sys/fs/bpf").exists() {
            attempts.push(TamperAttempt::BpfFsUnmounted);
        }

        Ok(attempts)
    }

    fn is_using_bpf(&self, pid: u32) -> bool {
        if let Ok(entries) = fs::read_dir(format!("/proc/{}/fd", pid)) {
            for entry in entries.flatten() {
                if let Ok(link) = fs::read_link(entry.path()) {
                    if link.to_string_lossy().contains("/sys/fs/bpf/nexus") {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn respond_to_tamper(&self, attempt: &TamperAttempt) -> Result<()> {
        match attempt {
            TamperAttempt::DaemonKilled => {
                log::error!("🚨 CRITICAL: Nexus Axiom daemon was killed!");
                // Trigger restart via systemd watchdog
            }
            TamperAttempt::MapDeleted(path) => {
                log::error!("🚨 CRITICAL: eBPF map deleted: {}", path);
            }
            TamperAttempt::UnauthorizedBpfSyscall(pid) => {
                log::warn!("⚠️  Unauthorized BPF access from PID {}", pid);
                unsafe {
                    libc::kill(*pid as i32, libc::SIGKILL);
                }
            }
            TamperAttempt::BinaryModified { expected, actual } => {
                log::error!(
                    "🚨 CRITICAL: Binary modified! Expected: {}.. Got: {}..",
                    &expected[..8],
                    &actual[..8]
                );
            }
            TamperAttempt::ConfigModified(path) => {
                log::error!("🚨 Config file tampered: {}", path);
            }
            TamperAttempt::BpfFsUnmounted => {
                log::error!("🚨 CRITICAL: bpffs unmounted!");
            }
        }
        Ok(())
    }

    fn compute_binary_hash() -> Option<String> {
        let exe = std::env::current_exe().ok()?;
        Self::hash_file(exe.to_str()?)
    }

    fn hash_file(path: &str) -> Option<String> {
        let data = fs::read(path).ok()?;
        Some(format!("{:x}", sha2::Sha256::digest(&data)))
    }
}

#[derive(Debug)]
pub enum TamperAttempt {
    DaemonKilled,
    MapDeleted(String),
    UnauthorizedBpfSyscall(u32),
    BinaryModified { expected: String, actual: String },
    ConfigModified(String),
    BpfFsUnmounted,
}
