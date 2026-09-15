use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

/// Typed, boundary-level error type for the service.
///
/// Every variant maps to a specific HTTP status and a stable machine-readable
/// `code` so callers (and `pulsar-web`) can branch on it instead of parsing
/// message strings. Internal details never leak into the response body — they
/// go to `tracing` instead.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("invalid request payload: {0}")]
    InvalidPayload(String),

    #[error("missing idempotency key")]
    MissingIdempotencyKey,

    #[error("duplicate request in flight")]
    IdempotencyConflict,

    #[error("unauthorized")]
    Unauthorized,

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("cache error")]
    Cache(#[from] redis::RedisError),

    #[error("upstream horizon/contract call failed: {0}")]
    Upstream(String),

    #[error("circuit breaker open for {0}")]
    CircuitOpen(String),

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config_error"),
            AppError::InvalidPayload(_) => (StatusCode::BAD_REQUEST, "invalid_payload"),
            AppError::MissingIdempotencyKey => (StatusCode::BAD_REQUEST, "missing_idempotency_key"),
            AppError::IdempotencyConflict => {
                (StatusCode::TOO_MANY_REQUESTS, "idempotency_conflict")
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            AppError::Cache(_) => (StatusCode::INTERNAL_SERVER_ERROR, "cache_error"),
            AppError::Upstream(_) => (StatusCode::BAD_GATEWAY, "upstream_error"),
            AppError::CircuitOpen(_) => (StatusCode::SERVICE_UNAVAILABLE, "circuit_open"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        }
    }
}