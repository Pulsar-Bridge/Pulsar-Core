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

impl IdempotencyStore {
    pub async fn connect(redis_url: &str, ttl: Duration) -> AppResult<Self> {
        let client = redis::Client::open(redis_url).map_err(AppError::Cache)?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(AppError::Cache)?;
        Ok(Self { conn, ttl })
    }

    /// Atomically claims `idempotency_key` for `tenant_id` using `SET NX EX`.
    /// Concurrent requests with the same key race on this single Redis
    /// command, so exactly one gets `Acquired`; the rest get
    /// `AlreadyClaimed` and the handler must return 429, per this repo's
    /// idempotency contract (`docs/auth-rate-limiting.md`).
    pub async fn claim(&self, tenant_id: &str, idempotency_key: &str) -> AppResult<Claim> {
        let key = redis_key(tenant_id, idempotency_key);
        let mut conn = self.conn.clone();
        let set: Option<String> = redis::cmd("SET")
            .arg(&key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(self.ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(AppError::Cache)?;