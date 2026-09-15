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