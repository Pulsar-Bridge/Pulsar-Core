use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct DepositTransaction {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub idempotency_key: String,
    pub external_deposit_id: Option<String>,
    pub status: String,
    pub amount: sqlx::types::BigDecimal,
    pub asset_code: String,
    pub stellar_account: String,
    pub anchor_platform_payload: serde_json::Value,
    pub contract_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}