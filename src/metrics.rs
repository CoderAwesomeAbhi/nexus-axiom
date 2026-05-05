use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

pub struct MetricsServer {
    pub blocked_events: Arc<AtomicU64>,
    pub total_events: Arc<AtomicU64>,
    pub mmap_events: Arc<AtomicU64>,
    pub mprotect_events: Arc<AtomicU64>,
    pub ptrace_events: Arc<AtomicU64>,
    pub exec_events: Arc<AtomicU64>,
    pub file_events: Arc<AtomicU64>,
    pub network_drops: Arc<AtomicU64>,
    pub dropped_events: Arc<AtomicU64>,
    pub start_time: Instant,
}

impl Default for MetricsServer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsServer {
    pub fn new() -> Self {
        Self {
            blocked_events: Arc::new(AtomicU64::new(0)),
            total_events: Arc::new(AtomicU64::new(0)),
            mmap_events: Arc::new(AtomicU64::new(0)),
            mprotect_events: Arc::new(AtomicU64::new(0)),
            ptrace_events: Arc::new(AtomicU64::new(0)),
            exec_events: Arc::new(AtomicU64::new(0)),
            file_events: Arc::new(AtomicU64::new(0)),
            network_drops: Arc::new(AtomicU64::new(0)),
            dropped_events: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }

    pub fn start(&self, port: u16) -> Result<(), String> {
        let blocked = self.blocked_events.clone();
        let total = self.total_events.clone();
        let mmap = self.mmap_events.clone();
        let mprotect = self.mprotect_events.clone();
        let ptrace = self.ptrace_events.clone();
        let exec = self.exec_events.clone();
        let file = self.file_events.clone();
        let network = self.network_drops.clone();
        let dropped = self.dropped_events.clone();
        let start = self.start_time;

        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(l) => l,
            Err(e) => {
                log::error!("❌ Failed to bind metrics server on port {}: {}", port, e);
                return Err(format!("Port {} already in use", port));
            }
        };

        log::info!(
            "📈 Prometheus Metrics endpoint listening on http://0.0.0.0:{}/metrics",
            port
        );

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = [0; 1024];
                        if let Ok(n) = stream.read(&mut buffer) {
                            // Basic validation: check for GET /metrics
                            let request = String::from_utf8_lossy(&buffer[..n]);
                            if request.starts_with("GET") && request.contains("/metrics") {
                                let uptime = start.elapsed().as_secs();
                                let response = format!(
                                    "HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain; version=0.0.4\r\n\
                                    \r\n\
                                    # HELP nexus_axiom_events_total Total number of eBPF events processed\n\
                                    # TYPE nexus_axiom_events_total counter\n\
                                    nexus_axiom_events_total {}\n\
                                    \n\
                                    # HELP nexus_axiom_blocked_total Total number of exploits blocked\n\
                                    # TYPE nexus_axiom_blocked_total counter\n\
                                    nexus_axiom_blocked_total {}\n\
                                    \n\
                                    # HELP nexus_axiom_mmap_events W^X mmap events detected\n\
                                    # TYPE nexus_axiom_mmap_events counter\n\
                                    nexus_axiom_mmap_events {}\n\
                                    \n\
                                    # HELP nexus_axiom_mprotect_events W^X mprotect events detected\n\
                                    # TYPE nexus_axiom_mprotect_events counter\n\
                                    nexus_axiom_mprotect_events {}\n\
                                    \n\
                                    # HELP nexus_axiom_ptrace_events Ptrace debugging attempts\n\
                                    # TYPE nexus_axiom_ptrace_events counter\n\
                                    nexus_axiom_ptrace_events {}\n\
                                    \n\
                                    # HELP nexus_axiom_exec_events Execution control events\n\
                                    # TYPE nexus_axiom_exec_events counter\n\
                                    nexus_axiom_exec_events {}\n\
                                    \n\
                                    # HELP nexus_axiom_file_events File access events\n\
                                    # TYPE nexus_axiom_file_events counter\n\
                                    nexus_axiom_file_events {}\n\
                                    \n\
                                    # HELP nexus_axiom_network_drops Network packets dropped\n\
                                    # TYPE nexus_axiom_network_drops counter\n\
                                    nexus_axiom_network_drops {}\n\
                                    \n\
                                    # HELP nexus_axiom_dropped_events Events dropped due to rate limiting\n\
                                    # TYPE nexus_axiom_dropped_events counter\n\
                                    nexus_axiom_dropped_events {}\n\
                                    \n\
                                    # HELP nexus_axiom_uptime_seconds Uptime in seconds\n\
                                    # TYPE nexus_axiom_uptime_seconds gauge\n\
                                    nexus_axiom_uptime_seconds {}\n",
                                    total.load(Ordering::Relaxed),
                                    blocked.load(Ordering::Relaxed),
                                    mmap.load(Ordering::Relaxed),
                                    mprotect.load(Ordering::Relaxed),
                                    ptrace.load(Ordering::Relaxed),
                                    exec.load(Ordering::Relaxed),
                                    file.load(Ordering::Relaxed),
                                    network.load(Ordering::Relaxed),
                                    dropped.load(Ordering::Relaxed),
                                    uptime
                                );
                                let _ = stream.write_all(response.as_bytes());
                            } else {
                                // Reject invalid requests
                                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                                let _ = stream.write_all(response.as_bytes());
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        let metrics = MetricsServer::new();
        assert_eq!(metrics.blocked_events.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.total_events.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_metrics_increment() {
        let metrics = MetricsServer::new();
        metrics.blocked_events.fetch_add(1, Ordering::Relaxed);
        metrics.total_events.fetch_add(1, Ordering::Relaxed);
        assert_eq!(metrics.blocked_events.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_events.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_metrics_default() {
        let metrics = MetricsServer::default();
        assert_eq!(metrics.mmap_events.load(Ordering::Relaxed), 0);
    }
}
