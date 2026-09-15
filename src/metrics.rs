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

pub static IDEMPOTENCY_HITS: Counter = Counter::new();
pub static IDEMPOTENCY_MISSES: Counter = Counter::new();
pub static PARTITION_JOB_RUNS: Counter = Counter::new();
pub static PARTITION_JOB_FAILURES: Counter = Counter::new();
pub static CIRCUIT_BREAKER_OPENS: Counter = Counter::new();
pub static AUTH_FAILURES: Counter = Counter::new();

pub fn render() -> String {
    format!(
        "# TYPE pulsar_idempotency_hits_total counter\n\
         pulsar_idempotency_hits_total {}\n\
         # TYPE pulsar_idempotency_misses_total counter\n\
         pulsar_idempotency_misses_total {}\n\
         # TYPE pulsar_partition_job_runs_total counter\n\
         pulsar_partition_job_runs_total {}\n\
         # TYPE pulsar_partition_job_failures_total counter\n\
         pulsar_partition_job_failures_total {}\n\
         # TYPE pulsar_circuit_breaker_opens_total counter\n\
         pulsar_circuit_breaker_opens_total {}\n\
         # TYPE pulsar_auth_failures_total counter\n\
         pulsar_auth_failures_total {}\n",
        IDEMPOTENCY_HITS.get(),
        IDEMPOTENCY_MISSES.get(),
        PARTITION_JOB_RUNS.get(),
        PARTITION_JOB_FAILURES.get(),
        CIRCUIT_BREAKER_OPENS.get(),
        AUTH_FAILURES.get(),
    )
}
