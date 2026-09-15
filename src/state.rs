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