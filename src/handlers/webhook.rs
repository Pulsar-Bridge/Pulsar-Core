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

fn validate(payload: &WebhookPayload) -> AppResult<BigDecimal> {
    if payload.external_deposit_id.trim().is_empty() {
        return Err(AppError::InvalidPayload(
            "external_deposit_id is required".into(),
        ));
    }
    if payload.asset_code.trim().is_empty() {
        return Err(AppError::InvalidPayload("asset_code is required".into()));
    }
    if payload.stellar_account.len() != 56 || !payload.stellar_account.starts_with('G') {
        return Err(AppError::InvalidPayload(
            "stellar_account does not look like a Stellar public key".into(),
        ));
    }
    let amount = BigDecimal::from_str(&payload.amount)
        .map_err(|_| AppError::InvalidPayload("amount is not a valid decimal".into()))?;
    if amount <= BigDecimal::from(0) {
        return Err(AppError::InvalidPayload("amount must be positive".into()));
    }
    Ok(amount)
}

pub async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::Extension(tenant): axum::Extension<TenantContext>,
    Json(payload): Json<WebhookPayload>,
) -> AppResult<impl IntoResponse> {
    let idempotency_key = headers
        .get("X-Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .ok_or(AppError::MissingIdempotencyKey)?
        .to_string();