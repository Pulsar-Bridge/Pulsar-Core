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

pub struct NewDeposit {
    pub tenant_id: Uuid,
    pub idempotency_key: String,
    pub external_deposit_id: Option<String>,
    pub amount: sqlx::types::BigDecimal,
    pub asset_code: String,
    pub stellar_account: String,
    pub anchor_platform_payload: serde_json::Value,
}

/// Inserts a new deposit in `pending` status. Must be called on a transaction
/// opened with `db::begin_tenant_scoped` so RLS scopes the insert to
/// `new_deposit.tenant_id`.
pub async fn insert_pending(
    tx: &mut Transaction<'_, Postgres>,
    new_deposit: NewDeposit,
) -> AppResult<DepositTransaction> {
    sqlx::query_as::<_, DepositTransaction>(
        r#"
        INSERT INTO transactions
            (tenant_id, idempotency_key, external_deposit_id, status, amount,
             asset_code, stellar_account, anchor_platform_payload)
        VALUES ($1, $2, $3, 'pending', $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(new_deposit.tenant_id)
    .bind(new_deposit.idempotency_key)
    .bind(new_deposit.external_deposit_id)
    .bind(new_deposit.amount)
    .bind(new_deposit.asset_code)
    .bind(new_deposit.stellar_account)
    .bind(new_deposit.anchor_platform_payload)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::Database)
}

pub async fn mark_submitted(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    created_at: DateTime<Utc>,
    contract_tx_hash: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE transactions SET status = 'submitted', contract_tx_hash = $3
         WHERE id = $1 AND created_at = $2",
    )
    .bind(id)
    .bind(created_at)
    .bind(contract_tx_hash)
    .execute(&mut **tx)
    .await
    .map_err(AppError::Database)?;
    Ok(())
}

pub async fn mark_failed(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    created_at: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query("UPDATE transactions SET status = 'failed' WHERE id = $1 AND created_at = $2")
        .bind(id)
        .bind(created_at)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}