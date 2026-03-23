use std::sync::atomic::{AtomicU64, Ordering};

/// Lock-free histogram using predefined bucket boundaries.
/// Zero contention under high load — each record() is just atomic increments.
const BOUNDARIES: &[u64] = &[1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000];
const NUM_BUCKETS: usize = 14; // BOUNDARIES.len() + 1 overflow

pub struct AtomicHistogram {
    counts: [AtomicU64; NUM_BUCKETS],
    total_count: AtomicU64,
    total_sum: AtomicU64,
    min: AtomicU64,
    max: AtomicU64,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Percentiles {
    pub p50: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
    pub avg: f64,
    pub min: u64,
    pub max: u64,
    pub count: u64,
}

impl AtomicHistogram {
    pub fn new() -> Self {
        Self {
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
            total_count: AtomicU64::new(0),
            total_sum: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    /// Record a value. Completely lock-free.
    pub fn record(&self, value: u64) {
        let idx = BOUNDARIES
            .iter()
            .position(|&b| value <= b)
            .unwrap_or(BOUNDARIES.len());
        self.counts[idx].fetch_add(1, Ordering::Relaxed);
        self.total_count.fetch_add(1, Ordering::Relaxed);
        self.total_sum.fetch_add(value, Ordering::Relaxed);
        self.min.fetch_min(value, Ordering::Relaxed);
        self.max.fetch_max(value, Ordering::Relaxed);
    }

    pub fn count(&self) -> u64 {
        self.total_count.load(Ordering::Relaxed)
    }

    fn avg(&self) -> f64 {
        let count = self.total_count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        self.total_sum.load(Ordering::Relaxed) as f64 / count as f64
    }

    fn min_val(&self) -> u64 {
        let v = self.min.load(Ordering::Relaxed);
        if v == u64::MAX { 0 } else { v }
    }

    fn max_val(&self) -> u64 {
        self.max.load(Ordering::Relaxed)
    }

    /// Approximate percentile using bucket boundaries.
    fn percentile(&self, pct: f64) -> u64 {
        let total = self.total_count.load(Ordering::Relaxed);
        if total == 0 {
            return 0;
        }
        let target = (total as f64 * pct / 100.0).ceil() as u64;
        let mut accumulated = 0u64;
        for (i, count) in self.counts.iter().enumerate() {
            accumulated += count.load(Ordering::Relaxed);
            if accumulated >= target {
                return if i < BOUNDARIES.len() {
                    BOUNDARIES[i]
                } else {
                    self.max.load(Ordering::Relaxed)
                };
            }
        }
        self.max.load(Ordering::Relaxed)
    }

    /// Snapshot all percentiles at once.
    pub fn percentiles(&self) -> Percentiles {
        Percentiles {
            p50: self.percentile(50.0),
            p90: self.percentile(90.0),
            p95: self.percentile(95.0),
            p99: self.percentile(99.0),
            avg: self.avg(),
            min: self.min_val(),
            max: self.max_val(),
            count: self.count(),
        }
    }
}
