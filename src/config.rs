use std::time::Duration;

use crate::error::{AppError, AppResult};

/// Runtime configuration, loaded from environment variables.
///
/// See `.env.example` for the full variable list and `docs/security-design.md`
/// for why `database_url` must never point at a superuser / RLS-bypass role.
#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    pub idempotency_ttl: Duration,
    pub horizon_base_url: String,
    pub horizon_circuit_breaker_failure_threshold: u32,
    pub horizon_circuit_breaker_reset_after: Duration,
    pub contract_rpc_url: String,
    pub relay_signer_secret: String,
    pub admin_api_keys: Vec<String>,
    pub tenant_api_keys: Vec<(String, String)>, // (api_key, tenant_id)
    pub rate_limit_per_minute: u32,
    pub partition_retention_months: u32,
    pub partition_maintenance_interval: Duration,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        let _ = dotenvy::dotenv();