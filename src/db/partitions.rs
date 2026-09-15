use std::time::Duration;

use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use tokio::time::interval;

use crate::error::AppResult;

/// Ensures the current and next month's partitions exist, then drops
/// partitions older than `retention_months`. Called once at startup and then
/// on `interval` (default 24h) forever. Metrics are exposed via
/// `crate::metrics::PARTITION_JOB_RUNS` / `PARTITION_JOB_FAILURES`.
pub async fn run_once(pool: &PgPool, retention_months: u32) -> AppResult<()> {
    let today = Utc::now().date_naive();
    let this_month = first_of_month(today);
    let next_month = add_months(this_month, 1);

    sqlx::query("SELECT ensure_transactions_partition($1)")
        .bind(this_month)
        .execute(pool)
        .await?;
    sqlx::query("SELECT ensure_transactions_partition($1)")
        .bind(next_month)
        .execute(pool)
        .await?;

    let cutoff = add_months(this_month, -(retention_months as i32));
    let dropped: Vec<(String,)> =
        sqlx::query_as("SELECT dropped_partition FROM drop_transactions_partitions_older_than($1)")
            .bind(cutoff)
            .fetch_all(pool)
            .await?;

    if !dropped.is_empty() {
        tracing::info!(
            dropped = ?dropped.iter().map(|(n,)| n.as_str()).collect::<Vec<_>>(),
            "dropped retired transaction partitions"
        );
    }

    Ok(())
}