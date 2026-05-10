// Containment: Graduated response with freeze, quarantine, network block, forensic snapshot

use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize)]
pub enum ContainmentAction {
    Kill,
    Freeze,
    NetworkQuarantine,
    CgroupIsolate,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainmentRecord {
    pub pid: u32,
    pub action: ContainmentAction,
    pub timestamp: String,
    pub reason: String,
    pub forensic_snapshot: Option<ForensicSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForensicSnapshot {
    pub pid: u32,
    pub cmdline: String,
    pub status: String,
    pub maps_summary: String,
    pub open_fds: usize,
    pub cgroup: String,
    pub captured_at: String,
}

pub struct ContainmentEngine {
    records: Vec<ContainmentRecord>,
}

impl ContainmentEngine {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn contain(&mut self, pid: u32, action: ContainmentAction, reason: &str) -> Result<()> {
        // Capture forensic snapshot BEFORE containment
        let snapshot = self.capture_forensic_snapshot(pid);

        let record = ContainmentRecord {
            pid,
            action: action.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            reason: reason.to_string(),
            forensic_snapshot: snapshot,
        };

        log::info!("🔒 Containing PID {} via {:?}: {}", pid, action, reason);

        let result = match action {
            ContainmentAction::Kill => self.kill_process(pid),
            ContainmentAction::Freeze => self.freeze_process(pid),
            ContainmentAction::NetworkQuarantine => self.quarantine_network(pid),
            ContainmentAction::CgroupIsolate => self.isolate_cgroup(pid),
        };

        self.records.push(record);
        result
    }

    /// Graduated response: escalate based on threat level.
    pub fn graduated_response(&mut self, pid: u32, threat_level: &str, reason: &str) -> Result<()> {
        let action = match threat_level {
            "critical" => ContainmentAction::Kill,
            "high" => ContainmentAction::CgroupIsolate,
            "medium" => ContainmentAction::Freeze,
            _ => {
                log::info!("📋 Low threat — audit only for PID {}", pid);
                return Ok(());
            }
        };
        self.contain(pid, action, reason)
    }

    fn kill_process(&self, pid: u32) -> Result<()> {
        unsafe { libc::kill(pid as i32, libc::SIGKILL); }
        log::info!("💀 Killed process {}", pid);
        Ok(())
    }

    fn freeze_process(&self, pid: u32) -> Result<()> {
        let cgroup_path = format!("/sys/fs/cgroup/freezer/nexus-quarantine-{}", pid);
        fs::create_dir_all(&cgroup_path)?;
        fs::write(format!("{}/cgroup.procs", cgroup_path), pid.to_string())?;
        fs::write(format!("{}/freezer.state", cgroup_path), "FROZEN")?;
        log::info!("🧊 Froze process {} in cgroup", pid);
        Ok(())
    }

    fn quarantine_network(&self, pid: u32) -> Result<()> {
        Command::new("ip")
            .args(["netns", "add", &format!("quarantine-{}", pid)])
            .output()?;
        Command::new("nsenter")
            .args(["-t", &pid.to_string(), "-n", "ip", "link", "set", "lo", "down"])
            .output()?;
        log::info!("🚫 Quarantined network for process {}", pid);
        Ok(())
    }

    fn isolate_cgroup(&self, pid: u32) -> Result<()> {
        let cgroup_path = format!("/sys/fs/cgroup/nexus-isolated-{}", pid);
        fs::create_dir_all(&cgroup_path)?;
        fs::write(format!("{}/cpu.max", cgroup_path), "10000 1000000")?; // 1% CPU
        fs::write(format!("{}/memory.max", cgroup_path), "10485760")?;   // 10MB
        fs::write(format!("{}/cgroup.procs", cgroup_path), pid.to_string())?;
        log::info!("🔒 Isolated process {} in restricted cgroup", pid);
        Ok(())
    }

    fn capture_forensic_snapshot(&self, pid: u32) -> Option<ForensicSnapshot> {
        let proc_path = format!("/proc/{}", pid);
        if !std::path::Path::new(&proc_path).exists() { return None; }

        let cmdline = fs::read_to_string(format!("{}/cmdline", proc_path))
            .unwrap_or_default()
            .replace('\0', " ");
        let status = fs::read_to_string(format!("{}/status", proc_path))
            .unwrap_or_default()
            .lines().take(10).collect::<Vec<_>>().join("\n");
        let maps = fs::read_to_string(format!("{}/maps", proc_path))
            .map(|m| format!("{} regions", m.lines().count()))
            .unwrap_or_else(|_| "unavailable".into());
        let fds = fs::read_dir(format!("{}/fd", proc_path))
            .map(|d| d.count()).unwrap_or(0);
        let cgroup = fs::read_to_string(format!("{}/cgroup", proc_path))
            .unwrap_or_default().trim().to_string();

        Some(ForensicSnapshot {
            pid,
            cmdline,
            status,
            maps_summary: maps,
            open_fds: fds,
            cgroup,
            captured_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn release(&self, pid: u32) -> Result<()> {
        let cgroup_path = format!("/sys/fs/cgroup/freezer/nexus-quarantine-{}", pid);
        if std::path::Path::new(&cgroup_path).exists() {
            fs::write(format!("{}/freezer.state", cgroup_path), "THAWED")?;
            fs::remove_dir(&cgroup_path)?;
        }

        let iso_path = format!("/sys/fs/cgroup/nexus-isolated-{}", pid);
        if std::path::Path::new(&iso_path).exists() {
            fs::remove_dir(&iso_path)?;
        }

        log::info!("✅ Released process {} from containment", pid);
        Ok(())
    }

    pub fn get_audit_trail(&self) -> &[ContainmentRecord] {
        &self.records
    }
}

impl Default for ContainmentEngine { fn default() -> Self { Self::new() } }
