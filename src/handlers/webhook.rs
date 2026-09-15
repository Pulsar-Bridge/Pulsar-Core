use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::contracts::RegisterCallbackRequest;
use crate::db::{self, transactions};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::TenantContext;
use crate::redis_store::Claim;
use crate::state::AppState;