// Seccomp User Notification Mode: Unprivileged protection

use anyhow::Result;
use std::os::unix::io::RawFd;

pub struct SeccompMode {
    notify_fd: Option<RawFd>,
}

impl SeccompMode {
    pub fn new() -> Self {
        Self { notify_fd: None }
    }

    pub fn start_listener(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            // Create Unix socket for client connections
            let listener = std::os::unix::net::UnixListener::bind("/tmp/nexus-axiom.sock")?;

            log::info!("🔓 Unprivileged mode: listening on /tmp/nexus-axiom.sock");
            log::info!("   Apps can connect with: LD_PRELOAD=libnexus-client.so ./app");

            // Accept client connections
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        log::info!("✅ Client connected");
                        self.handle_client(stream)?;
                    }
                    Err(e) => {
                        log::error!("Connection error: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_client(&self, mut stream: std::os::unix::net::UnixStream) -> Result<()> {
        use std::io::{Read, Write};

        loop {
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf)?;

            if n == 0 {
                break; // Client disconnected
            }

            // Parse seccomp notification
            let notification = self.parse_notification(&buf[..n])?;

            // Evaluate policy
            let decision = self.evaluate(&notification);

            // Send response
            let response = if decision {
                b"ALLOW"
            } else {
                log::warn!("🚫 Blocked W^X attempt from client");
                b"DENY"
            };

            stream.write_all(response)?;
        }

        Ok(())
    }

    fn parse_notification(&self, data: &[u8]) -> Result<SeccompNotification> {
        // Parse notification from client
        // Format: syscall_nr:arg0:arg1:arg2
        let s = std::str::from_utf8(data)?;
        let parts: Vec<&str> = s.trim().split(':').collect();

        if parts.len() < 4 {
            anyhow::bail!("Invalid notification format");
        }

        Ok(SeccompNotification {
            syscall: parts[0].parse()?,
            arg0: u64::from_str_radix(parts[1], 16)?,
            arg1: u64::from_str_radix(parts[2], 16)?,
            arg2: u64::from_str_radix(parts[3], 16)?,
        })
    }

    fn evaluate(&self, notif: &SeccompNotification) -> bool {
        // Check if it's mprotect with W^X
        if notif.syscall == 10 {
            // mprotect
            let prot = notif.arg1 as i32;
            let is_wx = (prot & 0x1) != 0 && (prot & 0x4) != 0; // PROT_READ | PROT_EXEC

            if is_wx {
                return false; // DENY
            }
        }

        true // ALLOW
    }
}

#[derive(Debug)]
struct SeccompNotification {
    syscall: u64,
    arg0: u64,
    arg1: u64,
    arg2: u64,
}
