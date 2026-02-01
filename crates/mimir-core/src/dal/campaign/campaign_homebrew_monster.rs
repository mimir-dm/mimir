//! CampaignHomebrewMonster Data Access Layer
//!
//! Database operations for campaign homebrew monsters.

use crate::models::campaign::{
    CampaignHomebrewMonster, NewCampaignHomebrewMonster, UpdateCampaignHomebrewMonster,
};
use crate::schema::campaign_homebrew_monsters;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new homebrew monster.
pub fn insert_campaign_homebrew_monster(
    conn: &mut SqliteConnection,
    monster: &NewCampaignHomebrewMonster,
) -> QueryResult<String> {
    diesel::insert_into(campaign_homebrew_monsters::table)
        .values(monster)
        .execute(conn)?;
    Ok(monster.id.to_string())
}

/// Get a homebrew monster by ID.
pub fn get_campaign_homebrew_monster(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<CampaignHomebrewMonster> {
    campaign_homebrew_monsters::table.find(id).first(conn)
}

/// Get a homebrew monster by campaign_id and name.
pub fn get_campaign_homebrew_monster_by_name(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    name: &str,
) -> QueryResult<Option<CampaignHomebrewMonster>> {
    campaign_homebrew_monsters::table
        .filter(campaign_homebrew_monsters::campaign_id.eq(campaign_id))
        .filter(campaign_homebrew_monsters::name.eq(name))
        .first(conn)
        .optional()
}

/// List all homebrew monsters for a campaign.
pub fn list_campaign_homebrew_monsters(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<CampaignHomebrewMonster>> {
    campaign_homebrew_monsters::table
        .filter(campaign_homebrew_monsters::campaign_id.eq(campaign_id))
        .order(campaign_homebrew_monsters::name.asc())
        .load(conn)
}

/// Update a homebrew monster.
pub fn update_campaign_homebrew_monster(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCampaignHomebrewMonster,
) -> QueryResult<usize> {
    diesel::update(campaign_homebrew_monsters::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a homebrew monster by ID.
pub fn delete_campaign_homebrew_monster(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<usize> {
    diesel::delete(campaign_homebrew_monsters::table.find(id)).execute(conn)
}

/// Delete all homebrew monsters for a campaign.
pub fn delete_campaign_homebrew_monsters(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        campaign_homebrew_monsters::table
            .filter(campaign_homebrew_monsters::campaign_id.eq(campaign_id)),
    )
    .execute(conn)
}
