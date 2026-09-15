use std::sync::atomic::{AtomicU64, Ordering};

/// Minimal dependency-free counters exposed as Prometheus text format at
/// `GET /metrics`. Swap for the `metrics`/`prometheus` crates if richer
/// histograms are needed later; these cover what `CLAUDE.md.pulsar-core`
/// calls out explicitly: circuit-breaker state, idempotency hit rate, and
/// partition-job runs.
pub struct Counter(AtomicU64);

impl Counter {
    const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub fn increment(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}