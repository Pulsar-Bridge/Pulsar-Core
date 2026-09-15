use std::time::Duration;

use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use tokio::time::interval;