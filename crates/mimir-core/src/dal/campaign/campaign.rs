//! Campaign Data Access Layer
//!
//! Database operations for campaigns.

use crate::models::campaign::{Campaign, NewCampaign, UpdateCampaign};
use crate::schema::campaigns;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new campaign.
pub fn insert_campaign(conn: &mut SqliteConnection, campaign: &NewCampaign) -> QueryResult<String> {
    diesel::insert_into(campaigns::table)
        .values(campaign)
        .execute(conn)?;

    Ok(campaign.id.to_string())
}

/// Get a campaign by ID.
pub fn get_campaign(conn: &mut SqliteConnection, id: &str) -> QueryResult<Campaign> {
    campaigns::table.find(id).first(conn)
}

/// Get a campaign by ID, returning None if not found.
pub fn get_campaign_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<Campaign>> {
    campaigns::table.find(id).first(conn).optional()
}

/// List all campaigns, optionally including archived.
pub fn list_campaigns(
    conn: &mut SqliteConnection,
    include_archived: bool,
) -> QueryResult<Vec<Campaign>> {
    let mut query = campaigns::table.into_boxed();

    if !include_archived {
        query = query.filter(campaigns::archived_at.is_null());
    }

    query.order(campaigns::name.asc()).load(conn)
}

/// List only archived campaigns.
pub fn list_archived_campaigns(conn: &mut SqliteConnection) -> QueryResult<Vec<Campaign>> {
    campaigns::table
        .filter(campaigns::archived_at.is_not_null())
        .order(campaigns::name.asc())
        .load(conn)
}

/// Update a campaign.
pub fn update_campaign(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCampaign,
) -> QueryResult<usize> {
    diesel::update(campaigns::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a campaign by ID.
///
/// Note: This will cascade delete all related data (modules, sources, etc.).
pub fn delete_campaign(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(campaigns::table.find(id)).execute(conn)
}

/// Check if a campaign exists.
pub fn campaign_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(campaigns::table.find(id))).get_result(conn)
}

/// Count all campaigns (excluding archived).
pub fn count_campaigns(conn: &mut SqliteConnection) -> QueryResult<i64> {
    campaigns::table
        .filter(campaigns::archived_at.is_null())
        .count()
        .get_result(conn)
}

