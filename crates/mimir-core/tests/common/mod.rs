#![allow(dead_code)]
//! Shared test helpers for mimir-core integration tests.
//!
//! Provides `setup_srd_db()` which creates an in-memory SQLite database
//! pre-seeded with SRD fixture data from `tests/fixtures/*.json`.
//!
//! SRD content is published under the OGL and safe to commit.

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use mimir_core::dal::catalog;
use mimir_core::models::catalog::NewCatalogSource;
use mimir_core::seed::srd::seed_srd_catalog;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Create an in-memory database with migrations applied.
fn create_db() -> SqliteConnection {
    let mut conn =
        SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    conn
}

/// Set up an in-memory database seeded with SRD fixture data.
///
/// This includes:
/// - PHB catalog source
/// - All 12 SRD classes
/// - All 12 SRD subclasses
/// - 170 class features
/// - 89 subclass features
/// - 7 backgrounds
/// - 17 races
/// - ~44 items
/// - ~43 spells
/// - ~17 monsters
pub fn setup_srd_db() -> SqliteConnection {
    let mut conn = create_db();

    // The loader lives in the library (`fixtures` feature) so the UI bridge
    // seeds the same catalog.
    seed_srd_catalog(&mut conn).expect("Failed to seed SRD catalog fixtures");

    conn
}

/// Set up the SRD database plus additional sources for broader testing.
pub fn setup_srd_db_with_extra_sources() -> SqliteConnection {
    let mut conn = setup_srd_db();

    let extra_sources = [
        NewCatalogSource::new("MM", "Monster Manual", true, "2024-01-20T12:00:00Z"),
        NewCatalogSource::new(
            "DMG",
            "Dungeon Master's Guide",
            true,
            "2024-01-20T12:00:00Z",
        ),
        NewCatalogSource::new(
            "XGE",
            "Xanathar's Guide to Everything",
            true,
            "2024-01-20T12:00:00Z",
        ),
    ];
    for source in &extra_sources {
        catalog::insert_source(&mut conn, source).expect("Failed to insert source");
    }

    conn
}
