use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

pub struct Metrics {
    pub connections_attempted: AtomicU64,
    pub connections_successful: AtomicU64,
    pub connections_failed: AtomicU64,
    pub connection_time_ms: Mutex<Vec<u64>>,

    pub messages_sent: AtomicU64,
    pub send_errors: AtomicU64,

    pub location_messages_received: AtomicU64,
    pub group_messages_received: AtomicU64,

    pub latencies_ms: Mutex<Vec<u64>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            connections_attempted: AtomicU64::new(0),
            connections_successful: AtomicU64::new(0),
            connections_failed: AtomicU64::new(0),
            connection_time_ms: Mutex::new(Vec::new()),
            messages_sent: AtomicU64::new(0),
            send_errors: AtomicU64::new(0),
            location_messages_received: AtomicU64::new(0),
            group_messages_received: AtomicU64::new(0),
            latencies_ms: Mutex::new(Vec::new()),
        }
    }

    pub fn record_connection_time(&self, ms: u64) {
        self.connection_time_ms.lock().unwrap().push(ms);
    }

    pub fn record_latency(&self, ms: u64) {
        self.latencies_ms.lock().unwrap().push(ms);
    }
}

pub fn percentiles(data: &mut [u64]) -> (u64, u64, u64, u64) {
    data.sort_unstable();
    let len = data.len();
    if len == 0 {
        return (0, 0, 0, 0);
    }
    let p50 = data[len * 50 / 100];
    let p90 = data[len * 90 / 100];
    let p95 = data[len * 95 / 100];
    let p99 = data[(len * 99 / 100).min(len - 1)];
    (p50, p90, p95, p99)
}
