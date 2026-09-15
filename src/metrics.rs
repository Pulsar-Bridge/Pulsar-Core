use std::sync::atomic::{AtomicU64, Ordering};

/// Minimal dependency-free counters exposed as Prometheus text format at
/// `GET /metrics`. Swap for the `metrics`/`prometheus` crates if richer
/// histograms are needed later; these cover what `CLAUDE.md.pulsar-core`
/// calls out explicitly: circuit-breaker state, idempotency hit rate, and
/// partition-job runs.
pub struct Counter(AtomicU64);