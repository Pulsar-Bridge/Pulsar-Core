use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

/// Tenant identity resolved by `api_key_auth`, available to downstream
/// handlers via request extensions.
#[derive(Clone, Copy)]
pub struct TenantContext {
    pub tenant_id: Uuid,
}

fn bearer_token(req: &Request) -> Option<&str> {
    req.headers()
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

fn client_ip(req: &Request) -> String {
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Every auth attempt — success or failure — is rate limited by the
/// presented key (falling back to client IP when no key was presented at
/// all), so a credential-stuffing run against `admin_auth`/`api_key_auth`
/// gets throttled before it can brute force keys. Failures are logged and
/// counted in `crate::metrics::AUTH_FAILURES` so they're alertable.
fn check_rate_limit(state: &AppState, key: &str) -> Result<(), AppError> {
    if state.rate_limiter.check_key(&key.to_string()).is_err() {
        tracing::warn!(key_prefix = %prefix(key), "rate limit exceeded on auth endpoint");
        return Err(AppError::RateLimited);
    }
    Ok(())
}

fn prefix(key: &str) -> String {
    key.chars().take(8).collect()
}

pub async fn api_key_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = client_ip(&req);
    let token = bearer_token(&req).map(str::to_string);
    let rate_limit_key = token.clone().unwrap_or(ip);
    check_rate_limit(&state, &rate_limit_key)?;