use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::circuit_breaker::State as BreakerState;
use crate::state::AppState;

/// Liveness: the process is up and can answer HTTP. No dependency checks —
/// a flapping dependency should not make the orchestrator kill and restart
/// this process.
pub async fn healthz() -> impl IntoResponse {
    StatusCode::OK
}

/// Readiness: the process can actually serve traffic. Checked against
/// Postgres, Redis, and Horizon (through its circuit breaker, so an open
/// breaker fails readiness rather than accepting traffic it will just 503).
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    let redis_ok = state.idempotency.ping().await.is_ok();

    // Only actually probe Horizon when its breaker isn't already open — no
    // point paying the latency of a call we know will be rejected.
    let horizon_breaker_ok = state.horizon.breaker_state() != BreakerState::Open;
    let horizon_ok = horizon_breaker_ok && state.horizon.ping().await.is_ok();
    let contract_breaker_ok = state.contracts.breaker_state() != BreakerState::Open;

    let ok = db_ok && redis_ok && horizon_ok && contract_breaker_ok;
    let status = if ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "database": db_ok,
            "redis": redis_ok,
            "horizon_reachable": horizon_ok,
            "contract_circuit_ok": contract_breaker_ok,
        })),
    )
}