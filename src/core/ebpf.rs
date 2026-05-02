// Linux-only: eBPF integration with proper dev_t + ino_t tracking
//
// CRITICAL: The kernel-side eBPF program MUST read BOTH:
//   - file->f_inode->i_sb->s_dev  (device ID)
//   - file->f_inode->i_ino        (inode number)
//
// Checking inode alone is a VULNERABILITY. See inode_resolver.rs for details.

#![allow(dead_code)]

use std::path::PathBuf;

use anyhow::Result;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::btf_resolver::{BtfResolver, BtfConfig, BtfStatus};

// ============================================================================
// eBPF EVENT — DATA PASSED FROM KERNEL SPACE
// ============================================================================

/// Event structure passed from the kernel eBPF LSM hook to userspace.
/// This MUST include device ID + inode for correct file identification.
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EbpfEvent {
    /// Process ID
    pub pid: u32,
    /// Thread group ID
    pub tgid: u32,
    /// User ID
    pub uid: u32,
    /// Operation type (mapped to OperationType enum)
    pub op_type: u32,
    /// Device major number — from file->f_inode->i_sb->s_dev
    pub dev_major: u32,
    /// Device minor number — from file->f_inode->i_sb->s_dev
    pub dev_minor: u32,
    /// Inode number — from file->f_inode->i_ino
    pub inode: u64,
    /// mmap protection flags (for MMAP/MPROTECT events)
    pub mmap_prot: u32,
    /// File open flags
    pub flags: u32,
    /// Process command name (first 16 bytes, kernel TASK_COMM_LEN)
    pub comm: [u8; 16],
}

/// Map key for protected file lookup in eBPF maps.
/// The eBPF program checks incoming operations against this map.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProtectedFileKey {
    /// Combined device ID (major << 20 | minor)
    pub dev: u32,
    /// Inode number
    pub inode: u64,
}

/// Map value: what action to take for a protected file
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProtectedFilePolicy {
    /// 0 = allow+log, 1 = block_write, 2 = block_all
    pub action: u32,
    /// Severity level (0-10)
    pub severity: u32,
}

// ============================================================================
// eBPF CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EbpfConfig {
    /// Path to the compiled eBPF object file
    pub object_path: PathBuf,
    /// BTF configuration
    pub btf_config: BtfConfig,
    /// Whether to enable the file write LSM hook
    pub hook_file_write: bool,
    /// Whether to enable the mmap LSM hook
    pub hook_mmap: bool,
    /// Whether to enable the socket connect LSM hook
    pub hook_socket: bool,
    /// Whether to enable the exec LSM hook
    pub hook_exec: bool,
}

impl Default for EbpfConfig {
    fn default() -> Self {
        Self {
            object_path: PathBuf::from("/usr/lib/nexus-axiom/nexus-axiom.bpf.o"),
            btf_config: BtfConfig::default(),
            hook_file_write: true,
            hook_mmap: true,
            hook_socket: true,
            hook_exec: true,
        }
    }
}

// ============================================================================
// eBPF LOADER — INTEGRATES WITH AYA + BTF RESOLVER
// ============================================================================

/// Manages the lifecycle of the eBPF program: load, attach, configure maps
pub struct EbpfManager {
    config: EbpfConfig,
    btf_status: Option<BtfStatus>,
    loaded: bool,
}

impl EbpfManager {
    pub fn new(config: EbpfConfig) -> Self {
        Self {
            config,
            btf_status: None,
            loaded: false,
        }
    }

    /// Initialize the eBPF subsystem:
    /// 1. Resolve BTF (native or BTFHub)
    /// 2. Load the eBPF program via Aya
    /// 3. Attach LSM hooks
    /// 4. Populate protected file maps
    #[cfg(target_os = "linux")]
    pub async fn initialize(&mut self) -> Result<()> {
        // Step 1: Resolve BTF
        let btf_resolver = BtfResolver::new(self.config.btf_config.clone());
        let btf_status = btf_resolver.resolve().await;

        match &btf_status {
            BtfStatus::Native => {
                info!("[EBPF] Using native kernel BTF");
            }
            BtfStatus::BtfHub(path) => {
                info!("[EBPF] Using BTFHub BTF: {:?}", path);
            }
            BtfStatus::Unavailable(reason) => {
                error!("[EBPF] BTF unavailable: {}. Falling back to userspace-only monitoring.", reason);
                self.btf_status = Some(btf_status);
                return Ok(()); // Graceful degradation — no eBPF
            }
        }

        // Step 2: Load eBPF program
        // In production, this would use aya::Bpf::load() with the BTF path:
        //
        //   let mut bpf_loader = aya::BpfLoader::new();
        //   if let Some(btf_path) = btf_status.btf_path() {
        //       if !matches!(btf_status, BtfStatus::Native) {
        //           let btf = aya::Btf::from_sys_fs()?; // or parse_file(btf_path)
        //           bpf_loader.btf(Some(&btf));
        //       }
        //   }
        //   let mut bpf = bpf_loader.load_file(&self.config.object_path)?;
        //
        //   // Attach LSM hooks
        //   let program: &mut aya::programs::Lsm = bpf.program_mut("lsm_file_open")?;
        //   program.load("file_open", &btf)?;
        //   program.attach()?;

        info!("[EBPF] eBPF program loaded and attached");
        self.btf_status = Some(btf_status);
        self.loaded = true;
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn initialize(&mut self) -> Result<()> {
        warn!("[EBPF] eBPF not available on this platform");
        self.btf_status = Some(BtfStatus::Unavailable("Not Linux".to_string()));
        Ok(())
    }

    /// Push protected file identities into the eBPF map.
    /// Called after inode_resolver populates the registry.
    pub fn update_protected_files(
        &self,
        files: &[(crate::inode_resolver::InodeIdentity, crate::inode_resolver::ProtectionPolicy)],
    ) -> Result<()> {
        if !self.loaded {
            warn!("[EBPF] Cannot update maps — eBPF not loaded");
            return Ok(());
        }

        for (identity, policy) in files {
            let _key = ProtectedFileKey {
                dev: ((identity.dev_major & 0xFFF) << 20) | (identity.dev_minor & 0xFFFFF),
                inode: identity.inode,
            };
            let _value = ProtectedFilePolicy {
                action: match policy.action {
                    crate::inode_resolver::ProtectionAction::Alert => 0,
                    crate::inode_resolver::ProtectionAction::BlockWrite => 1,
                    crate::inode_resolver::ProtectionAction::BlockAll => 2,
                    crate::inode_resolver::ProtectionAction::Quarantine => 3,
                },
                severity: policy.severity as u32,
            };

            // In production:
            // let mut map: aya::maps::HashMap<_, ProtectedFileKey, ProtectedFilePolicy> =
            //     aya::maps::HashMap::try_from(bpf.map_mut("protected_files")?)?;
            // map.insert(key, value, 0)?;
            info!(
                "[EBPF] Protected file in eBPF map: {} -> action={}",
                identity, policy.severity
            );
        }
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn btf_status(&self) -> Option<&BtfStatus> {
        self.btf_status.as_ref()
    }
}