use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::circuit_breaker::CircuitBreaker;
use crate::error::{AppError, AppResult};