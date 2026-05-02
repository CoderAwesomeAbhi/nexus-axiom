use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

pub struct MetricsServer {
    pub blocked_events: Arc<AtomicU64>,
    pub total_events: Arc<AtomicU64>,
}

impl MetricsServer {
    pub fn new() -> Self {
        Self {
            blocked_events: Arc::new(AtomicU64::new(0)),
            total_events: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn start(&self, port: u16) {
        let blocked = self.blocked_events.clone();
        let total = self.total_events.clone();
        
        thread::spawn(move || {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
                .expect("Failed to bind metrics server");
            
            log::info!("📈 Prometheus Metrics endpoint listening on http://0.0.0.0:{}/metrics", port);
            
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = [0; 1024];
                        if let Ok(_) = stream.read(&mut buffer) {
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
                                nexus_axiom_blocked_total {}\n",
                                total.load(Ordering::Relaxed),
                                blocked.load(Ordering::Relaxed)
                            );
                            let _ = stream.write_all(response.as_bytes());
                        }
                    }
                    Err(_) => continue,
                }
            }
        });
    }
}
