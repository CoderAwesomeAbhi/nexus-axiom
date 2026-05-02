#![allow(dead_code)]

//! Inode Resolver — Unique file identity via (dev_t + ino_t)
//!
//! Inodes are only unique within a single filesystem. To prevent cross-filesystem
//! collisions, every file identity check MUST use device ID + inode number.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
#[cfg(target_os = "linux")]
use anyhow::Context;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// ============================================================================
// INODE IDENTITY — THE ONLY CORRECT WAY TO IDENTIFY A FILE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InodeIdentity {
    pub dev_major: u32,
    pub dev_minor: u32,
    pub inode: u64,
}

impl InodeIdentity {
    pub fn new(dev_major: u32, dev_minor: u32, inode: u64) -> Self {
        Self { dev_major, dev_minor, inode }
    }

    #[cfg(target_os = "linux")]
    pub fn from_raw_dev(dev: u64, inode: u64) -> Self {
        let major = unsafe { libc::major(dev) } as u32;
        let minor = unsafe { libc::minor(dev) } as u32;
        Self { dev_major: major, dev_minor: minor, inode }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn from_raw_dev(dev: u64, inode: u64) -> Self {
        Self { dev_major: dev as u32, dev_minor: 0, inode }
    }

    pub fn matches(&self, other: &InodeIdentity) -> bool {
        self.dev_major == other.dev_major
            && self.dev_minor == other.dev_minor
            && self.inode == other.inode
    }

    pub fn combined_dev(&self) -> u64 {
        ((self.dev_major as u64) << 32) | (self.dev_minor as u64)
    }
}

impl fmt::Display for InodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dev={}:{} ino={}", self.dev_major, self.dev_minor, self.inode)
    }
}

// ============================================================================
// FILE IDENTITY RESOLUTION
// ============================================================================

#[cfg(target_os = "linux")]
pub fn resolve_file_identity(path: &str) -> Result<InodeIdentity> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path = CString::new(path)
        .with_context(|| format!("Invalid path for stat: {}", path))?;
    let mut stat_buf = MaybeUninit::<libc::stat>::uninit();
    let ret = unsafe { libc::stat(c_path.as_ptr(), stat_buf.as_mut_ptr()) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        anyhow::bail!("stat({}) failed: {}", path, err);
    }
    let stat_buf = unsafe { stat_buf.assume_init() };
    Ok(InodeIdentity::from_raw_dev(stat_buf.st_dev as u64, stat_buf.st_ino as u64))
}

#[cfg(not(target_os = "linux"))]
pub fn resolve_file_identity(path: &str) -> Result<InodeIdentity> {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(path.as_bytes());
    let inode = u64::from_le_bytes(hash[0..8].try_into().unwrap());
    Ok(InodeIdentity::new(0, 0, inode))
}

// ============================================================================
// PROTECTION POLICY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtectionAction {
    BlockWrite,
    BlockAll,
    Quarantine,
    Alert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionPolicy {
    pub action: ProtectionAction,
    pub description: String,
    pub severity: u8,
}

// ============================================================================
// PROTECTED FILE REGISTRY
// ============================================================================

#[derive(Debug, Clone)]
pub struct ProtectedFileRegistry {
    entries: Arc<RwLock<HashMap<InodeIdentity, (PathBuf, ProtectionPolicy)>>>,
}

impl ProtectedFileRegistry {
    pub fn new() -> Self {
        Self { entries: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn register_defaults(&self) -> Result<()> {
        let critical_paths = vec![
            ("/etc/passwd", ProtectionAction::BlockWrite, "System user database", 10),
            ("/etc/shadow", ProtectionAction::BlockWrite, "Password hashes", 10),
            ("/etc/group", ProtectionAction::BlockWrite, "System group database", 9),
            ("/etc/gshadow", ProtectionAction::BlockWrite, "Group password hashes", 9),
            ("/etc/sudoers", ProtectionAction::BlockWrite, "Sudo configuration", 10),
            ("/etc/ssh/sshd_config", ProtectionAction::BlockWrite, "SSH daemon config", 10),
            ("/etc/pam.d/common-auth", ProtectionAction::BlockWrite, "PAM auth config", 10),
            ("/etc/ld.so.preload", ProtectionAction::BlockAll, "Rootkit preload vector", 10),
            ("/etc/crontab", ProtectionAction::Quarantine, "System cron table", 8),
            ("/boot/vmlinuz", ProtectionAction::BlockAll, "Kernel image", 10),
            ("/boot/initrd.img", ProtectionAction::BlockAll, "Initial ramdisk", 10),
            ("/boot/grub/grub.cfg", ProtectionAction::BlockWrite, "Bootloader config", 10),
        ];

        let mut entries = self.entries.write().await;
        for (path, action, desc, severity) in critical_paths {
            match resolve_file_identity(path) {
                Ok(identity) => {
                    let policy = ProtectionPolicy {
                        action, description: desc.to_string(), severity,
                    };
                    info!("[INODE] Protected: {} -> {} ({})", path, identity, desc);
                    entries.insert(identity, (PathBuf::from(path), policy));
                }
                Err(e) => {
                    warn!("[INODE] Skipping {}: {}", path, e);
                }
            }
        }
        info!("[INODE] Registry initialized with {} entries", entries.len());
        Ok(())
    }

    pub async fn check(&self, identity: &InodeIdentity) -> Option<(PathBuf, ProtectionPolicy)> {
        self.entries.read().await.get(identity).cloned()
    }

    pub async fn check_path(&self, path: &str) -> Option<(PathBuf, ProtectionPolicy)> {
        match resolve_file_identity(path) {
            Ok(identity) => self.check(&identity).await,
            Err(_) => None,
        }
    }

    pub async fn refresh(&self) -> Result<()> {
        let old_entries = self.entries.read().await.clone();
        let mut new_entries = HashMap::new();
        for (_identity, (path, policy)) in old_entries {
            let path_str = path.to_string_lossy();
            match resolve_file_identity(&path_str) {
                Ok(new_id) => { new_entries.insert(new_id, (path, policy)); }
                Err(e) => { warn!("[INODE] Refresh: {} gone: {}", path_str, e); }
            }
        }
        *self.entries.write().await = new_entries;
        info!("[INODE] Registry refreshed");
        Ok(())
    }

    pub async fn get_all_identities(&self) -> Vec<(InodeIdentity, ProtectionPolicy)> {
        self.entries.read().await.iter()
            .map(|(id, (_path, policy))| (*id, policy.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_inode_different_device_must_not_match() {
        let real_passwd = InodeIdentity::new(8, 1, 10452);
        let usb_file = InodeIdentity::new(8, 17, 10452);
        assert!(!real_passwd.matches(&usb_file));
        assert_ne!(real_passwd, usb_file);
    }

    #[test]
    fn test_same_device_same_inode_matches() {
        let a = InodeIdentity::new(8, 1, 10452);
        let b = InodeIdentity::new(8, 1, 10452);
        assert!(a.matches(&b));
    }

    #[test]
    fn test_display() {
        let id = InodeIdentity::new(8, 1, 10452);
        assert_eq!(format!("{}", id), "dev=8:1 ino=10452");
    }
}
