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

    /// Start the HTTP dashboard (port) and the WebSocket live-feed server (port+1).
    pub fn start(&self, port: u16) -> Result<(), String> {
        let metrics = self.metrics.clone();

        // ── HTTP dashboard ────────────────────────────────────────────────────
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).map_err(|e| {
            log::error!("❌ Failed to bind dashboard on port {}: {}", port, e);
            format!("Port {} already in use", port)
        })?;

        log::info!("🌐 Dashboard at http://0.0.0.0:{}", port);
        log::info!("🌍 Live Attack Map at http://0.0.0.0:{}/live_map", port);

        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    let mut buf = [0u8; 2048];
                    if let Ok(n) = stream.read(&mut buf) {
                        let req = String::from_utf8_lossy(&buf[..n]);
                        if !req.starts_with("GET") {
                            let _ = stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n");
                            continue;
                        }

                        let path = req
                            .lines()
                            .next()
                            .and_then(|l| l.split_whitespace().nth(1))
                            .unwrap_or("/");

                        let (content_type, body) = match path {
                            "/live_map" | "/live_map.html" => {
                                let html = include_str!("../dashboard/live_map.html");
                                ("text/html", html.to_string())
                            }
                            "/health" => ("application/json", r#"{"status":"ok"}"#.to_string()),
                            _ => ("text/html", Self::generate_html(&metrics)),
                        };

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                            content_type,
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                }
            }
        });

        // ── WebSocket live-feed server ────────────────────────────────────────
        let ws_port = port + 1;
        start_ws_server(ws_port);

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
        .indicator {{ display: inline-block; width: 12px; height: 12px; background: #2ecc71; border-radius: 50%; margin-right: 8px; animation: pulse 2s infinite; }}
        .live-map-btn {{ display: inline-block; margin-top: 20px; padding: 12px 24px; background: linear-gradient(90deg, #00d4ff, #7b2ff7); border-radius: 8px; color: #fff; text-decoration: none; font-weight: bold; }}
        @keyframes pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.5; }} }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🛡️ Nexus Axiom</h1>
        <p>Real-time Security Monitoring</p>
        <a class="live-map-btn" href="/live_map">🌍 Open Live Attack Map</a>
    </div>
    <div class="stats">
        <div class="card blocked"><h2>Exploits Blocked</h2><p class="value">{}</p></div>
        <div class="card total"><h2>Total Events</h2><p class="value">{}</p></div>
        <div class="card uptime"><h2>Uptime</h2><p class="value">{}h {}m</p></div>
    </div>
    <div class="status">
        <span class="indicator"></span>
        <strong>ACTIVE</strong> — eBPF LSM hooks loaded and monitoring
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

// ── WebSocket server (RFC 6455, minimal handshake + framing) ─────────────────

fn start_ws_server(port: u16) {
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(l) => l,
        Err(e) => {
            log::warn!("⚠️  WebSocket server could not bind on port {}: {}", port, e);
            return;
        }
    };

    log::info!("🔌 WebSocket live-feed at ws://0.0.0.0:{}/ws", port);

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                thread::spawn(move || {
                    if let Err(e) = handle_ws_client(stream) {
                        log::debug!("WS client disconnected: {}", e);
                    }
                });
            }
        }
    });
}

fn handle_ws_client(mut stream: std::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;

    // ── HTTP upgrade handshake ────────────────────────────────────────────────
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf)?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let ws_key = request
        .lines()
        .find(|l| l.to_lowercase().starts_with("sec-websocket-key:"))
        .and_then(|l| l.splitn(2, ':').nth(1))
        .map(|s| s.trim().to_string())
        .ok_or("missing Sec-WebSocket-Key")?;

    let accept = ws_accept_key(&ws_key);
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\r\n",
        accept
    );
    stream.write_all(response.as_bytes())?;

    // ── Subscribe to live feed and forward frames ─────────────────────────────
    let rx = crate::live_feed::subscribe().ok_or("live_feed not initialised")?;

    loop {
        match rx.recv() {
            Ok(event) => {
                let json = serde_json::to_string(&event)?;
                ws_send_text(&mut stream, &json)?;
            }
            Err(_) => break, // sender dropped → server shutting down
        }
    }
    Ok(())
}

