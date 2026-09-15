use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{self, transactions};
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

/// Lists transactions for one explicitly-named tenant. This still goes
/// through `begin_tenant_scoped` (i.e. through RLS) — admin access is an
/// authorization decision made in application code (any tenant, by design),
/// never a database-level RLS bypass. See docs/security-design.md for why
/// that distinction matters.
pub async fn list_transactions(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<transactions::DepositTransaction>>> {
    let mut tx = db::begin_tenant_scoped(&state.db, tenant_id).await?;
    let rows = transactions::list_for_tenant(&mut tx, query.limit.clamp(1, 500)).await?;
    tx.commit()
        .await
        .map_err(crate::error::AppError::Database)?;
    Ok(Json(rows))
}
