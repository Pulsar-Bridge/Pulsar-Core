use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::contracts::RegisterCallbackRequest;
use crate::db::{self, transactions};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::TenantContext;
use crate::redis_store::Claim;
use crate::state::AppState;

/// Raw Anchor Platform deposit callback body. Only the fields this relay
/// actually needs are typed; everything else is preserved via `#[serde(flatten)]`
/// into `extra` and stored verbatim in `anchor_platform_payload` for audit /
/// replay.
#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    pub external_deposit_id: String,
    pub amount: String,
    pub asset_code: String,
    pub stellar_account: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Serialize)]
pub struct WebhookResponse {
    pub id: uuid::Uuid,
    pub status: String,
    pub contract_tx_hash: Option<String>,
}