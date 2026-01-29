//! Database Utilities
//!
//! Provides database connection and migration helpers.

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

/// Embed all migrations at compile time.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Run all pending migrations on the given connection.
///
/// Returns the list of migration names that were run.
pub fn run_migrations(conn: &mut SqliteConnection) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let migrations = conn.run_pending_migrations(MIGRATIONS)?;
    Ok(migrations.iter().map(|m| m.to_string()).collect())
}

/// Establish a database connection and run migrations.
///
/// This is the primary entry point for initializing the database.
pub fn init_database(db_url: &str) -> Result<SqliteConnection, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = SqliteConnection::establish(db_url)?;

    // Enable foreign keys
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(&mut conn)
        .ok();

    // Run migrations
    run_migrations(&mut conn)?;

    Ok(conn)
}

/// Create an in-memory SQLite connection for testing.
#[cfg(test)]
pub fn test_connection() -> SqliteConnection {
    init_database(":memory:").expect("Failed to create test database")
}
