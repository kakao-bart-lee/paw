//! Database layer for Paw server
//!
//! Uses SQLx with PostgreSQL.
//! Phase 1: No E2EE columns (added in Phase 2 migrations)
//!
//! Key design: server-assigned monotonic `seq` per conversation
//! is the source of truth for ordering and gap-fill.

pub mod schema;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use std::sync::Arc;

/// Shared database connection pool
pub type DbPool = Arc<PgPool>;

pub(crate) const POOL_MAX_CONNECTIONS: u32 = 20;
pub(crate) const POOL_MIN_CONNECTIONS: u32 = 5;
pub(crate) const POOL_ACQUIRE_TIMEOUT_SECS: u64 = 30;
pub(crate) const POOL_IDLE_TIMEOUT_SECS: u64 = 600;

/// Create a new database connection pool
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(POOL_MAX_CONNECTIONS)
        .min_connections(POOL_MIN_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(POOL_ACQUIRE_TIMEOUT_SECS))
        .idle_timeout(Duration::from_secs(POOL_IDLE_TIMEOUT_SECS))
        .connect(database_url)
        .await?;

    Ok(Arc::new(pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_config_constants_match_performance_targets() {
        assert_eq!(POOL_MAX_CONNECTIONS, 20);
        assert_eq!(POOL_MIN_CONNECTIONS, 5);
        assert_eq!(POOL_ACQUIRE_TIMEOUT_SECS, 30);
        assert_eq!(POOL_IDLE_TIMEOUT_SECS, 600);
    }
}
