//! Database Utilities
//!
//! Provides database connection and migration helpers.
//!
//! Uses SQLite WAL (Write-Ahead Logging) mode for concurrent read access.
//! WAL allows multiple readers while writing, unlike the default rollback journal.

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

/// Embed all migrations at compile time.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Configure a connection with pragmas for optimal operation.
///
/// Enables:
/// - `journal_mode=WAL` for concurrent reads
/// - `foreign_keys=ON` for referential integrity
/// - `synchronous=NORMAL` for better performance with WAL
fn configure_connection(conn: &mut SqliteConnection) {
    // WAL mode allows concurrent readers and better write performance
    diesel::sql_query("PRAGMA journal_mode=WAL")
        .execute(conn)
        .ok();

    // Foreign keys must be enabled per-connection in SQLite
    diesel::sql_query("PRAGMA foreign_keys=ON")
        .execute(conn)
        .ok();

    // NORMAL synchronous is safe with WAL and faster than FULL
    diesel::sql_query("PRAGMA synchronous=NORMAL")
        .execute(conn)
        .ok();
}

/// Run all pending migrations on the given connection.
///
/// Returns the list of migration names that were run.
pub fn run_migrations(conn: &mut SqliteConnection) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let migrations = conn.run_pending_migrations(MIGRATIONS)?;
    Ok(migrations.iter().map(|m| m.to_string()).collect())
}

/// Establish a database connection, run migrations, and configure pragmas.
///
/// This is used during application startup to ensure the database is ready.
/// For on-demand connections after startup, use `create_connection`.
pub fn init_database(db_url: &str) -> Result<SqliteConnection, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = SqliteConnection::establish(db_url)?;

    // Configure pragmas (including WAL mode)
    configure_connection(&mut conn);

    // Run migrations
    run_migrations(&mut conn)?;

    Ok(conn)
}

/// Create a new database connection with pragmas configured.
///
/// Use this for on-demand connections after the database has been initialized.
/// Each connection is configured with WAL mode, foreign keys, and optimal settings.
pub fn create_connection(db_url: &str) -> Result<SqliteConnection, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = SqliteConnection::establish(db_url)?;
    configure_connection(&mut conn);
    Ok(conn)
}

/// Create an in-memory SQLite connection for testing.
#[cfg(test)]
pub fn test_connection() -> SqliteConnection {
    init_database(":memory:").expect("Failed to create test database")
}
