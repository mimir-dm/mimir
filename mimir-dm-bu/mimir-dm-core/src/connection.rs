//! Database connection management

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::error::Result;

/// Type alias for our database connection
pub type DbConnection = SqliteConnection;

/// Establish a connection to the SQLite database
pub fn establish_connection(database_url: &str) -> Result<DbConnection> {
    let mut conn = DbConnection::establish(database_url)?;

    // Enable foreign key constraints
    diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn)?;

    // Enable WAL mode for better concurrency
    diesel::sql_query("PRAGMA journal_mode = WAL").execute(&mut conn)?;

    // Set busy timeout to 5 seconds
    diesel::sql_query("PRAGMA busy_timeout = 5000").execute(&mut conn)?;

    Ok(conn)
}

/// Create an in-memory database for testing
#[cfg(test)]
pub fn establish_test_connection() -> Result<DbConnection> {
    establish_connection(":memory:")
}
