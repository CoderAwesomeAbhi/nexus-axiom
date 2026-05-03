use std::collections::HashSet;
use std::path::Path;

/// Critical system paths that should be protected from unauthorized writes
pub struct FsProtection {
    critical_paths: HashSet<String>,
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
        
        Self { critical_paths }
    }
    
    pub fn is_critical(&self, path: &str) -> bool {
        self.critical_paths.contains(path) || 
        path.starts_with("/boot/") ||
        path.starts_with("/sys/") ||
        path.starts_with("/proc/sys/kernel/")
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
}
