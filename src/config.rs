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

        let admin_api_keys = env_or("ADMIN_API_KEYS", "")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        let tenant_api_keys = env_or("TENANT_API_KEYS", "")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(|pair| {
                let (key, tenant) = pair.split_once(':')?;
                Some((key.to_string(), tenant.to_string()))
            })
            .collect();

        let rate_limit_per_minute = env_or("RATE_LIMIT_PER_MINUTE", "60")
            .parse()
            .map_err(|_| AppError::Config("RATE_LIMIT_PER_MINUTE must be a number".into()))?;

        let partition_retention_months = env_or("PARTITION_RETENTION_MONTHS", "12")
            .parse()
            .map_err(|_| AppError::Config("PARTITION_RETENTION_MONTHS must be a number".into()))?;

        let partition_maintenance_interval_secs: u64 =
            env_or("PARTITION_MAINTENANCE_INTERVAL_SECONDS", "86400")
                .parse()
                .map_err(|_| {
                    AppError::Config(
                        "PARTITION_MAINTENANCE_INTERVAL_SECONDS must be a number".into(),
                    )
                })?;

        Ok(Self {
            bind_addr,
            database_url,
            database_max_connections,
            redis_url,
            idempotency_ttl: Duration::from_secs(idempotency_ttl_secs),
            horizon_base_url,
            horizon_circuit_breaker_failure_threshold,
            horizon_circuit_breaker_reset_after: Duration::from_secs(
                horizon_circuit_breaker_reset_after_secs,
            ),
            contract_rpc_url,
            relay_signer_secret,
            admin_api_keys,
            tenant_api_keys,
            rate_limit_per_minute,
            partition_retention_months,
            partition_maintenance_interval: Duration::from_secs(
                partition_maintenance_interval_secs,
            ),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn require_env(key: &str) -> AppResult<String> {
    std::env::var(key).map_err(|_| AppError::Config(format!("missing required env var {key}")))
}

/// Refuses to start if the configured database role can bypass Row-Level Security.
///
/// This repo previously shipped with every `.env*` file pointed at the Postgres
/// `initdb` bootstrap superuser, which silently bypasses RLS regardless of how
/// correct the policies are. This check makes that failure mode fail loudly at
/// startup instead of leaking cross-tenant data at runtime. See
/// `docs/security-design.md`.
pub async fn assert_not_rls_bypassing(pool: &sqlx::PgPool) -> AppResult<()> {
    let row: (bool, bool) =
        sqlx::query_as("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user")
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;