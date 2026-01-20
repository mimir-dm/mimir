//! Database connection management with r2d2 connection pooling

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use tracing::{info, warn};

/// Connection pool type for SQLite connections.
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
/// Pooled SQLite connection type.
pub type DbConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

/// Database service that owns the connection pool
pub struct DatabaseService {
    pool: DbPool,
}

impl DatabaseService {
    /// Create a new DatabaseService with connection pooling
    pub fn new(database_url: &str, is_memory_db: bool) -> Result<Self> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);

        let mut pool_builder = Pool::builder();

        if is_memory_db {
            // For in-memory database, we need to maintain at least one connection
            // to prevent the database from being destroyed
            info!("Configuring connection pool for in-memory database");
            pool_builder = pool_builder
                .min_idle(Some(1)) // Always keep at least 1 connection
                .max_size(1); // Use only 1 connection for in-memory DB
        }

        let pool = pool_builder
            .build(manager)
            .context("Failed to create connection pool")?;

        // For in-memory databases, run migrations on the pool connection
        if is_memory_db {
            let mut conn = pool.get().context("Failed to get connection from pool")?;
            info!("Running migrations on in-memory database...");
            match crate::run_migrations(&mut conn) {
                Ok(_) => info!("Migrations completed successfully"),
                Err(e) => warn!("Migration warning: {}", e),
            }
        }

        Ok(Self { pool })
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> Result<DbConnection> {
        self.pool
            .get()
            .context("Failed to get connection from pool")
    }

    /// Get a reference to the underlying pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}
