//! Campaign Asset Data Access Layer
//!
//! Database operations for campaign assets (user-uploaded files).

use crate::models::campaign::{CampaignAsset, NewCampaignAsset};
use crate::schema::campaign_assets;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new campaign asset.
pub fn insert_campaign_asset(
    conn: &mut SqliteConnection,
    asset: &NewCampaignAsset,
) -> QueryResult<String> {
    diesel::insert_into(campaign_assets::table)
        .values(asset)
        .execute(conn)?;

    Ok(asset.id.to_string())
}

/// Get a campaign asset by ID.
pub fn get_campaign_asset(conn: &mut SqliteConnection, id: &str) -> QueryResult<CampaignAsset> {
    campaign_assets::table.find(id).first(conn)
}

/// Get a campaign asset by ID, returning None if not found.
pub fn get_campaign_asset_optional(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<Option<CampaignAsset>> {
    campaign_assets::table.find(id).first(conn).optional()
}

/// List all assets for a campaign.
pub fn list_campaign_assets(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<CampaignAsset>> {
    campaign_assets::table
        .filter(campaign_assets::campaign_id.eq(campaign_id))
        .order(campaign_assets::filename.asc())
        .load(conn)
}

/// List all assets for a module.
pub fn list_module_assets(
    conn: &mut SqliteConnection,
    module_id: &str,
) -> QueryResult<Vec<CampaignAsset>> {
    campaign_assets::table
        .filter(campaign_assets::module_id.eq(module_id))
        .order(campaign_assets::filename.asc())
        .load(conn)
}

/// List assets by MIME type for a campaign.
pub fn list_campaign_assets_by_mime(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    mime_type: &str,
) -> QueryResult<Vec<CampaignAsset>> {
    campaign_assets::table
        .filter(campaign_assets::campaign_id.eq(campaign_id))
        .filter(campaign_assets::mime_type.eq(mime_type))
        .order(campaign_assets::filename.asc())
        .load(conn)
}

/// List assets by MIME type for a module.
pub fn list_module_assets_by_mime(
    conn: &mut SqliteConnection,
    module_id: &str,
    mime_type: &str,
) -> QueryResult<Vec<CampaignAsset>> {
    campaign_assets::table
        .filter(campaign_assets::module_id.eq(module_id))
        .filter(campaign_assets::mime_type.eq(mime_type))
        .order(campaign_assets::filename.asc())
        .load(conn)
}

/// Delete a campaign asset by ID.
///
/// Note: This only deletes the database record, not the file.
/// Use the asset storage utilities to delete the actual file.
pub fn delete_campaign_asset(conn: &mut SqliteConnection, id: &str) -> QueryResult<usize> {
    diesel::delete(campaign_assets::table.find(id)).execute(conn)
}

/// Check if a campaign asset exists.
pub fn campaign_asset_exists(conn: &mut SqliteConnection, id: &str) -> QueryResult<bool> {
    use diesel::dsl::exists;
    use diesel::select;

    select(exists(campaign_assets::table.find(id))).get_result(conn)
}

/// Count assets for a campaign.
pub fn count_campaign_assets(conn: &mut SqliteConnection, campaign_id: &str) -> QueryResult<i64> {
    campaign_assets::table
        .filter(campaign_assets::campaign_id.eq(campaign_id))
        .count()
        .get_result(conn)
}

/// Count assets for a module.
pub fn count_module_assets(conn: &mut SqliteConnection, module_id: &str) -> QueryResult<i64> {
    campaign_assets::table
        .filter(campaign_assets::module_id.eq(module_id))
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
            CREATE TABLE modules (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT,
                module_number INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(campaign_id, module_number)
            );
            CREATE TABLE campaign_assets (
                id TEXT PRIMARY KEY NOT NULL,
                campaign_id TEXT REFERENCES campaigns(id) ON DELETE CASCADE,
                module_id TEXT REFERENCES modules(id) ON DELETE CASCADE,
                filename TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                blob_path TEXT NOT NULL,
                file_size INTEGER,
                uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
                CHECK (
                    (campaign_id IS NOT NULL AND module_id IS NULL) OR
                    (campaign_id IS NULL AND module_id IS NOT NULL)
                )
            );
            CREATE INDEX idx_campaign_assets_campaign ON campaign_assets(campaign_id);
            CREATE INDEX idx_campaign_assets_module ON campaign_assets(module_id);

            -- Insert test data
            INSERT INTO campaigns (id, name) VALUES ('camp-1', 'Test Campaign');
            INSERT INTO modules (id, campaign_id, name, module_number) VALUES ('mod-1', 'camp-1', 'Chapter 1', 1);
            PRAGMA foreign_keys = ON;",
        )
        .expect("Failed to create tables");

        conn
    }

    #[test]
    fn test_insert_and_get_campaign_asset() {
        let mut conn = setup_test_db();

        let asset = NewCampaignAsset::for_campaign(
            "asset-1",
            "camp-1",
            "treasure_map.png",
            "image/png",
            "assets/camp-1/asset-1.png",
        );
        let id = insert_campaign_asset(&mut conn, &asset).expect("Failed to insert");
        assert_eq!(id, "asset-1");

        let retrieved = get_campaign_asset(&mut conn, "asset-1").expect("Failed to get");
        assert_eq!(retrieved.id, "asset-1");
        assert_eq!(retrieved.campaign_id, Some("camp-1".to_string()));
        assert!(retrieved.module_id.is_none());
        assert_eq!(retrieved.filename, "treasure_map.png");
        assert_eq!(retrieved.mime_type, "image/png");
    }

    #[test]
    fn test_insert_module_asset() {
        let mut conn = setup_test_db();

        let asset = NewCampaignAsset::for_module(
            "asset-1",
            "mod-1",
            "dungeon.dd2vtt",
            "application/octet-stream",
            "assets/mod-1/asset-1.dd2vtt",
        );
        insert_campaign_asset(&mut conn, &asset).expect("Failed to insert");

        let retrieved = get_campaign_asset(&mut conn, "asset-1").expect("Failed to get");
        assert!(retrieved.campaign_id.is_none());
        assert_eq!(retrieved.module_id, Some("mod-1".to_string()));
    }

    #[test]
    fn test_list_campaign_assets() {
        let mut conn = setup_test_db();

        let asset1 = NewCampaignAsset::for_campaign(
            "asset-1",
            "camp-1",
            "a_map.png",
            "image/png",
            "assets/camp-1/asset-1.png",
        );
        let asset2 = NewCampaignAsset::for_campaign(
            "asset-2",
            "camp-1",
            "b_image.jpg",
            "image/jpeg",
            "assets/camp-1/asset-2.jpg",
        );
        insert_campaign_asset(&mut conn, &asset1).expect("Failed to insert");
        insert_campaign_asset(&mut conn, &asset2).expect("Failed to insert");

        let assets = list_campaign_assets(&mut conn, "camp-1").expect("Failed to list");
        assert_eq!(assets.len(), 2);
        // Sorted by filename
        assert_eq!(assets[0].filename, "a_map.png");
        assert_eq!(assets[1].filename, "b_image.jpg");
    }

    #[test]
    fn test_list_module_assets() {
        let mut conn = setup_test_db();

        let asset1 = NewCampaignAsset::for_module(
            "asset-1",
            "mod-1",
            "map1.dd2vtt",
            "application/octet-stream",
            "assets/mod-1/asset-1.dd2vtt",
        );
        let asset2 = NewCampaignAsset::for_module(
            "asset-2",
            "mod-1",
            "map2.dd2vtt",
            "application/octet-stream",
            "assets/mod-1/asset-2.dd2vtt",
        );
        insert_campaign_asset(&mut conn, &asset1).expect("Failed to insert");
        insert_campaign_asset(&mut conn, &asset2).expect("Failed to insert");

        let assets = list_module_assets(&mut conn, "mod-1").expect("Failed to list");
        assert_eq!(assets.len(), 2);
    }

    #[test]
    fn test_list_assets_by_mime_type() {
        let mut conn = setup_test_db();

        let asset1 = NewCampaignAsset::for_campaign(
            "asset-1",
            "camp-1",
            "image.png",
            "image/png",
            "assets/camp-1/asset-1.png",
        );
        let asset2 = NewCampaignAsset::for_campaign(
            "asset-2",
            "camp-1",
            "photo.jpg",
            "image/jpeg",
            "assets/camp-1/asset-2.jpg",
        );
        insert_campaign_asset(&mut conn, &asset1).expect("Failed to insert");
        insert_campaign_asset(&mut conn, &asset2).expect("Failed to insert");

        let png_assets =
            list_campaign_assets_by_mime(&mut conn, "camp-1", "image/png").expect("Failed to list");
        assert_eq!(png_assets.len(), 1);
        assert_eq!(png_assets[0].filename, "image.png");
    }

    #[test]
    fn test_delete_campaign_asset() {
        let mut conn = setup_test_db();

        let asset = NewCampaignAsset::for_campaign(
            "asset-1",
            "camp-1",
            "map.png",
            "image/png",
            "assets/camp-1/asset-1.png",
        );
        insert_campaign_asset(&mut conn, &asset).expect("Failed to insert");

        assert!(campaign_asset_exists(&mut conn, "asset-1").expect("Failed to check"));

        delete_campaign_asset(&mut conn, "asset-1").expect("Failed to delete");

        assert!(!campaign_asset_exists(&mut conn, "asset-1").expect("Failed to check"));
    }

    #[test]
    fn test_get_campaign_asset_optional() {
        let mut conn = setup_test_db();

        let result = get_campaign_asset_optional(&mut conn, "nonexistent").expect("Failed to query");
        assert!(result.is_none());

        let asset = NewCampaignAsset::for_campaign(
            "asset-1",
            "camp-1",
            "map.png",
            "image/png",
            "assets/camp-1/asset-1.png",
        );
        insert_campaign_asset(&mut conn, &asset).expect("Failed to insert");

        let result = get_campaign_asset_optional(&mut conn, "asset-1").expect("Failed to query");
        assert!(result.is_some());
    }

    #[test]
    fn test_count_campaign_assets() {
        let mut conn = setup_test_db();

        assert_eq!(
            count_campaign_assets(&mut conn, "camp-1").expect("Failed to count"),
            0
        );

        let asset1 = NewCampaignAsset::for_campaign(
            "asset-1",
            "camp-1",
            "map1.png",
            "image/png",
            "assets/camp-1/asset-1.png",
        );
        let asset2 = NewCampaignAsset::for_campaign(
            "asset-2",
            "camp-1",
            "map2.png",
            "image/png",
            "assets/camp-1/asset-2.png",
        );
        insert_campaign_asset(&mut conn, &asset1).expect("Failed to insert");
        insert_campaign_asset(&mut conn, &asset2).expect("Failed to insert");

        assert_eq!(
            count_campaign_assets(&mut conn, "camp-1").expect("Failed to count"),
            2
        );
    }

    #[test]
    fn test_count_module_assets() {
        let mut conn = setup_test_db();

        assert_eq!(
            count_module_assets(&mut conn, "mod-1").expect("Failed to count"),
            0
        );

        let asset = NewCampaignAsset::for_module(
            "asset-1",
            "mod-1",
            "map.dd2vtt",
            "application/octet-stream",
            "assets/mod-1/asset-1.dd2vtt",
        );
        insert_campaign_asset(&mut conn, &asset).expect("Failed to insert");

        assert_eq!(
            count_module_assets(&mut conn, "mod-1").expect("Failed to count"),
            1
        );
    }
}
