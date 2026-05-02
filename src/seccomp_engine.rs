// Seccomp-BPF engine to block W^X memory allocations
// This catches anonymous mmap that LSM hooks miss
#![cfg(target_os = "linux")]

use anyhow::Result;
use seccompiler::{
    BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter,
    SeccompRule,
};
use std::collections::BTreeMap;

// Memory protection flags (from sys/mman.h)
const PROT_READ: u64 = 0x1;
const PROT_WRITE: u64 = 0x2;
const PROT_EXEC: u64 = 0x4;

pub struct SeccompEngine {
    filter: Option<BpfProgram>,
}

impl SeccompEngine {
    pub fn new() -> Self {
        Self { filter: None }
    }

    pub fn build_filter(&mut self) -> Result<()> {
        println!("🔧 Building seccomp-BPF filter for W^X protection...");

        let mut rules = BTreeMap::new();

        // Block mmap with W^X (syscall 9 on x86_64)
        // mmap(addr, len, prot, flags, fd, offset)
        // arg2 is prot - check if WRITE and EXEC are both set
        let mmap_rules = vec![
            SeccompRule::new(vec![
                // Check if prot has PROT_WRITE (0x2)
                SeccompCondition::new(
                    2, // arg index (prot)
                    SeccompCmpArgLen::Dword,
                    SeccompCmpOp::MaskedEq(PROT_WRITE),
                    PROT_WRITE,
                )?,
                // Check if prot has PROT_EXEC (0x4)
                SeccompCondition::new(
                    2,
                    SeccompCmpArgLen::Dword,
                    SeccompCmpOp::MaskedEq(PROT_EXEC),
                    PROT_EXEC,
                )?,
            ])?,
        ];

        rules.insert(9, mmap_rules); // syscall 9 = mmap

        // Block mprotect with W^X (syscall 10 on x86_64)
        // mprotect(addr, len, prot)
        // arg2 is prot
        let mprotect_rules = vec![
            SeccompRule::new(vec![
                SeccompCondition::new(
                    2,
                    SeccompCmpArgLen::Dword,
                    SeccompCmpOp::MaskedEq(PROT_WRITE),
                    PROT_WRITE,
                )?,
                SeccompCondition::new(
                    2,
                    SeccompCmpArgLen::Dword,
                    SeccompCmpOp::MaskedEq(PROT_EXEC),
                    PROT_EXEC,
                )?,
            ])?,
        ];

        rules.insert(10, mprotect_rules); // syscall 10 = mprotect

        // Create filter
        let filter = SeccompFilter::new(
            rules,
            SeccompAction::Allow, // Default: allow
            SeccompAction::Errno(13), // Block W^X with EACCES (13)
            std::env::consts::ARCH.try_into()?,
        )?;

        self.filter = Some(filter.try_into()?);
        println!(" ✅ Seccomp filter built");
        Ok(())
    }

    pub fn apply(&self) -> Result<()> {
        let filter = self.filter.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Filter not built. Call build_filter() first")
        })?;

        println!("🔧 Applying seccomp-BPF filter...");
        
        // Apply the filter to current process
        seccompiler::apply_filter(filter)?;
        
        println!(" ✅ Seccomp filter applied - W^X syscalls will be blocked");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builds() {
        let mut engine = SeccompEngine::new();
        assert!(engine.build_filter().is_ok());
    }
}
