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

impl HorizonClient {
    pub fn new(base_url: String, failure_threshold: u32, reset_after: Duration) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            breaker: CircuitBreaker::new("horizon", failure_threshold, reset_after),
        }
    }

    pub fn breaker_state(&self) -> crate::circuit_breaker::State {
        self.breaker.state()
    }

    /// Confirms the Horizon endpoint is reachable; used by `/readyz`.
    pub async fn ping(&self) -> AppResult<()> {
        self.breaker
            .call(|| async {
                let resp = self
                    .http
                    .get(format!("{}/", self.base_url))
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await
                    .map_err(|e| AppError::Upstream(e.to_string()))?;

                if resp.status().is_success() {
                    Ok(())
                } else {
                    Err(AppError::Upstream(format!(
                        "horizon returned {}",
                        resp.status()
                    )))
                }
            })
            .await
    }
}
