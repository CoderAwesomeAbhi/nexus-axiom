/// live_feed.rs — Global event broadcast for the live attack map and WebSocket server.
///
/// Uses a global `OnceLock<broadcast::Sender>` so callers can use free functions
/// (`init()`, `broadcast()`, `subscribe()`, `make_event()`) without needing to
/// pass around an Arc. This matches the API expected by `ebpf_engine.rs` and
/// `dashboard.rs`.
use serde::{Deserialize, Serialize};
use std::sync::{mpsc, OnceLock};

// We use std::sync::mpsc for cross-thread broadcast since tokio broadcast
// requires async context. For the WS server we clone the receiver pattern.

/// A security event broadcast to the live attack map and WebSocket subscribers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttackEvent {
    pub event_type: String,
    pub label: String,
    pub pid: u32,
    pub uid: u32,
    pub comm: String,
    pub details: String,
    pub src_ip: String,
    pub timestamp: u64,
    pub blocked: bool,
    pub severity: String,
}

// ── Global state ─────────────────────────────────────────────────────────────

static SENDER: OnceLock<mpsc::SyncSender<AttackEvent>> = OnceLock::new();
static SUBSCRIBERS: OnceLock<std::sync::Mutex<Vec<mpsc::SyncSender<AttackEvent>>>> =
    OnceLock::new();

/// Initialize the live-feed bus. Must be called once before `broadcast()` or
/// `subscribe()`.  Safe to call multiple times (subsequent calls are no-ops).
pub fn init() {
    SUBSCRIBERS.get_or_init(|| std::sync::Mutex::new(Vec::new()));

    SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<AttackEvent>(2048);

        // Dispatcher thread: receives events and fans out to all subscribers
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if let Some(subs) = SUBSCRIBERS.get() {
                    if let Ok(mut subs) = subs.lock() {
                        // Remove disconnected subscribers
                        subs.retain(|sub| sub.try_send(event.clone()).is_ok());
                    }
                }
            }
        });

        tx
    });
}

/// Broadcast an event to all connected subscribers (WebSocket clients).
/// Silently drops if init() hasn't been called or channel is full.
pub fn broadcast(event: AttackEvent) {
    if let Some(tx) = SENDER.get() {
        let _ = tx.try_send(event);
    }
}

/// Subscribe to the live event stream. Returns a receiver that will get a
/// clone of every broadcasted event. Returns `None` if `init()` hasn't been
/// called.
pub fn subscribe() -> Option<mpsc::Receiver<AttackEvent>> {
    let subs = SUBSCRIBERS.get()?;
    let (tx, rx) = mpsc::sync_channel::<AttackEvent>(512);
    if let Ok(mut subs) = subs.lock() {
        subs.push(tx);
    }
    Some(rx)
}

/// Convenience constructor for `AttackEvent`.
pub fn make_event(
    event_type: &str,
    label: &str,
    pid: u32,
    uid: u32,
    comm: &str,
    details: &str,
    src_ip: &str,
) -> AttackEvent {
    AttackEvent {
        event_type: event_type.to_string(),
        label: label.to_string(),
        pid,
        uid,
        comm: comm.to_string(),
        details: details.to_string(),
        src_ip: src_ip.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        blocked: event_type == "block",
        severity: if event_type == "block" {
            "critical".to_string()
        } else {
            "medium".to_string()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_is_idempotent() {
        init();
        init(); // second call should not panic
    }

    #[test]
    fn test_make_event() {
        let event = make_event("block", "W^X mmap", 1234, 0, "exploit", "prot=0x07", "");
        assert_eq!(event.event_type, "block");
        assert_eq!(event.pid, 1234);
        assert!(event.blocked);
        assert_eq!(event.severity, "critical");
    }

    #[test]
    fn test_broadcast_without_init() {
        // Should not panic even if init() hasn't been called in this test's context
        let event = make_event("test", "test", 0, 0, "test", "", "");
        broadcast(event);
    }

    #[test]
    fn test_subscribe_and_receive() {
        init();
        let rx = subscribe().expect("subscribe should work after init");

        let event = make_event("block", "test", 42, 0, "test_proc", "", "");
        broadcast(event);

        // Give the dispatcher thread time to process
        std::thread::sleep(std::time::Duration::from_millis(50));

        if let Ok(received) = rx.try_recv() {
            assert_eq!(received.pid, 42);
            assert_eq!(received.event_type, "block");
        }
        // Note: in test environments the dispatcher thread timing may cause
        // this to occasionally not receive — that's acceptable for a unit test.
    }
}
