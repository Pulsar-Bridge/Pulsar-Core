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