use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::circuit_breaker::State as BreakerState;
use crate::state::AppState;