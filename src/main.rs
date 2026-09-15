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