#![allow(dead_code)]

//! BTF Resolver — CO-RE support for kernels without built-in BTF
//!
//! CO-RE (Compile Once, Run Everywhere) requires BTF (BPF Type Format) data.
//! Kernels < 5.8 or those compiled without CONFIG_DEBUG_INFO_BTF=y lack
//! /sys/kernel/btf/vmlinux. This module detects that and falls back to
//! BTFHub (https://github.com/aquasecurity/btfhub-archive) for pre-built BTF.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

// ============================================================================
// BTF STATUS
// ============================================================================

/// The resolved BTF status for the current kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BtfStatus {
    /// Kernel has native BTF at /sys/kernel/btf/vmlinux
    Native,
    /// BTF was resolved from BTFHub archive
    BtfHub(PathBuf),
    /// BTF is completely unavailable — eBPF CO-RE will not work
    Unavailable(String),
}

impl BtfStatus {
    pub fn is_available(&self) -> bool {
        !matches!(self, BtfStatus::Unavailable(_))
    }

    pub fn btf_path(&self) -> Option<PathBuf> {
        match self {
            BtfStatus::Native => Some(PathBuf::from("/sys/kernel/btf/vmlinux")),
            BtfStatus::BtfHub(path) => Some(path.clone()),
            BtfStatus::Unavailable(_) => None,
        }
    }
}

// ============================================================================
// BTF RESOLVER
// ============================================================================

/// Configuration for BTF resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtfConfig {
    /// Local cache directory for downloaded BTF files
    pub cache_dir: PathBuf,
    /// Whether to auto-download from BTFHub if native BTF is missing
    pub auto_download: bool,
    /// BTFHub archive base URL
    pub btfhub_base_url: String,
}

impl Default for BtfConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("/var/cache/nexus-axiom/btf"),
            auto_download: true,
            btfhub_base_url: "https://github.com/aquasecurity/btfhub-archive/raw/main"
                .to_string(),
        }
    }
}

pub struct BtfResolver {
    config: BtfConfig,
}

impl BtfResolver {
    pub fn new(config: BtfConfig) -> Self {
        Self { config }
    }

    /// Resolve BTF for the current kernel. Priority:
    /// 1. Native /sys/kernel/btf/vmlinux
    /// 2. Cached BTF file from previous download
    /// 3. Download from BTFHub archive
    /// 4. Unavailable (graceful degradation)
    pub async fn resolve(&self) -> BtfStatus {
        // 1. Check native BTF
        if self.check_native_btf() {
            info!("[BTF] Native BTF found at /sys/kernel/btf/vmlinux");
            return BtfStatus::Native;
        }
        warn!("[BTF] Native BTF not found — kernel may lack CONFIG_DEBUG_INFO_BTF");

        // 2. Get kernel release
        let kernel_release = match self.get_kernel_release() {
            Ok(kr) => kr,
            Err(e) => {
                error!("[BTF] Failed to get kernel release: {}", e);
                return BtfStatus::Unavailable(format!("Cannot determine kernel: {}", e));
            }
        };
        info!("[BTF] Kernel release: {}", kernel_release);

        // 3. Check cache
        let cached_path = self.cached_btf_path(&kernel_release);
        if cached_path.exists() {
            info!("[BTF] Using cached BTF: {:?}", cached_path);
            return BtfStatus::BtfHub(cached_path);
        }

        // 4. Download from BTFHub if enabled
        if self.config.auto_download {
            match self.download_btf(&kernel_release).await {
                Ok(path) => {
                    info!("[BTF] Downloaded BTF from BTFHub: {:?}", path);
                    return BtfStatus::BtfHub(path);
                }
                Err(e) => {
                    error!("[BTF] BTFHub download failed: {}", e);
                }
            }
        } else {
            warn!("[BTF] Auto-download disabled. Pre-provision BTF at {:?}", cached_path);
        }

        BtfStatus::Unavailable(
            "No native BTF and BTFHub download failed. eBPF CO-RE unavailable.".to_string(),
        )
    }

    fn check_native_btf(&self) -> bool {
        Path::new("/sys/kernel/btf/vmlinux").exists()
    }

