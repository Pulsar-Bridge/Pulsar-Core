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

        let bind_addr = env_or("BIND_ADDR", "0.0.0.0:8080");
        let database_url = require_env("DATABASE_URL")?;
        let database_max_connections = env_or("DATABASE_MAX_CONNECTIONS", "10")
            .parse()
            .map_err(|_| AppError::Config("DATABASE_MAX_CONNECTIONS must be a number".into()))?;
        let redis_url = require_env("REDIS_URL")?;
        let idempotency_ttl_secs: u64 = env_or("IDEMPOTENCY_TTL_SECONDS", "86400")
            .parse()
            .map_err(|_| AppError::Config("IDEMPOTENCY_TTL_SECONDS must be a number".into()))?;
        let horizon_base_url = require_env("HORIZON_BASE_URL")?;
        let horizon_circuit_breaker_failure_threshold =
            env_or("HORIZON_CIRCUIT_BREAKER_FAILURE_THRESHOLD", "5")
                .parse()
                .map_err(|_| {
                    AppError::Config(
                        "HORIZON_CIRCUIT_BREAKER_FAILURE_THRESHOLD must be a number".into(),
                    )
                })?;
        let horizon_circuit_breaker_reset_after_secs: u64 =
            env_or("HORIZON_CIRCUIT_BREAKER_RESET_AFTER_SECONDS", "30")
                .parse()
                .map_err(|_| {
                    AppError::Config(
                        "HORIZON_CIRCUIT_BREAKER_RESET_AFTER_SECONDS must be a number".into(),
                    )
                })?;
        let contract_rpc_url = require_env("CONTRACT_RPC_URL")?;
        let relay_signer_secret = require_env("RELAY_SIGNER_SECRET")?;