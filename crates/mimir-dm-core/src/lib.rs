//! Mimir Core Business Logic
//!
//! This crate provides the core domain models, business logic, and data persistence
//! for the Mimir D&D Campaign Assistant. It includes both the rules reference system
//! (D&D 5e data) and the campaign management system (modules, sessions, documents).

#![warn(missing_docs)]

pub mod connection;
pub mod dal;
pub mod db;
pub mod domain;
pub mod error;
pub mod models;
pub mod paths;
pub mod schema;
pub mod seed;
pub mod services;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

// Re-export commonly used types
pub use connection::establish_connection;
pub use db::{DatabaseService, DbConnection, DbPool};
pub use error::{BoxedError, DbError, MimirError, MimirResult, Result};
pub use paths::AppPaths;

// Re-export campaign models
pub use models::campaign::{Campaign, Document, Module, TemplateDocument, WorkflowCard};

// Re-export DAL traits
pub use dal::traits::Repository;

/// Embedded database migrations for automatic schema updates.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Run all pending migrations on the database
pub fn run_migrations(conn: &mut diesel::SqliteConnection) -> Result<()> {
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| DbError::Migration(format!("Failed to run migrations: {}", e)))?;
    Ok(())
}
