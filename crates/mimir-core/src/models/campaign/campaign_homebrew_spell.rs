//! CampaignHomebrewSpell Model
//!
//! Custom spells created by DMs within a campaign, optionally cloned from catalog spells.

use crate::schema::campaign_homebrew_spells;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A homebrew spell belonging to a campaign.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = campaign_homebrew_spells)]
pub struct CampaignHomebrewSpell {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub level: Option<i32>,
    pub school: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Data for inserting a new homebrew spell.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = campaign_homebrew_spells)]
pub struct NewCampaignHomebrewSpell<'a> {
    pub id: &'a str,
    pub campaign_id: &'a str,
    pub name: &'a str,
    pub level: Option<i32>,
    pub school: Option<&'a str>,
    pub data: &'a str,
    pub cloned_from_name: Option<&'a str>,
    pub cloned_from_source: Option<&'a str>,
}

impl<'a> NewCampaignHomebrewSpell<'a> {
    pub fn new(id: &'a str, campaign_id: &'a str, name: &'a str, data: &'a str) -> Self {
        Self {
            id,
            campaign_id,
            name,
            level: None,
            school: None,
            data,
            cloned_from_name: None,
            cloned_from_source: None,
        }
    }

    pub fn with_level(mut self, level: i32) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_school(mut self, school: &'a str) -> Self {
        self.school = Some(school);
        self
    }

    pub fn cloned_from(mut self, name: &'a str, source: &'a str) -> Self {
        self.cloned_from_name = Some(name);
        self.cloned_from_source = Some(source);
        self
    }
}

/// Data for updating a homebrew spell.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = campaign_homebrew_spells)]
pub struct UpdateCampaignHomebrewSpell<'a> {
    pub name: Option<&'a str>,
    pub level: Option<Option<i32>>,
    pub school: Option<Option<&'a str>>,
    pub data: Option<&'a str>,
    pub updated_at: Option<&'a str>,
}
