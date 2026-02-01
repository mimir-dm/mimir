//! CampaignHomebrewItem Data Access Layer
//!
//! Database operations for campaign homebrew items.

use crate::models::campaign::{
    CampaignHomebrewItem, NewCampaignHomebrewItem, UpdateCampaignHomebrewItem,
};
use crate::schema::campaign_homebrew_items;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new homebrew item.
pub fn insert_campaign_homebrew_item(
    conn: &mut SqliteConnection,
    item: &NewCampaignHomebrewItem,
) -> QueryResult<String> {
    diesel::insert_into(campaign_homebrew_items::table)
        .values(item)
        .execute(conn)?;
    Ok(item.id.to_string())
}

/// Get a homebrew item by ID.
pub fn get_campaign_homebrew_item(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<CampaignHomebrewItem> {
    campaign_homebrew_items::table.find(id).first(conn)
}

/// Get a homebrew item by campaign_id and name.
pub fn get_campaign_homebrew_item_by_name(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    name: &str,
) -> QueryResult<Option<CampaignHomebrewItem>> {
    campaign_homebrew_items::table
        .filter(campaign_homebrew_items::campaign_id.eq(campaign_id))
        .filter(campaign_homebrew_items::name.eq(name))
        .first(conn)
        .optional()
}

/// List all homebrew items for a campaign.
pub fn list_campaign_homebrew_items(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<CampaignHomebrewItem>> {
    campaign_homebrew_items::table
        .filter(campaign_homebrew_items::campaign_id.eq(campaign_id))
        .order(campaign_homebrew_items::name.asc())
        .load(conn)
}

/// Update a homebrew item.
pub fn update_campaign_homebrew_item(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCampaignHomebrewItem,
) -> QueryResult<usize> {
    diesel::update(campaign_homebrew_items::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a homebrew item by ID.
pub fn delete_campaign_homebrew_item(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<usize> {
    diesel::delete(campaign_homebrew_items::table.find(id)).execute(conn)
}

/// Delete all homebrew items for a campaign.
pub fn delete_campaign_homebrew_items(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        campaign_homebrew_items::table
            .filter(campaign_homebrew_items::campaign_id.eq(campaign_id)),
    )
    .execute(conn)
}
