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

/// How long a connection waits for another connection's lock before
/// returning SQLITE_BUSY.
pub const BUSY_TIMEOUT_MS: u32 = 5_000;

/// Configure a connection with pragmas for optimal operation.
///
/// Enables:
/// - `busy_timeout` so a write waits for a lock instead of failing at once
/// - `journal_mode=WAL` for concurrent reads
/// - `foreign_keys=ON` for referential integrity
/// - `synchronous=NORMAL` for better performance with WAL
fn configure_connection(conn: &mut SqliteConnection) {
    // The desktop app and the MCP server open the same file. Without a busy
    // timeout, a write that meets another process's lock fails at once with
    // SQLITE_BUSY ("database is locked"). Set first so the WAL switch below
    // also waits. (MIMIR-T-0643)
    diesel::sql_query(format!("PRAGMA busy_timeout={}", BUSY_TIMEOUT_MS))
        .execute(conn)
        .ok();

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

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::BigInt;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[derive(QueryableByName)]
    struct BusyTimeout {
        #[diesel(sql_type = BigInt)]
        timeout: i64,
    }

    fn temp_db_url() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let url = dir.path().join("test.db").to_string_lossy().into_owned();
        (dir, url)
    }

    #[test]
    fn create_connection_sets_busy_timeout() {
        let (_dir, url) = temp_db_url();
        let mut conn = create_connection(&url).unwrap();
        let row: BusyTimeout = diesel::sql_query("PRAGMA busy_timeout")
            .get_result(&mut conn)
            .unwrap();
        assert_eq!(row.timeout, i64::from(BUSY_TIMEOUT_MS));
    }

    #[test]
    fn write_waits_for_a_held_lock_instead_of_failing() {
        let (_dir, url) = temp_db_url();
        let mut setup = init_database(&url).unwrap();
        diesel::sql_query("CREATE TABLE t (x INTEGER)")
            .execute(&mut setup)
            .unwrap();
        drop(setup);

        // Another connection (think: the desktop app) holds the write lock
        // for a short time, then commits.
        let (locked_tx, locked_rx) = mpsc::channel();
        let holder_url = url.clone();
        let holder = std::thread::spawn(move || {
            let mut conn = create_connection(&holder_url).unwrap();
            diesel::sql_query("BEGIN IMMEDIATE")
                .execute(&mut conn)
                .unwrap();
            diesel::sql_query("INSERT INTO t VALUES (1)")
                .execute(&mut conn)
                .unwrap();
            locked_tx.send(()).unwrap();
            std::thread::sleep(Duration::from_millis(300));
            diesel::sql_query("COMMIT").execute(&mut conn).unwrap();
        });
        locked_rx.recv().unwrap();

        // This write meets the lock. It must wait, not fail with SQLITE_BUSY.
        let mut conn = create_connection(&url).unwrap();
        let started = Instant::now();
        let result = diesel::sql_query("INSERT INTO t VALUES (2)").execute(&mut conn);
        holder.join().unwrap();

        assert!(
            result.is_ok(),
            "write failed instead of waiting: {:?}",
            result
        );
        assert!(
            started.elapsed() >= Duration::from_millis(100),
            "write did not actually meet the lock (elapsed {:?})",
            started.elapsed()
        );
    }
}
