//! Database layer for Paw server
//!
//! Uses SQLx with PostgreSQL.
//! Phase 1: No E2EE columns (added in Phase 2 migrations)
//!
//! Key design: server-assigned monotonic `seq` per conversation
//! is the source of truth for ordering and gap-fill.

pub mod schema;

use sqlx::PgPool;
use std::sync::Arc;

/// Shared database connection pool
pub type DbPool = Arc<PgPool>;

/// Create a new database connection pool
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;
    Ok(Arc::new(pool))
}
