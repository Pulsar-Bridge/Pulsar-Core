use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use governor::{Quota, RateLimiter};
use sqlx::PgPool;
use uuid::Uuid;