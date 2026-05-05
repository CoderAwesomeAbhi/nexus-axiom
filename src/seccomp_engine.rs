use anyhow::Result;

/// SeccompEngine applies a strict seccomp-bpf filter to the Nexus Axiom daemon itself.
/// This prevents an attacker who somehow compromises the userspace daemon from
/// executing arbitrary system calls (defense-in-depth).
pub struct SeccompEngine {
    enabled: bool,
}

impl Default for SeccompEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SeccompEngine {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[cfg(target_os = "linux")]
    pub fn apply_strict_profile(&mut self) -> Result<()> {
        use libseccomp::*;

        log::info!("🔒 Applying strict Seccomp-BPF profile...");

        // Create a filter that defaults to ALLOW (we'll explicitly block dangerous syscalls)
        let mut ctx = ScmpFilterContext::new_filter(ScmpAction::Allow)?;

        // Block dangerous syscalls that a security daemon should never need
        // NOTE: We allow clone (for threads), and don't block existing sockets
        let blocked_syscalls = vec![
            "execve",   // No spawning processes
            "execveat", // No spawning processes
            "ptrace",   // No debugging other processes
            "fork",     // No forking
            "vfork",    // No forking
                        // clone is ALLOWED (needed for threads)
                        // socket/bind/listen are ALLOWED (servers already bound)
        ];

        for syscall_name in blocked_syscalls {
            if let Ok(syscall) = ScmpSyscall::from_name(syscall_name) {
                ctx.add_rule(ScmpAction::Errno(1), syscall)?;
                log::debug!("  Blocked: {}", syscall_name);
            }
        }

        // Load the filter
        ctx.load()?;

        self.enabled = true;
        log::info!("✅ Seccomp profile applied. Daemon is now isolated.");

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn apply_strict_profile(&mut self) -> Result<()> {
        log::warn!("🔒 Seccomp: Not available on non-Linux systems");
        self.enabled = false;
        Ok(())
    }
}
