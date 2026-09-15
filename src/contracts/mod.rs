use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::circuit_breaker::CircuitBreaker;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct RegisterCallbackRequest {
    pub tenant_id: String,
    pub deposit_id: String,
    pub amount: String,
    pub asset_code: String,
    pub stellar_account: String,
}

#[async_trait]
pub trait ContractClient: Send + Sync {
    async fn register_callback(&self, req: RegisterCallbackRequest) -> AppResult<String>;
    fn breaker_state(&self) -> crate::circuit_breaker::State;
}

/// HTTP client for `pulsar-core-contracts`' `register_callback()` entry
/// point, called as the trusted `relay_signer`.
///
/// **The request/response shape below is a placeholder**, not a confirmed
/// contract. `pulsar-core-contracts` is a sibling repo this checkout does not
/// have cloned (see the top of `CLAUDE.md.pulsar-core`), so its real
/// `EVENTS.md` / ABI has not been read. Do not treat this as load-bearing —
/// confirm the actual signature against that repo and update this module (and
/// `docs/contract-integration.md`) before relying on it against a real
/// contract deployment.
pub struct SorobanContractClient {
    http: reqwest::Client,
    rpc_url: String,
    relay_signer_secret: String,
    breaker: CircuitBreaker,
}

impl SorobanContractClient {
    pub fn new(
        rpc_url: String,
        relay_signer_secret: String,
        failure_threshold: u32,
        reset_after: Duration,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            rpc_url,
            relay_signer_secret,
            breaker: CircuitBreaker::new("contract_rpc", failure_threshold, reset_after),
        }
    }
}

#[derive(Serialize)]
struct RegisterCallbackWireRequest<'a> {
    tenant_id: &'a str,
    deposit_id: &'a str,
    amount: &'a str,
    asset_code: &'a str,
    stellar_account: &'a str,
}

#[derive(Deserialize)]
struct RegisterCallbackWireResponse {
    tx_hash: String,
}

#[async_trait]
impl ContractClient for SorobanContractClient {
    async fn register_callback(&self, req: RegisterCallbackRequest) -> AppResult<String> {
        self.breaker
            .call(|| async {
                let wire = RegisterCallbackWireRequest {
                    tenant_id: &req.tenant_id,
                    deposit_id: &req.deposit_id,
                    amount: &req.amount,
                    asset_code: &req.asset_code,
                    stellar_account: &req.stellar_account,
                };

                let resp = self
                    .http
                    .post(format!("{}/register_callback", self.rpc_url))
                    .bearer_auth(&self.relay_signer_secret)
                    .json(&wire)
                    .timeout(Duration::from_secs(10))
                    .send()
                    .await
                    .map_err(|e| AppError::Upstream(e.to_string()))?;