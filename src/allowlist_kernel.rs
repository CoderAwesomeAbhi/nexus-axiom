// Allowlist kernel map integration
use anyhow::{Context, Result};
use std::os::raw::c_int;

const BPF_MAP_UPDATE_ELEM: c_int = 2;
const BPF_MAP_DELETE_ELEM: c_int = 3;
const BPF_ANY: u64 = 0;

pub struct AllowlistKernel;

impl AllowlistKernel {
    /// Add PID to kernel allowlist map via BPF syscall
    pub fn add_to_map(pid: u32) -> Result<()> {
        // Find the allowlist map FD by reading /sys/fs/bpf/
        let map_fd = Self::find_map_fd("allowlist")?;
        
        let key = pid.to_ne_bytes();
        let value: u8 = 1;
        let value_bytes = [value];
        
        let ret = unsafe {
            libc::syscall(
                libc::SYS_bpf,
                BPF_MAP_UPDATE_ELEM,
                &BpfMapUpdateAttr {
                    map_fd,
                    key: key.as_ptr() as u64,
                    value: value_bytes.as_ptr() as u64,
                    flags: BPF_ANY,
                } as *const _ as *const u8,
                std::mem::size_of::<BpfMapUpdateAttr>(),
            )
        };
        
        if ret < 0 {
            anyhow::bail!("BPF syscall failed: {}", std::io::Error::last_os_error());
        }
        
        log::info!("✅ Added PID {} to kernel allowlist map", pid);
        Ok(())
    }
    
    /// Remove PID from kernel allowlist map
    pub fn remove_from_map(pid: u32) -> Result<()> {
        let map_fd = Self::find_map_fd("allowlist")?;
        
        let key = pid.to_ne_bytes();
        
        let ret = unsafe {
            libc::syscall(
                libc::SYS_bpf,
                BPF_MAP_DELETE_ELEM,
                &BpfMapDeleteAttr {
                    map_fd,
                    key: key.as_ptr() as u64,
                } as *const _ as *const u8,
                std::mem::size_of::<BpfMapDeleteAttr>(),
            )
        };
        
        if ret < 0 {
            anyhow::bail!("BPF syscall failed: {}", std::io::Error::last_os_error());
        }
        
        log::info!("✅ Removed PID {} from kernel allowlist map", pid);
        Ok(())
    }
    
    /// Find map FD by name (simplified - in production use libbpf-rs)
    fn find_map_fd(name: &str) -> Result<c_int> {
        // Try to find pinned map in /sys/fs/bpf/
        let pin_path = format!("/sys/fs/bpf/{}", name);
        
        if std::path::Path::new(&pin_path).exists() {
            // Open pinned map
            let fd = unsafe {
                libc::syscall(
                    libc::SYS_bpf,
                    1, // BPF_OBJ_GET
                    &pin_path as *const _ as *const u8,
                    pin_path.len(),
                )
            };
            
            if fd > 0 {
                return Ok(fd as c_int);
            }
        }
        
        // Fallback: log warning and return dummy FD
        log::warn!("⚠️  Could not find allowlist map FD, allowlist updates may not work");
        log::warn!("   This is expected if eBPF is not loaded yet");
        anyhow::bail!("Map not found: {}", name)
    }
}

#[repr(C)]
struct BpfMapUpdateAttr {
    map_fd: c_int,
    key: u64,
    value: u64,
    flags: u64,
}

#[repr(C)]
struct BpfMapDeleteAttr {
    map_fd: c_int,
    key: u64,
}
