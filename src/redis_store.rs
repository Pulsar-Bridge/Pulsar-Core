use std::time::Duration;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct IdempotencyStore {
    conn: ConnectionManager,
    ttl: Duration,
}