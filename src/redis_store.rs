use std::time::Duration;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct IdempotencyStore {
    conn: ConnectionManager,
    ttl: Duration,
}

pub enum Claim {
    /// First time this key has been seen within the TTL window — the caller
    /// owns it and should proceed with processing.
    Acquired,
    /// A prior request with this key is still in flight or already
    /// completed within the TTL window.
    AlreadyClaimed,
}