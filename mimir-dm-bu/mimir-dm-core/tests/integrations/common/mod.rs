//! Common test utilities
//!
//! Provides reusable fixtures and helpers for integration tests.

use diesel::prelude::*;
use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
use mimir_dm_core::error::Result;
use mimir_dm_core::models::campaign::campaigns::{Campaign, NewCampaign};
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::{NamedTempFile, TempDir};

/// Test database fixture that handles setup and cleanup
pub struct TestDatabase {
    _temp_dir: Option<TempDir>,
    _temp_file: Option<NamedTempFile>,
    pub url: String,
}

impl TestDatabase {
    /// Create a file-based test database (for async tests or when persistence is needed)
    pub fn file_based() -> Result<Self> {
        let temp_file = NamedTempFile::new()?;
        let url = temp_file.path().to_string_lossy().to_string();

        // Set up the database
        let mut conn = establish_connection(&url)?;
        Self::setup_database(&mut conn)?;
        drop(conn); // Close connection

        Ok(Self {
            _temp_dir: None,
            _temp_file: Some(temp_file),
            url,
        })
    }

    /// Get a new connection to this test database
    pub fn connection(&self) -> Result<SqliteConnection> {
        establish_connection(&self.url)
    }

    /// Set up the database with migrations and test settings
    fn setup_database(conn: &mut SqliteConnection) -> Result<()> {
        // Run migrations
        run_migrations(conn)?;

        // Enable foreign keys
        diesel::sql_query("PRAGMA foreign_keys = ON").execute(conn)?;

        // Optimize for tests
        diesel::sql_query("PRAGMA synchronous = OFF").execute(conn)?;

        diesel::sql_query("PRAGMA journal_mode = MEMORY").execute(conn)?;

        Ok(())
    }
}

/// Create a test campaign with sensible defaults.
///
/// # Arguments
/// * `conn` - Database connection
/// * `name` - Campaign name
/// * `directory` - Campaign directory path
///
/// # Returns
/// The created campaign
pub fn create_test_campaign(
    conn: &mut SqliteConnection,
    name: &str,
    directory: &str,
) -> Campaign {
    let mut repo = CampaignRepository::new(conn);
    repo.create(NewCampaign {
        name: name.to_string(),
        status: "concept".to_string(),
        directory_path: directory.to_string(),
    })
    .expect("Failed to create test campaign")
}

/// Create a test campaign with a unique name and temp directory.
///
/// # Arguments
/// * `conn` - Database connection
/// * `temp_dir` - Temporary directory for campaign files
///
/// # Returns
/// The created campaign
pub fn create_test_campaign_in_dir(
    conn: &mut SqliteConnection,
    temp_dir: &TempDir,
) -> Campaign {
    create_test_campaign(
        conn,
        "Test Campaign",
        temp_dir.path().to_str().expect("Invalid temp path"),
    )
}
