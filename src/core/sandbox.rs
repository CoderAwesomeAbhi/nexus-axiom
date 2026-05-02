#![allow(dead_code)]

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enable_static_analysis: bool,
    pub enable_dynamic_analysis: bool,
    pub anti_vm_detection: bool,
    pub timeout_seconds: u64,
    pub max_memory_mb: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_static_analysis: true,
            enable_dynamic_analysis: true,
            anti_vm_detection: true,
            timeout_seconds: 30,
            max_memory_mb: 512,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sandbox {
    config: SandboxConfig,
    known_hypervisors: HashSet<String>,
    anti_vm_strings: HashSet<String>,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        let mut known_hypervisors = HashSet::new();
        known_hypervisors.insert("VMware".to_string());
        known_hypervisors.insert("VirtualBox".to_string());
        known_hypervisors.insert("QEMU".to_string());
        known_hypervisors.insert("KVM".to_string());
        known_hypervisors.insert("Xen".to_string());
        known_hypervisors.insert("Hyper-V".to_string());
        known_hypervisors.insert("Parallels".to_string());

        let mut anti_vm_strings = HashSet::new();
        anti_vm_strings.insert("GetSystemInfo".to_string());
        anti_vm_strings.insert("cpuid".to_string());
        anti_vm_strings.insert("rdtsc".to_string());
        anti_vm_strings.insert("smbios".to_string());
        anti_vm_strings.insert("dmidecode".to_string());

        Self {
            config,
            known_hypervisors,
            anti_vm_strings,
        }
    }

    pub fn check_environment(&self) -> SandboxResult {
        let mut indicators = Vec::new();
        let mut is_vm = false;

        if self.config.anti_vm_detection {
            if let Ok(mac) = self.get_mac_address() {
                if self.is_vm_mac(&mac) {
                    indicators.push(format!("VM MAC address detected: {}", mac));
                    is_vm = true;
                }
            }

            if let Ok(cpuinfo) = self.get_cpu_info() {
                if self.contains_vm_signature(&cpuinfo) {
                    indicators.push("CPU info contains VM signature".to_string());
                    is_vm = true;
                }
            }

            if self.is_hypervisor_present() {
                indicators.push("Hypervisor bit is set in CPU".to_string());
                is_vm = true;
            }

            if self.check_vm_files() {
                indicators.push("VM device files detected".to_string());
                is_vm = true;
            }
        }

        SandboxResult {
            is_vm_environment: is_vm,
            indicators,
        }
    }

    fn get_mac_address(&self) -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("cat")
                .arg("/sys/class/net/eth0/address")
                .output()?;
            
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
        
        Ok(String::new())
    }

    fn is_vm_mac(&self, mac: &str) -> bool {
        let vm_prefixes = [
            "00:05:69", // VMware
            "00:0c:29", // VMware
            "00:1c:14", // VMware
            "00:50:56", // VMware
            "08:00:27", // VirtualBox
            "0c:c4:7a", // KVM/QEMU
            "52:54:00", // QEMU
            "00:16:3e", // Xen
            "ac:de:48", // Parallels
        ];

        let mac_upper = mac.to_uppercase();
        for prefix in vm_prefixes {
            if mac_upper.starts_with(&prefix.to_uppercase()) {
                return true;
            }
        }
        false
    }

    fn get_cpu_info(&self) -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("cat").arg("/proc/cpuinfo").output()?;
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            let output = Command::new("system_profiler").arg("-detailLevel").arg("full").output()?;
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        Ok(String::new())
    }

    fn contains_vm_signature(&self, cpuinfo: &str) -> bool {
        let vm_signatures = [
            "VMware",
            "VirtualBox", 
            "QEMU",
            "KVM",
            "Xen",
            "Hyper-V",
            "Hygon",
        ];

        let cpuinfo_upper = cpuinfo.to_uppercase();
        for sig in vm_signatures {
            if cpuinfo_upper.contains(&sig.to_uppercase()) {
                return true;
            }
        }
        false
    }

    fn is_hypervisor_present(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("grep")
                .arg("-o")
                .arg("1")
                .arg("/proc/cpuinfo")
                .output();
            
            if let Ok(output) = output {
                let result = String::from_utf8_lossy(&output.stdout);
                if result.contains("1") {
                    return true;
                }
            }
        }
        
        false
    }

    fn check_vm_files(&self) -> bool {
        let vm_device_paths = [
            "/dev/vboxguest",
            "/dev/vboxvfs", 
            "/dev/vboxfs",
            "/dev/qemu",
            "/dev/kvm",
            "/dev/xen blkfront",
            "/dev/xen netfront",
        ];

        for path in vm_device_paths {
            if Path::new(path).exists() {
                return true;
            }
        }

        false
    }

    pub fn execute_sandbox(&self, binary_path: &Path) -> Result<SandboxExecution> {
        info!("[SANDBOX] Executing: {:?}", binary_path);

        let env_result = self.check_environment();
        if env_result.is_vm_environment {
            warn!(
                "[SANDBOX] WARNING: Running inside VM! Indicators: {:?}",
                env_result.indicators
            );
        }

        let mut execution = SandboxExecution {
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            signals: Vec::new(),
            vm_detected: env_result.is_vm_environment,
            vm_indicators: env_result.indicators,
            timed_out: false,
        };

        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new(binary_path);
            cmd.arg(format!("--sandbox-timeout={}", self.config.timeout_seconds));
            cmd.arg(format!("--sandbox-memory-limit={}", self.config.max_memory_mb));
            cmd.arg("--sandbox-disable-network");
            
            match cmd.output() {
                Ok(output) => {
                    execution.exit_code = output.status.code();
                    execution.stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    execution.stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    
                    if self.is_malware_behavior(&execution.stdout, &execution.stderr) {
                        execution.signals.push("MALICIOUS_BEHAVIOR".to_string());
                    }
                }
                Err(e) => {
                    execution.signals.push(format!("EXECUTION_ERROR: {}", e));
                }
            }
        }

        Ok(execution)
    }

    fn is_malware_behavior(&self, stdout: &str, stderr: &str) -> bool {
        let dangerous_signals = [
            "trying to mount",
            "attempting to bind",
            "trying to escalate",
            "injecting into",
            "packet flood",
            "port scanning",
        ];

        let combined = format!("{} {}", stdout, stderr).to_lowercase();
        for signal in dangerous_signals {
            if combined.contains(&signal.to_lowercase()) {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub is_vm_environment: bool,
    pub indicators: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SandboxExecution {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub signals: Vec<String>,
    pub vm_detected: bool,
    pub vm_indicators: Vec<String>,
    pub timed_out: bool,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new(SandboxConfig::default())
    }
}