    #[cfg(target_os = "linux")]
    fn get_kernel_release(&self) -> Result<String> {
        let output = std::process::Command::new("uname")
            .arg("-r")
            .output()
            .context("Failed to execute uname -r")?;
        if !output.status.success() {
            anyhow::bail!("uname -r failed with status {}", output.status);
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    #[cfg(not(target_os = "linux"))]
    fn get_kernel_release(&self) -> Result<String> {
        Ok("5.15.0-generic".to_string()) // Stub for non-Linux
    }

    fn cached_btf_path(&self, kernel_release: &str) -> PathBuf {
        self.config.cache_dir.join(format!("{}.btf", kernel_release))
    }

    /// Detect the distro to construct the correct BTFHub URL path
    #[cfg(target_os = "linux")]
    fn detect_distro(&self) -> Result<(String, String)> {
        let content = std::fs::read_to_string("/etc/os-release")
            .context("Failed to read /etc/os-release")?;
        let mut id = String::new();
        let mut version_id = String::new();
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("ID=") {
                id = val.trim_matches('"').to_string();
            }
            if let Some(val) = line.strip_prefix("VERSION_ID=") {
                version_id = val.trim_matches('"').to_string();
            }
        }
        if id.is_empty() {
            anyhow::bail!("Could not determine distro ID from /etc/os-release");
        }
        Ok((id, version_id))
    }

    #[cfg(not(target_os = "linux"))]
    fn detect_distro(&self) -> Result<(String, String)> {
        Ok(("ubuntu".to_string(), "22.04".to_string()))
    }

    /// Determine CPU architecture
    fn detect_arch(&self) -> String {
        if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "arm64".to_string()
        } else {
            std::env::consts::ARCH.to_string()
        }
    }

    /// Download BTF from BTFHub archive.
    /// URL format: {base}/{distro}/{version}/{arch}/{kernel_release}.btf
    async fn download_btf(&self, kernel_release: &str) -> Result<PathBuf> {
        let (distro, version) = self.detect_distro()?;
        let arch = self.detect_arch();

        let url = format!(
            "{}/{}/{}/{}/{}.btf",
            self.config.btfhub_base_url, distro, version, arch, kernel_release
        );
        info!("[BTF] Downloading from: {}", url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let response = client.get(&url).send().await?;
        if !response.status().is_success() {
            anyhow::bail!(
                "BTFHub returned {} for kernel {}",
                response.status(),
                kernel_release
            );
        }

        let btf_bytes = response.bytes().await?;

        // Verify minimum size (BTF files should be at least a few KB)
        if btf_bytes.len() < 1024 {
            anyhow::bail!(
                "Downloaded BTF too small ({} bytes) — likely not valid",
                btf_bytes.len()
            );
        }

        // Ensure cache directory exists
        std::fs::create_dir_all(&self.config.cache_dir)
            .context("Failed to create BTF cache directory")?;

        // Write to cache
        let dest = self.cached_btf_path(kernel_release);
        std::fs::write(&dest, &btf_bytes)
            .with_context(|| format!("Failed to write BTF to {:?}", dest))?;

        // Verify checksum (log it for manual verification)
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&btf_bytes);
        info!(
            "[BTF] SHA-256: {} ({} bytes)",
            hex::encode(hash),
            btf_bytes.len()
        );

        Ok(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btf_status_native() {
        let status = BtfStatus::Native;
        assert!(status.is_available());
        assert_eq!(
            status.btf_path().unwrap(),
            PathBuf::from("/sys/kernel/btf/vmlinux")
        );
    }

    #[test]
    fn test_btf_status_btfhub() {
        let path = PathBuf::from("/var/cache/nexus-axiom/btf/5.4.0-42-generic.btf");
        let status = BtfStatus::BtfHub(path.clone());
        assert!(status.is_available());
        assert_eq!(status.btf_path().unwrap(), path);
    }

    #[test]
    fn test_btf_status_unavailable() {
        let status = BtfStatus::Unavailable("test".to_string());
        assert!(!status.is_available());
        assert!(status.btf_path().is_none());
    }

    #[test]
    fn test_cached_path_format() {
        let resolver = BtfResolver::new(BtfConfig::default());
        let path = resolver.cached_btf_path("5.15.0-78-generic");
        assert_eq!(
            path,
            PathBuf::from("/var/cache/nexus-axiom/btf/5.15.0-78-generic.btf")
        );
    }
}
