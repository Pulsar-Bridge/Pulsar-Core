pub mod partitions;
pub mod transactions;

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

pub async fn connect(database_url: &str, max_connections: u32) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .map_err(AppError::Database)?;
    Ok(pool)
}