pub mod partitions;
pub mod transactions;

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

pub async fn connect(database_url: &str, max_connections: u32) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .map_err(AppError::Database)?;
    Ok(pool)
}

/// Opens a transaction and sets `app.tenant_id` for the lifetime of that
/// transaction so the `tenant_isolation` RLS policy scopes every statement
/// run on it. Callers must always go through this (never a bare
/// `pool.begin()`) when the query touches tenant-owned data — see
/// docs/security-design.md.
pub async fn begin_tenant_scoped<'a>(
    pool: &PgPool,
    tenant_id: Uuid,
) -> AppResult<Transaction<'a, Postgres>> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;
    Ok(tx)
}
