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