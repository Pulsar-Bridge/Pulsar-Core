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

impl AppState {
    pub fn new(
        config: crate::config::Config,
        db: PgPool,
        idempotency: IdempotencyStore,
        horizon: HorizonClient,
        contracts: Arc<dyn ContractClient>,
    ) -> anyhow::Result<Self> {
        let quota = Quota::per_minute(
            NonZeroU32::new(config.rate_limit_per_minute).unwrap_or(NonZeroU32::new(60).unwrap()),
        );
        let rate_limiter = Arc::new(RateLimiter::keyed(quota));

        let tenant_api_keys = config
            .tenant_api_keys
            .iter()
            .map(|(key, tenant)| Ok((key.clone(), Uuid::parse_str(tenant)?)))
            .collect::<anyhow::Result<HashMap<_, _>>>()?;