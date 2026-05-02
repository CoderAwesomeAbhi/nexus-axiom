#![allow(dead_code)]

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub struct StaticAnalyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisResult {
    pub is_malicious: bool,
    pub confidence: f32,
    pub indicators: Vec<String>,
}

impl StaticAnalyzer {
    pub fn analyze(path: &PathBuf) -> StaticAnalysisResult {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                return StaticAnalysisResult {
                    is_malicious: false,
                    confidence: 0.0,
                    indicators: vec![format!("Failed to read file: {}", e)],
                };
            }
        };

        let mut indicators = Vec::new();

        if Self::has_anti_vm_bytes(&data) {
            indicators.push("Anti-VM CPUID instructions detected".to_string());
        }

        if Self::has_suspicious_strings(&data) {
            indicators.push("VM/Hypervisor detection strings found".to_string());
        }

        if Self::has_dangerous_patterns(&data) {
            return StaticAnalysisResult {
                is_malicious: true,
                confidence: 0.99,
                indicators: vec!["Known malicious patterns detected".to_string()],
            };
        }

        StaticAnalysisResult {
            is_malicious: false,
            confidence: if indicators.is_empty() { 0.9 } else { 0.6 },
            indicators,
        }
    }

    fn has_anti_vm_bytes(data: &[u8]) -> bool {
        let patterns = [
            vec![0x0F, 0x01, 0xD2],
            vec![0x0F, 0x01, 0xD9],
            vec![0x0F, 0x31],
        ];

        for pattern in patterns {
            if data.windows(pattern.len()).any(|w| w == pattern) {
                return true;
            }
        }

        for i in 0..data.len().saturating_sub(4) {
            if data[i] == 0x0F && data[i + 1] == 0xA2 {
                return true;
            }
        }

        false
    }

    fn has_suspicious_strings(data: &[u8]) -> bool {
        let data_str = String::from_utf8_lossy(data);
        let suspicious = ["VMware", "VirtualBox", "QEMU", "KVM", "Xen", "hyperv", "Red Hat", "Parallels", "VMX", "Hyper-V"];
        
        for s in suspicious {
            if data_str.contains(s) {
                return true;
            }
        }
        false
    }

    fn has_dangerous_patterns(data: &[u8]) -> bool {
        let data_str = String::from_utf8_lossy(data);
        let dangerous = ["chmod 777", "/etc/passwd", "/etc/shadow", "curl | bash", "wget | bash", "nc -e /bin/sh", "/bin/sh -i", "msfvenom", "meterpreter", "powershell -enc"];
        
        for p in dangerous {
            if data_str.contains(p) {
                return true;
            }
        }
        false
    }
}