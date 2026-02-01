//! CampaignHomebrewItem Model
//!
//! Custom items created by DMs within a campaign, optionally cloned from catalog items.

use crate::schema::campaign_homebrew_items;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A homebrew item belonging to a campaign.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = campaign_homebrew_items)]
pub struct CampaignHomebrewItem {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Data for inserting a new homebrew item.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = campaign_homebrew_items)]
pub struct NewCampaignHomebrewItem<'a> {
    pub id: &'a str,
    pub campaign_id: &'a str,
    pub name: &'a str,
    pub item_type: Option<&'a str>,
    pub rarity: Option<&'a str>,
    pub data: &'a str,
    pub cloned_from_name: Option<&'a str>,
    pub cloned_from_source: Option<&'a str>,
}

impl<'a> NewCampaignHomebrewItem<'a> {
    pub fn new(id: &'a str, campaign_id: &'a str, name: &'a str, data: &'a str) -> Self {
        Self {
            id,
            campaign_id,
            name,
            item_type: None,
            rarity: None,
            data,
            cloned_from_name: None,
            cloned_from_source: None,
        }
    }

    pub fn with_item_type(mut self, item_type: &'a str) -> Self {
        self.item_type = Some(item_type);
        self
    }

    pub fn with_rarity(mut self, rarity: &'a str) -> Self {
        self.rarity = Some(rarity);
        self
    }

    pub fn cloned_from(mut self, name: &'a str, source: &'a str) -> Self {
        self.cloned_from_name = Some(name);
        self.cloned_from_source = Some(source);
        self
    }
}

/// Data for updating a homebrew item.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = campaign_homebrew_items)]
pub struct UpdateCampaignHomebrewItem<'a> {
    pub name: Option<&'a str>,
    pub item_type: Option<Option<&'a str>>,
    pub rarity: Option<Option<&'a str>>,
    pub data: Option<&'a str>,
    pub updated_at: Option<&'a str>,
}
