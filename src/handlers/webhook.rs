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

    let amount = validate(&payload)?;
    let tenant_id_str = tenant.tenant_id.to_string();

    let claim = state
        .idempotency
        .claim(&tenant_id_str, &idempotency_key)
        .await?;
    if matches!(claim, Claim::AlreadyClaimed) {
        return Err(AppError::IdempotencyConflict);
    }

    let insert_result = insert_and_submit(&state, tenant, &idempotency_key, &payload, amount).await;

    if insert_result.is_err() {
        // Payload was valid but something downstream failed before a
        // customer-visible side effect completed; release the claim so a
        // corrected/retried webhook within the TTL window isn't dropped.
        let _ = state
            .idempotency
            .release(&tenant_id_str, &idempotency_key)
            .await;
    }

    insert_result
}

async fn insert_and_submit(
    state: &AppState,
    tenant: TenantContext,
    idempotency_key: &str,
    payload: &WebhookPayload,
    amount: BigDecimal,
) -> AppResult<(StatusCode, Json<WebhookResponse>)> {
    let mut tx = db::begin_tenant_scoped(&state.db, tenant.tenant_id).await?;

    if let Some(existing) = transactions::find_by_idempotency_key(&mut tx, idempotency_key).await? {
        tx.commit().await.map_err(AppError::Database)?;
        return Ok((
            StatusCode::OK,
            Json(WebhookResponse {
                id: existing.id,
                status: existing.status,
                contract_tx_hash: existing.contract_tx_hash,
            }),
        ));
    }

    let record = transactions::insert_pending(
        &mut tx,
        transactions::NewDeposit {
            tenant_id: tenant.tenant_id,
            idempotency_key: idempotency_key.to_string(),
            external_deposit_id: Some(payload.external_deposit_id.clone()),
            amount,
            asset_code: payload.asset_code.clone(),
            stellar_account: payload.stellar_account.clone(),
            anchor_platform_payload: payload.extra.clone(),
        },
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    let register_result = state
        .contracts
        .register_callback(RegisterCallbackRequest {
            tenant_id: tenant.tenant_id.to_string(),
            deposit_id: record.id.to_string(),
            amount: record.amount.to_string(),
            asset_code: record.asset_code.clone(),
            stellar_account: record.stellar_account.clone(),
        })
        .await;

    let mut tx = db::begin_tenant_scoped(&state.db, tenant.tenant_id).await?;
    let (status, contract_tx_hash) = match register_result {
        Ok(tx_hash) => {
            transactions::mark_submitted(&mut tx, record.id, record.created_at, &tx_hash).await?;
            ("submitted", Some(tx_hash))
        }
        Err(err) => {
            tracing::error!(error = %err, deposit_id = %record.id, "register_callback failed");
            transactions::mark_failed(&mut tx, record.id, record.created_at).await?;
            ("failed", None)
        }
    };
    tx.commit().await.map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(WebhookResponse {
            id: record.id,
            status: status.to_string(),
            contract_tx_hash,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(overrides: impl FnOnce(&mut WebhookPayload)) -> WebhookPayload {
        let mut p = WebhookPayload {
            external_deposit_id: "dep-123".to_string(),
            amount: "10.5".to_string(),
            asset_code: "USD".to_string(),
            stellar_account: format!("G{}", "A".repeat(55)),
            extra: serde_json::json!({}),
        };
        overrides(&mut p);
        p
    }

    #[test]
    fn accepts_valid_payload() {
        let p = payload(|_| {});
        assert!(validate(&p).is_ok());
    }

    #[test]
    fn rejects_empty_deposit_id() {
        let p = payload(|p| p.external_deposit_id = "".to_string());
        assert!(validate(&p).is_err());
    }