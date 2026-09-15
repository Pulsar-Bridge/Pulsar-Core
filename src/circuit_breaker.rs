use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Closed,
    Open,
    HalfOpen,
}

/// A minimal closed/open/half-open circuit breaker guarding the Horizon
/// client. Opens after `failure_threshold` consecutive failures, stays open
/// for `reset_after`, then allows a single half-open probe before deciding
/// to close (probe succeeded) or re-open (probe failed).
pub struct CircuitBreaker {
    name: String,
    failure_threshold: u32,
    reset_after: Duration,
    consecutive_failures: AtomicU32,
    opened_at_unix_secs: AtomicU64,
}

impl CircuitBreaker {
    pub fn new(name: impl Into<String>, failure_threshold: u32, reset_after: Duration) -> Self {
        Self {
            name: name.into(),
            failure_threshold,
            reset_after,
            consecutive_failures: AtomicU32::new(0),
            opened_at_unix_secs: AtomicU64::new(0),
        }
    }