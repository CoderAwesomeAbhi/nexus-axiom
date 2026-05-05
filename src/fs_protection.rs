use anyhow::Result;
use inotify::{EventMask, Inotify, WatchMask};
use log;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// FsProtection monitors critical system files for unauthorized modifications
/// using inotify for real-time detection
pub struct FsProtection {
    critical_paths: Vec<String>,
    running: Arc<AtomicBool>,
}

impl FsProtection {
    pub fn new() -> Self {
        let mut critical_paths = Vec::new();

        // Critical binaries
        critical_paths.push("/usr/bin/sudo".to_string());
        critical_paths.push("/usr/bin/su".to_string());
        critical_paths.push("/bin/login".to_string());
        critical_paths.push("/usr/sbin/sshd".to_string());

        // Critical configs
        critical_paths.push("/etc/passwd".to_string());
        critical_paths.push("/etc/shadow".to_string());
        critical_paths.push("/etc/sudoers".to_string());
        critical_paths.push("/etc/ssh/sshd_config".to_string());

        // System libraries
        critical_paths.push("/lib/x86_64-linux-gnu/libc.so.6".to_string());
        critical_paths.push("/lib/x86_64-linux-gnu/libpam.so.0".to_string());

        Self {
            critical_paths,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start real-time monitoring with inotify
    pub fn start_monitoring(&mut self) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);

        let mut inotify = Inotify::init()?;
        let mut watch_descriptors = HashMap::new();

        // Add watches for all critical paths
        for path in &self.critical_paths {
            if Path::new(path).exists() {
                match inotify.watches().add(
                    path,
                    WatchMask::MODIFY | WatchMask::ATTRIB | WatchMask::DELETE_SELF,
                ) {
                    Ok(wd) => {
                        watch_descriptors.insert(wd, path.clone());
                        log::info!("🛡️  Monitoring: {}", path);
                    }
                    Err(e) => {
                        log::warn!("⚠️  Failed to watch {}: {}", path, e);
                    }
                }
            } else {
                log::warn!("⚠️  Critical file missing: {}", path);
            }
        }

        log::info!(
            "✅ FS Protection: Monitoring {} critical paths in real-time",
            watch_descriptors.len()
        );

        // Spawn monitoring thread
        let running = self.running.clone();
        thread::spawn(move || {
            let mut buffer = [0; 4096];

            while running.load(Ordering::SeqCst) {
                match inotify.read_events(&mut buffer) {
                    Ok(events) => {
                        for event in events {
                            if let Some(path) = watch_descriptors.get(&event.wd) {
                                if event.mask.contains(EventMask::MODIFY) {
                                    log::warn!("🚨 CRITICAL FILE MODIFIED: {}", path);
                                    println!("🚨 ALERT: Critical file modified: {}", path);
                                }
                                if event.mask.contains(EventMask::ATTRIB) {
                                    log::warn!("🚨 CRITICAL FILE ATTRIBUTES CHANGED: {}", path);
                                    println!("🚨 ALERT: File attributes changed: {}", path);
                                }
                                if event.mask.contains(EventMask::DELETE_SELF) {
                                    log::error!("🚨 CRITICAL FILE DELETED: {}", path);
                                    println!("🚨 ALERT: Critical file deleted: {}", path);
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No events, sleep briefly
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        log::error!("❌ inotify error: {}", e);
                        break;
                    }
                }
            }

            log::info!("FS Protection monitoring stopped");
        });

        Ok(())
    }

    /// Stop monitoring
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Legacy check_integrity for compatibility (now just validates paths exist)
    pub fn check_integrity(&self) {
        let mut checked = 0;
        let mut missing = 0;

        for path in &self.critical_paths {
            checked += 1;
            if !Path::new(path).exists() {
                missing += 1;
                log::warn!("⚠️  Critical file missing: {}", path);
            }
        }

        log::debug!(
            "🛡️  FS Protection: {} paths checked, {} missing",
            checked,
            missing
        );
    }
}

impl Default for FsProtection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_protection_creation() {
        let fs = FsProtection::new();
        assert!(!fs.critical_paths.is_empty());
        assert!(fs.critical_paths.contains(&"/etc/passwd".to_string()));
    }

    #[test]
    fn test_check_integrity() {
        let fs = FsProtection::new();
        fs.check_integrity(); // Should not panic
    }
}
