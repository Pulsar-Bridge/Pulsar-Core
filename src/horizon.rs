use std::time::Duration;

use crate::circuit_breaker::CircuitBreaker;
use crate::error::{AppError, AppResult};

/// Thin wrapper over the Stellar Horizon REST API, guarded by a circuit
/// breaker so a degraded Horizon doesn't cascade into webhook-handling
/// latency. Only the lookups the relay actually needs are implemented;
/// extend as `PRIMARY ROLE` work requires more.
pub struct HorizonClient {
    http: reqwest::Client,
    base_url: String,
    breaker: CircuitBreaker,
}