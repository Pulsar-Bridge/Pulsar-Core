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