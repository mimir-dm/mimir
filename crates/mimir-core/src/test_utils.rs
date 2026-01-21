//! Test utilities for mimir-core.
//!
//! Provides shared test database setup with embedded migrations.

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::dal::catalog::insert_source;
use crate::models::catalog::NewCatalogSource;

/// Embed all migrations at compile time.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Set up an in-memory database with all migrations applied.
///
/// This runs all embedded migrations and inserts common test sources.
pub fn setup_test_db() -> SqliteConnection {
    let mut conn =
        SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

    // Run all migrations
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    conn
}

/// Set up an in-memory database with common test sources pre-inserted.
///
/// Sources included: PHB, MM, DMG, XGE
pub fn setup_test_db_with_sources() -> SqliteConnection {
    let mut conn = setup_test_db();

    // Insert common test sources
    let sources = [
        NewCatalogSource::new("PHB", "Player's Handbook", true, "2024-01-20T12:00:00Z"),
        NewCatalogSource::new("MM", "Monster Manual", true, "2024-01-20T12:00:00Z"),
        NewCatalogSource::new("DMG", "Dungeon Master's Guide", true, "2024-01-20T12:00:00Z"),
        NewCatalogSource::new("XGE", "Xanathar's Guide to Everything", true, "2024-01-20T12:00:00Z"),
    ];

    for source in &sources {
        insert_source(&mut conn, source).expect("Failed to insert source");
    }

    conn
}
