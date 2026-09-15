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