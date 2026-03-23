use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use crate::histogram::AtomicHistogram;

/// Per-second snapshot for time-series analysis.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Snapshot {
    pub elapsed_secs: u64,
    pub connections: u64,
    pub msgs_sent: u64,
    pub msgs_sent_per_sec: u64,
    pub map_recv: u64,
    pub map_recv_per_sec: u64,
    pub group_recv: u64,
    pub send_errors: u64,
    pub lat_p50: u64,
    pub lat_p95: u64,
    pub lat_p99: u64,
}

pub struct Metrics {
    pub connections_attempted: AtomicU64,
    pub connections_successful: AtomicU64,
    pub connections_failed: AtomicU64,
    pub conn_time: AtomicHistogram,

    pub messages_sent: AtomicU64,
    pub send_errors: AtomicU64,

    pub location_messages_received: AtomicU64,
    pub group_messages_received: AtomicU64,

    pub latency: AtomicHistogram,

    pub time_series: Mutex<Vec<Snapshot>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            connections_attempted: AtomicU64::new(0),
            connections_successful: AtomicU64::new(0),
            connections_failed: AtomicU64::new(0),
            conn_time: AtomicHistogram::new(),
            messages_sent: AtomicU64::new(0),
            send_errors: AtomicU64::new(0),
            location_messages_received: AtomicU64::new(0),
            group_messages_received: AtomicU64::new(0),
            latency: AtomicHistogram::new(),
            time_series: Mutex::new(Vec::new()),
        }
    }

    pub fn record_connection_time(&self, ms: u64) {
        self.conn_time.record(ms);
    }

    pub fn record_latency(&self, ms: u64) {
        self.latency.record(ms);
    }

    /// Capture a time-series snapshot. Called once per second by the progress reporter.
    pub fn capture_snapshot(
        &self,
        elapsed_secs: u64,
        prev_sent: u64,
        prev_map_recv: u64,
    ) {
        let sent = self.messages_sent.load(std::sync::atomic::Ordering::Relaxed);
        let map_recv = self.location_messages_received.load(std::sync::atomic::Ordering::Relaxed);
        let lat = self.latency.percentiles();

        let snapshot = Snapshot {
            elapsed_secs,
            connections: self.connections_successful.load(std::sync::atomic::Ordering::Relaxed),
            msgs_sent: sent,
            msgs_sent_per_sec: sent.saturating_sub(prev_sent),
            map_recv,
            map_recv_per_sec: map_recv.saturating_sub(prev_map_recv),
            group_recv: self.group_messages_received.load(std::sync::atomic::Ordering::Relaxed),
            send_errors: self.send_errors.load(std::sync::atomic::Ordering::Relaxed),
            lat_p50: lat.p50,
            lat_p95: lat.p95,
            lat_p99: lat.p99,
        };

        self.time_series.lock().unwrap().push(snapshot);
    }
}
