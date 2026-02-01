//! CampaignHomebrewSpell Data Access Layer
//!
//! Database operations for campaign homebrew spells.

use crate::models::campaign::{
    CampaignHomebrewSpell, NewCampaignHomebrewSpell, UpdateCampaignHomebrewSpell,
};
use crate::schema::campaign_homebrew_spells;
use diesel::prelude::*;
use diesel::SqliteConnection;

/// Insert a new homebrew spell.
pub fn insert_campaign_homebrew_spell(
    conn: &mut SqliteConnection,
    spell: &NewCampaignHomebrewSpell,
) -> QueryResult<String> {
    diesel::insert_into(campaign_homebrew_spells::table)
        .values(spell)
        .execute(conn)?;
    Ok(spell.id.to_string())
}

/// Get a homebrew spell by ID.
pub fn get_campaign_homebrew_spell(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<CampaignHomebrewSpell> {
    campaign_homebrew_spells::table.find(id).first(conn)
}

/// Get a homebrew spell by campaign_id and name.
pub fn get_campaign_homebrew_spell_by_name(
    conn: &mut SqliteConnection,
    campaign_id: &str,
    name: &str,
) -> QueryResult<Option<CampaignHomebrewSpell>> {
    campaign_homebrew_spells::table
        .filter(campaign_homebrew_spells::campaign_id.eq(campaign_id))
        .filter(campaign_homebrew_spells::name.eq(name))
        .first(conn)
        .optional()
}

/// List all homebrew spells for a campaign.
pub fn list_campaign_homebrew_spells(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<Vec<CampaignHomebrewSpell>> {
    campaign_homebrew_spells::table
        .filter(campaign_homebrew_spells::campaign_id.eq(campaign_id))
        .order(campaign_homebrew_spells::name.asc())
        .load(conn)
}

/// Update a homebrew spell.
pub fn update_campaign_homebrew_spell(
    conn: &mut SqliteConnection,
    id: &str,
    update: &UpdateCampaignHomebrewSpell,
) -> QueryResult<usize> {
    diesel::update(campaign_homebrew_spells::table.find(id))
        .set(update)
        .execute(conn)
}

/// Delete a homebrew spell by ID.
pub fn delete_campaign_homebrew_spell(
    conn: &mut SqliteConnection,
    id: &str,
) -> QueryResult<usize> {
    diesel::delete(campaign_homebrew_spells::table.find(id)).execute(conn)
}

/// Delete all homebrew spells for a campaign.
pub fn delete_campaign_homebrew_spells(
    conn: &mut SqliteConnection,
    campaign_id: &str,
) -> QueryResult<usize> {
    diesel::delete(
        campaign_homebrew_spells::table
            .filter(campaign_homebrew_spells::campaign_id.eq(campaign_id)),
    )
    .execute(conn)
}
