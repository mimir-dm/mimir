//! Template seeding functionality - wrapper around mimir-dm-core's template seeder

use anyhow::Result;
use mimir_dm_core::DbConnection;

/// Seed the database with initial templates if they don't exist
/// Returns the number of templates added
pub fn seed_templates(conn: &mut DbConnection) -> Result<usize> {
    // Use the seeder from mimir-dm-core
    match mimir_dm_core::seed::template_seeder::seed_templates(conn) {
        Ok(count) => Ok(count),
        Err(e) => match e {
            diesel::result::Error::QueryBuilderError(ref err) => {
                anyhow::bail!("Template file error: {}", err);
            }
            diesel::result::Error::DatabaseError(kind, info) => {
                anyhow::bail!("Database error during seeding: {:?} - {:?}", kind, info);
            }
            _ => {
                anyhow::bail!("Failed to seed templates: {}", e);
            }
        },
    }
}
