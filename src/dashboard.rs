#![allow(dead_code)]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

pub struct Dashboard {
    metrics: Arc<crate::metrics::MetricsServer>,
}

impl Dashboard {
    pub fn new(metrics: Arc<crate::metrics::MetricsServer>) -> Self {
        Self { metrics }
    }

    pub fn start(&self, port: u16) -> Result<(), String> {
        let metrics = self.metrics.clone();

        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(l) => l,
            Err(e) => {
                log::error!("❌ Failed to bind dashboard server on port {}: {}", port, e);
                return Err(format!("Port {} already in use", port));
            }
        };

        log::info!("🌐 Dashboard available at http://0.0.0.0:{}", port);

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    let mut buffer = [0; 1024];
                    if let Ok(n) = stream.read(&mut buffer) {
                        // Basic validation: check for GET request
                        let request = String::from_utf8_lossy(&buffer[..n]);
                        if request.starts_with("GET") {
                            let html = Self::generate_html(&metrics);
                            let response =
                                format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", html);
                            let _ = stream.write_all(response.as_bytes());
                        } else {
                            // Reject non-GET requests
                            let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
                            let _ = stream.write_all(response.as_bytes());
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn generate_html(metrics: &Arc<crate::metrics::MetricsServer>) -> String {
        use std::sync::atomic::Ordering;

        let total = metrics.total_events.load(Ordering::Relaxed);
        let blocked = metrics.blocked_events.load(Ordering::Relaxed);
        let uptime = metrics.start_time.elapsed().as_secs();

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Nexus Axiom Dashboard</title>
    <meta http-equiv="refresh" content="5">
    <style>
        body {{ font-family: 'Segoe UI', sans-serif; background: #0a0e27; color: #fff; margin: 0; padding: 20px; }}
        .header {{ text-align: center; margin-bottom: 40px; }}
        .header h1 {{ font-size: 48px; margin: 0; background: linear-gradient(90deg, #00d4ff, #7b2ff7); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }}
        .stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; max-width: 1200px; margin: 0 auto; }}
        .card {{ background: #1a1f3a; border-radius: 12px; padding: 24px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); }}
        .card h2 {{ margin: 0 0 12px 0; font-size: 16px; color: #8b92b0; text-transform: uppercase; }}
        .card .value {{ font-size: 48px; font-weight: bold; margin: 0; }}
        .card.blocked .value {{ color: #ff4757; }}
        .card.total .value {{ color: #00d4ff; }}
        .card.uptime .value {{ color: #7b2ff7; font-size: 32px; }}
        .status {{ text-align: center; margin-top: 40px; padding: 20px; background: #1a1f3a; border-radius: 12px; }}
        .status .indicator {{ display: inline-block; width: 12px; height: 12px; background: #2ecc71; border-radius: 50%; margin-right: 8px; animation: pulse 2s infinite; }}
        @keyframes pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.5; }} }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🛡️ Nexus Axiom</h1>
        <p>Real-time Security Monitoring</p>
    </div>
    
    <div class="stats">
        <div class="card blocked">
            <h2>Exploits Blocked</h2>
            <p class="value">{}</p>
        </div>
        
        <div class="card total">
            <h2>Total Events</h2>
            <p class="value">{}</p>
        </div>
        
        <div class="card uptime">
            <h2>Uptime</h2>
            <p class="value">{}h {}m</p>
        </div>
    </div>
    
    <div class="status">
        <span class="indicator"></span>
        <strong>ACTIVE</strong> - eBPF LSM hooks loaded and monitoring
    </div>
</body>
</html>"#,
            blocked,
            total,
            uptime / 3600,
            (uptime % 3600) / 60
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_dashboard_html_generation() {
        let metrics = Arc::new(crate::metrics::MetricsServer::new());
        metrics.blocked_events.store(5, std::sync::atomic::Ordering::Relaxed);
        metrics.total_events.store(10, std::sync::atomic::Ordering::Relaxed);
        
        let html = Dashboard::generate_html(&metrics);
        assert!(html.contains("Nexus Axiom"));
        assert!(html.contains("<p class=\"value\">5</p>"));
        assert!(html.contains("<p class=\"value\">10</p>"));
    }

    #[test]
    fn test_dashboard_creation() {
        let metrics = Arc::new(crate::metrics::MetricsServer::new());
        let dashboard = Dashboard::new(metrics);
        assert!(dashboard.metrics.blocked_events.load(std::sync::atomic::Ordering::Relaxed) == 0);
    }
}

