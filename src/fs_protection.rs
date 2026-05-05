#![allow(dead_code)]
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Critical system paths that should be protected from unauthorized writes
pub struct FsProtection {
    critical_paths: HashSet<String>,
    last_check: SystemTime,
}

impl Default for FsProtection {
    fn default() -> Self {
        Self::new()
    }
}

impl FsProtection {
    pub fn new() -> Self {
        let mut critical_paths = HashSet::new();

        // System configuration
        critical_paths.insert("/etc/passwd".to_string());
        critical_paths.insert("/etc/shadow".to_string());
        critical_paths.insert("/etc/sudoers".to_string());
        critical_paths.insert("/etc/ssh/sshd_config".to_string());

        // Boot files
        critical_paths.insert("/boot/vmlinuz".to_string());
        critical_paths.insert("/boot/initrd.img".to_string());

        // Critical binaries
        critical_paths.insert("/usr/bin/sudo".to_string());
        critical_paths.insert("/usr/bin/su".to_string());
        critical_paths.insert("/bin/login".to_string());

        let mut fs = Self { 
            critical_paths,
            last_check: SystemTime::now(),
        };
        
        // Initial check
        fs.check_integrity();
        fs
    }

    pub fn is_critical(&self, path: &str) -> bool {
        self.critical_paths.contains(path)
            || path.starts_with("/boot/")
            || path.starts_with("/sys/")
            || path.starts_with("/proc/sys/kernel/")
    }

    pub fn hash_path(path: &str) -> u32 {
        let mut hash: u32 = 5381;
        for byte in path.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        hash
    }

    pub fn get_critical_paths(&self) -> &HashSet<String> {
        &self.critical_paths
    }

    pub fn check_integrity(&mut self) {
        // Validate state
        if self.critical_paths.is_empty() {
            log::warn!("FS protection not properly initialized");
            return;
        }

        let now = SystemTime::now();
        if now.duration_since(self.last_check).unwrap_or(Duration::from_secs(0)).as_secs() < 60 {
            return; // Check at most once per minute
        }
        self.last_check = now;

        let mut checked = 0;
        let mut missing = 0;

        for path in &self.critical_paths {
            if Path::new(path).exists() {
                if let Ok(metadata) = fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = now.duration_since(modified) {
                            if duration.as_secs() < 300 { // Modified in last 5 minutes
                                log::warn!("🔍 Critical file recently modified: {}", path);
                            }
                        }
                    }
                }
                checked += 1;
            } else {
                missing += 1;
            }
        }

        log::info!("🛡️  FS Protection: {} paths monitored, {} missing", checked, missing);
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_critical() {
        let fs = FsProtection::new();
        assert!(fs.is_critical("/etc/passwd"));
        assert!(fs.is_critical("/etc/shadow"));
        assert!(fs.is_critical("/boot/vmlinuz"));
        assert!(fs.is_critical("/usr/bin/sudo"));
        assert!(!fs.is_critical("/tmp/test.txt"));
    }

    #[test]
    fn test_hash_path() {
        let hash1 = FsProtection::hash_path("/etc/passwd");
        let hash2 = FsProtection::hash_path("/etc/passwd");
        let hash3 = FsProtection::hash_path("/etc/shadow");
        
        assert_eq!(hash1, hash2); // Same path = same hash
        assert_ne!(hash1, hash3); // Different paths = different hashes
    }

    #[test]
    fn test_get_critical_paths() {
        let fs = FsProtection::new();
        let paths = fs.get_critical_paths();
        assert!(paths.contains("/etc/passwd"));
        assert!(paths.contains("/etc/shadow"));
        assert!(paths.len() > 5);
    }
}
