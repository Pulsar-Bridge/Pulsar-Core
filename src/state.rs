use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use governor::{Quota, RateLimiter};
use sqlx::PgPool;
use uuid::Uuid;

use crate::contracts::ContractClient;
use crate::horizon::HorizonClient;
use crate::redis_store::IdempotencyStore;

pub type KeyedRateLimiter = RateLimiter<
    String,
    governor::state::keyed::DefaultKeyedStateStore<String>,
    governor::clock::DefaultClock,
>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub idempotency: IdempotencyStore,
    pub horizon: Arc<HorizonClient>,
    pub contracts: Arc<dyn ContractClient>,
    pub rate_limiter: Arc<KeyedRateLimiter>,
    /// api_key -> tenant_id
    pub tenant_api_keys: Arc<HashMap<String, Uuid>>,
    pub admin_api_keys: Arc<Vec<String>>,
}