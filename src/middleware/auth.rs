use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;