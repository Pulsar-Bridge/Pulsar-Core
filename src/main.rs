mod circuit_breaker;
mod config;
mod contracts;
mod db;
mod error;
mod handlers;
mod horizon;
mod metrics;
mod middleware;
mod redis_store;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;

use crate::config::Config;
use crate::contracts::SorobanContractClient;
use crate::horizon::HorizonClient;
use crate::redis_store::IdempotencyStore;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .json()
        .init();

    let config = Config::from_env()?;

    let db = db::connect(&config.database_url, config.database_max_connections).await?;
    config::assert_not_rls_bypassing(&db).await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    db::partitions::run_once(&db, config.partition_retention_months).await?;
    db::partitions::spawn_background_job(
        db.clone(),
        config.partition_retention_months,
        config.partition_maintenance_interval,
    );

    let idempotency = IdempotencyStore::connect(&config.redis_url, config.idempotency_ttl).await?;

    let horizon = HorizonClient::new(
        config.horizon_base_url.clone(),
        config.horizon_circuit_breaker_failure_threshold,
        config.horizon_circuit_breaker_reset_after,
    );