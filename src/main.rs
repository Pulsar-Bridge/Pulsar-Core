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