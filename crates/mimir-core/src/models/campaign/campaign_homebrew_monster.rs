//! CampaignHomebrewMonster Model
//!
//! Custom monsters created by DMs within a campaign, optionally cloned from catalog monsters.

use crate::schema::campaign_homebrew_monsters;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use ts_rs::TS;

/// A homebrew monster belonging to a campaign.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "bindings/"))]
#[diesel(table_name = campaign_homebrew_monsters)]
pub struct CampaignHomebrewMonster {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub cr: Option<String>,
    pub creature_type: Option<String>,
    pub size: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Data for inserting a new homebrew monster.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = campaign_homebrew_monsters)]
pub struct NewCampaignHomebrewMonster<'a> {
    pub id: &'a str,
    pub campaign_id: &'a str,
    pub name: &'a str,
    pub cr: Option<&'a str>,
    pub creature_type: Option<&'a str>,
    pub size: Option<&'a str>,
    pub data: &'a str,
    pub cloned_from_name: Option<&'a str>,
    pub cloned_from_source: Option<&'a str>,
}

impl<'a> NewCampaignHomebrewMonster<'a> {
    pub fn new(id: &'a str, campaign_id: &'a str, name: &'a str, data: &'a str) -> Self {
        Self {
            id,
            campaign_id,
            name,
            cr: None,
            creature_type: None,
            size: None,
            data,
            cloned_from_name: None,
            cloned_from_source: None,
        }
    }

    pub fn with_cr(mut self, cr: &'a str) -> Self {
        self.cr = Some(cr);
        self
    }

    pub fn with_creature_type(mut self, creature_type: &'a str) -> Self {
        self.creature_type = Some(creature_type);
        self
    }

    pub fn with_size(mut self, size: &'a str) -> Self {
        self.size = Some(size);
        self
    }

    pub fn cloned_from(mut self, name: &'a str, source: &'a str) -> Self {
        self.cloned_from_name = Some(name);
        self.cloned_from_source = Some(source);
        self
    }
}

/// Data for updating a homebrew monster.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = campaign_homebrew_monsters)]
pub struct UpdateCampaignHomebrewMonster<'a> {
    pub name: Option<&'a str>,
    pub cr: Option<Option<&'a str>>,
    pub creature_type: Option<Option<&'a str>>,
    pub size: Option<Option<&'a str>>,
    pub data: Option<&'a str>,
    pub updated_at: Option<&'a str>,
}