/// Count all campaigns including archived.
pub fn count_all_campaigns(conn: &mut SqliteConnection) -> QueryResult<i64> {
    campaigns::table.count().get_result(conn)
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
            );",
        )
        .expect("Failed to create table");

        conn
    }

    #[test]
    fn test_insert_and_get_campaign() {
        let mut conn = setup_test_db();

        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        let id = insert_campaign(&mut conn, &campaign).expect("Failed to insert");
        assert_eq!(id, "camp-1");

        let retrieved = get_campaign(&mut conn, "camp-1").expect("Failed to get");
        assert_eq!(retrieved.id, "camp-1");
        assert_eq!(retrieved.name, "Test Campaign");
        assert!(retrieved.description.is_none());
        assert!(retrieved.archived_at.is_none());
    }

    #[test]
    fn test_insert_campaign_with_description() {
        let mut conn = setup_test_db();

        let campaign =
            NewCampaign::new("camp-1", "Test Campaign").with_description("A great adventure");
        insert_campaign(&mut conn, &campaign).expect("Failed to insert");

        let retrieved = get_campaign(&mut conn, "camp-1").expect("Failed to get");
        assert_eq!(retrieved.description, Some("A great adventure".to_string()));
    }

    #[test]
    fn test_list_campaigns_excludes_archived() {
        let mut conn = setup_test_db();

        let campaign1 = NewCampaign::new("camp-1", "Active Campaign");
        let campaign2 = NewCampaign::new("camp-2", "Archived Campaign");
        insert_campaign(&mut conn, &campaign1).expect("Failed to insert");
        insert_campaign(&mut conn, &campaign2).expect("Failed to insert");

        // Archive the second campaign
        let update = UpdateCampaign::archive("2024-01-20T12:00:00Z");
        update_campaign(&mut conn, "camp-2", &update).expect("Failed to update");

        // List should only show active
        let active = list_campaigns(&mut conn, false).expect("Failed to list");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "camp-1");

        // List with include_archived should show both
        let all = list_campaigns(&mut conn, true).expect("Failed to list");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_list_archived_campaigns() {
        let mut conn = setup_test_db();

        let campaign1 = NewCampaign::new("camp-1", "Active Campaign");
        let campaign2 = NewCampaign::new("camp-2", "Archived Campaign");
        insert_campaign(&mut conn, &campaign1).expect("Failed to insert");
        insert_campaign(&mut conn, &campaign2).expect("Failed to insert");

        // Archive the second campaign
        let update = UpdateCampaign::archive("2024-01-20T12:00:00Z");
        update_campaign(&mut conn, "camp-2", &update).expect("Failed to update");

        let archived = list_archived_campaigns(&mut conn).expect("Failed to list");
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].id, "camp-2");
    }

    #[test]
    fn test_update_campaign_name() {
        let mut conn = setup_test_db();

        let campaign = NewCampaign::new("camp-1", "Original Name");
        insert_campaign(&mut conn, &campaign).expect("Failed to insert");

        let update = UpdateCampaign::set_name("New Name", "2024-01-20T12:00:00Z");
        update_campaign(&mut conn, "camp-1", &update).expect("Failed to update");

        let retrieved = get_campaign(&mut conn, "camp-1").expect("Failed to get");
        assert_eq!(retrieved.name, "New Name");
    }

    #[test]
    fn test_archive_and_unarchive_campaign() {
        let mut conn = setup_test_db();

        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(&mut conn, &campaign).expect("Failed to insert");

        // Archive
        let update = UpdateCampaign::archive("2024-01-20T12:00:00Z");
        update_campaign(&mut conn, "camp-1", &update).expect("Failed to update");

        let retrieved = get_campaign(&mut conn, "camp-1").expect("Failed to get");
        assert!(retrieved.is_archived());

        // Unarchive
        let update = UpdateCampaign::unarchive("2024-01-20T13:00:00Z");
        update_campaign(&mut conn, "camp-1", &update).expect("Failed to update");

        let retrieved = get_campaign(&mut conn, "camp-1").expect("Failed to get");
        assert!(!retrieved.is_archived());
    }

    #[test]
    fn test_delete_campaign() {
        let mut conn = setup_test_db();

        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(&mut conn, &campaign).expect("Failed to insert");

        assert!(campaign_exists(&mut conn, "camp-1").expect("Failed to check"));

        delete_campaign(&mut conn, "camp-1").expect("Failed to delete");

        assert!(!campaign_exists(&mut conn, "camp-1").expect("Failed to check"));
    }

    #[test]
    fn test_get_campaign_optional() {
        let mut conn = setup_test_db();

        let result = get_campaign_optional(&mut conn, "nonexistent").expect("Failed to query");
        assert!(result.is_none());

        let campaign = NewCampaign::new("camp-1", "Test Campaign");
        insert_campaign(&mut conn, &campaign).expect("Failed to insert");

        let result = get_campaign_optional(&mut conn, "camp-1").expect("Failed to query");
        assert!(result.is_some());
    }

    #[test]
    fn test_count_campaigns() {
        let mut conn = setup_test_db();

        assert_eq!(count_campaigns(&mut conn).expect("Failed to count"), 0);

        let campaign1 = NewCampaign::new("camp-1", "Campaign 1");
        let campaign2 = NewCampaign::new("camp-2", "Campaign 2");
        insert_campaign(&mut conn, &campaign1).expect("Failed to insert");
        insert_campaign(&mut conn, &campaign2).expect("Failed to insert");

        assert_eq!(count_campaigns(&mut conn).expect("Failed to count"), 2);

        // Archive one
        let update = UpdateCampaign::archive("2024-01-20T12:00:00Z");
        update_campaign(&mut conn, "camp-2", &update).expect("Failed to update");

        assert_eq!(count_campaigns(&mut conn).expect("Failed to count"), 1);
        assert_eq!(count_all_campaigns(&mut conn).expect("Failed to count"), 2);
    }
}