/// Compute the Sec-WebSocket-Accept header value (RFC 6455 §4.2.2).
fn ws_accept_key(key: &str) -> String {
    const MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut hasher = sha1_smol::Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(MAGIC.as_bytes());
    base64_encode(&hasher.digest().bytes())
}

/// Minimal base64 encoder (no external dep).
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };
        out.push(TABLE[((b0 >> 2) & 0x3f) as usize] as char);
        out.push(TABLE[(((b0 << 4) | (b1 >> 4)) & 0x3f) as usize] as char);
        out.push(if i + 1 < data.len() { TABLE[(((b1 << 2) | (b2 >> 6)) & 0x3f) as usize] as char } else { '=' });
        out.push(if i + 2 < data.len() { TABLE[(b2 & 0x3f) as usize] as char } else { '=' });
        i += 3;
    }
    out
}

/// Send a WebSocket text frame (RFC 6455 §5.2).
fn ws_send_text(stream: &mut std::net::TcpStream, text: &str) -> std::io::Result<()> {
    let payload = text.as_bytes();
    let len = payload.len();
    let mut frame = Vec::with_capacity(10 + len);
    frame.push(0x81); // FIN + opcode text
    if len < 126 {
        frame.push(len as u8);
    } else if len < 65536 {
        frame.push(126);
        frame.push((len >> 8) as u8);
        frame.push((len & 0xff) as u8);
    } else {
        frame.push(127);
        for i in (0..8).rev() {
            frame.push(((len >> (i * 8)) & 0xff) as u8);
        }
    }
    frame.extend_from_slice(payload);
    stream.write_all(&frame)
}

// ── SHA-1 (tiny, no external dep needed for WS handshake) ────────────────────
mod sha1_smol {
    pub struct Sha1 {
        state: [u32; 5],
        data: Vec<u8>,
    }
    pub struct Digest([u8; 20]);
    impl Digest {
        pub fn bytes(self) -> [u8; 20] { self.0 }
    }
    impl Sha1 {
        pub fn new() -> Self {
            Self {
                state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
                data: Vec::new(),
            }
        }
        pub fn update(&mut self, data: &[u8]) { self.data.extend_from_slice(data); }
        pub fn digest(mut self) -> Digest {
            let bit_len = (self.data.len() as u64) * 8;
            self.data.push(0x80);
            while self.data.len() % 64 != 56 { self.data.push(0); }
            for i in (0..8).rev() { self.data.push(((bit_len >> (i * 8)) & 0xff) as u8); }
            for chunk in self.data.chunks(64) {
                let mut w = [0u32; 80];
                for i in 0..16 {
                    w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
                }
                for i in 16..80 { w[i] = (w[i-3]^w[i-8]^w[i-14]^w[i-16]).rotate_left(1); }
                let (mut a,mut b,mut c,mut d,mut e) = (self.state[0],self.state[1],self.state[2],self.state[3],self.state[4]);
                for i in 0..80 {
                    let (f, k) = match i {
                        0..=19  => ((b&c)|((!b)&d), 0x5A827999u32),
                        20..=39 => (b^c^d,          0x6ED9EBA1),
                        40..=59 => ((b&c)|(b&d)|(c&d), 0x8F1BBCDC),
                        _       => (b^c^d,          0xCA62C1D6),
                    };
                    let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
                    e=d; d=c; c=b.rotate_left(30); b=a; a=temp;
                }
                self.state[0]=self.state[0].wrapping_add(a); self.state[1]=self.state[1].wrapping_add(b);
                self.state[2]=self.state[2].wrapping_add(c); self.state[3]=self.state[3].wrapping_add(d);
                self.state[4]=self.state[4].wrapping_add(e);
            }
            let mut out = [0u8; 20];
            for (i, &s) in self.state.iter().enumerate() {
                out[i*4..i*4+4].copy_from_slice(&s.to_be_bytes());
            }
            Digest(out)
        }
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
        assert!(html.contains(">5<"));
        assert!(html.contains(">10<"));
    }

    #[test]
    fn test_ws_accept_key() {
        // RFC 6455 §1.3 example
        let accept = ws_accept_key("dGhlIHNhbXBsZSBub25jZQ==");
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_encode(b"M"), "TQ==");
    }
}
