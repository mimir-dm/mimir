//! Campaign Source Data Access Layer
//!
//! Database operations for campaign sources (allowed books).

use crate::models::campaign::{CampaignSource, NewCampaignSource};
use crate::schema::campaign_sources;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new campaign source.
pub fn insert_campaign_source(
    conn: &mut SqliteConnection,
    source: &NewCampaignSource,
) -> QueryResult<String> {
    diesel::insert_into(campaign_sources::table)
        .values(source)
        .execute(conn)?;

    Ok(source.id.to_string())
}

/// Insert multiple campaign sources in a batch.
pub fn insert_campaign_sources(
    conn: &mut SqliteConnection,
    sources: &[NewCampaignSource],
) -> QueryResult<usize> {
    diesel::insert_into(campaign_sources::table)
        .values(sources)
        .execute(conn)
}

/// Get a campaign source by ID.
pub fn get_campaign_source(conn: &mut SqliteConnection, id: &str) -> QueryResult<CampaignSource> {
    campaign_sources::table.find(id).first(conn)
}

/// List all sources for a campaign.
pub fn list_campaign_sources(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<CampaignSource>> {
    campaign_sources::table
        .filter(campaign_sources::campaign_id.eq(campaign_id))
        .order(campaign_sources::source_code.asc())
        .load(conn)
}

/// List source codes for a campaign (just the codes, not full objects).
pub fn list_campaign_source_codes(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<String>> {
    campaign_sources::table
        .filter(campaign_sources::campaign_id.eq(campaign_id))
        .select(campaign_sources::source_code)
        .order(campaign_sources::source_code.asc())
        .load(conn)
}

/// Delete a campaign source by ID.
pub fn delete_campaign_source(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(campaign_sources::table.find(id)).execute(conn)
}

/// Delete a campaign source by campaign and source code.
pub fn delete_campaign_source_by_code(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    source_code: &str,
) -> QueryResult<usize> {
    diesel::delete(
        campaign_sources::table
            .filter(campaign_sources::campaign_id.eq(campaign_id))
            .filter(campaign_sources::source_code.eq(source_code)),
    )
    .execute(conn)
}

/// Delete all sources for a campaign.
pub fn delete_all_campaign_sources(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<usize> {
    diesel::delete(campaign_sources::table.filter(campaign_sources::campaign_id.eq(campaign_id)))
        .execute(conn)
}

/// Check if a campaign has a specific source enabled.
pub fn campaign_has_source(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    source_code: &str,
) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(
        campaign_sources::table
            .filter(campaign_sources::campaign_id.eq(campaign_id))
            .filter(campaign_sources::source_code.eq(source_code)),
    ))
    .get_result(conn)
}

/// Count sources for a campaign.
pub fn count_campaign_sources(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i64> {
    campaign_sources::table
        .filter(campaign_sources::campaign_id.eq(campaign_id))
        .count()
        .get_result(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

        conn.batch_execute(
            "CREATE TABLE campaigns (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                archived_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE catalog_sources (
                code TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                imported_at TEXT NOT NULL
            );
            CREATE TABLE campaign_sources (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
                source_code TEXT NOT NULL REFERENCES catalog_sources(code),
                UNIQUE(campaign_id, source_code)
            );
            CREATE INDEX idx_campaign_sources_campaign ON campaign_sources(campaign_id);

            -- Insert test data
            INSERT INTO campaigns (id, name) VALUES ('camp-1', 'Test Campaign');
            INSERT INTO catalog_sources (code, name, enabled, imported_at) VALUES
                ('PHB', 'Players Handbook', 1, '2024-01-01'),
                ('MM', 'Monster Manual', 1, '2024-01-01'),
                ('DMG', 'Dungeon Masters Guide', 1, '2024-01-01');
            PRAGMA foreign_keys = ON;",
        )
        .expect("Failed to create tables");

        conn
    }

    #[test]
    fn test_insert_and_get_campaign_source() {
        let mut conn = setup_test_db();

        let source = NewCampaignSource::new("src-1", "camp-1", "PHB");
        let id = insert_campaign_source(&mut conn, &source).expect("Failed to insert");
        assert_eq!(id, "src-1");

        let retrieved = get_campaign_source(&mut conn, "src-1").expect("Failed to get");
        assert_eq!(retrieved.id, "src-1");
        assert_eq!(retrieved.campaign_id, "camp-1");
        assert_eq!(retrieved.source_code, "PHB");
    }

    #[test]
    fn test_insert_multiple_campaign_sources() {
        let mut conn = setup_test_db();

        let sources = vec![
            NewCampaignSource::new("src-1", "camp-1", "PHB"),
            NewCampaignSource::new("src-2", "camp-1", "MM"),
            NewCampaignSource::new("src-3", "camp-1", "DMG"),
        ];

        let count = insert_campaign_sources(&mut conn, &sources).expect("Failed to insert");
        assert_eq!(count, 3);

        let all = list_campaign_sources(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_list_campaign_source_codes() {
        let mut conn = setup_test_db();

        let sources = vec![
            NewCampaignSource::new("src-1", "camp-1", "PHB"),
            NewCampaignSource::new("src-2", "camp-1", "MM"),
        ];
        insert_campaign_sources(&mut conn, &sources).expect("Failed to insert");

        let codes = list_campaign_source_codes(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(codes, vec!["MM", "PHB"]); // Sorted alphabetically
    }

    #[test]
    fn test_campaign_has_source() {
        let mut conn = setup_test_db();

        assert!(!campaign_has_source(&mut conn, "camp-1", "PHB").expect("Failed to check"));

        let source = NewCampaignSource::new("src-1", "camp-1", "PHB");
        insert_campaign_source(&mut conn, &source).expect("Failed to insert");

        assert!(campaign_has_source(&mut conn, "camp-1", "PHB").expect("Failed to check"));
        assert!(!campaign_has_source(&mut conn, "camp-1", "MM").expect("Failed to check"));
    }

    #[test]
    fn test_delete_campaign_source() {
        let mut conn = setup_test_db();

        let source = NewCampaignSource::new("src-1", "camp-1", "PHB");
        insert_campaign_source(&mut conn, &source).expect("Failed to insert");

        assert!(campaign_has_source(&mut conn, "camp-1", "PHB").expect("Failed to check"));

        delete_campaign_source(&mut conn, "src-1").expect("Failed to delete");

        assert!(!campaign_has_source(&mut conn, "camp-1", "PHB").expect("Failed to check"));
    }

    #[test]
    fn test_delete_campaign_source_by_code() {
        let mut conn = setup_test_db();

        let sources = vec![
            NewCampaignSource::new("src-1", "camp-1", "PHB"),
            NewCampaignSource::new("src-2", "camp-1", "MM"),
        ];
        insert_campaign_sources(&mut conn, &sources).expect("Failed to insert");

        delete_campaign_source_by_code(&mut conn, "camp-1", "PHB").expect("Failed to delete");

        assert!(!campaign_has_source(&mut conn, "camp-1", "PHB").expect("Failed to check"));
        assert!(campaign_has_source(&mut conn, "camp-1", "MM").expect("Failed to check"));
    }

    #[test]
    fn test_delete_all_campaign_sources() {
        let mut conn = setup_test_db();

        let sources = vec![
            NewCampaignSource::new("src-1", "camp-1", "PHB"),
            NewCampaignSource::new("src-2", "camp-1", "MM"),
        ];
        insert_campaign_sources(&mut conn, &sources).expect("Failed to insert");

        assert_eq!(
            count_campaign_sources(&mut conn, "camp-1").expect("Failed to count"),
            2
        );

        delete_all_campaign_sources(&mut conn, "camp-1").expect("Failed to delete");

        assert_eq!(
            count_campaign_sources(&mut conn, "camp-1").expect("Failed to count"),
            0
        );
    }

    #[test]
    fn test_count_campaign_sources() {
        let mut conn = setup_test_db();

        assert_eq!(
            count_campaign_sources(&mut conn, "camp-1").expect("Failed to count"),
            0
        );

        let sources = vec![
            NewCampaignSource::new("src-1", "camp-1", "PHB"),
            NewCampaignSource::new("src-2", "camp-1", "MM"),
        ];
        insert_campaign_sources(&mut conn, &sources).expect("Failed to insert");

        assert_eq!(
            count_campaign_sources(&mut conn, "camp-1").expect("Failed to count"),
            2
        );
    }
}
