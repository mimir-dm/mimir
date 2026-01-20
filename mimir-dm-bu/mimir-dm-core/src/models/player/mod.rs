//! Player models
//!
//! Models for managing players and their association with campaigns.

use crate::schema::{campaign_players, players};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for players
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = players)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

/// New player for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = players)]
pub struct NewPlayer {
    pub name: String,
    pub email: Option<String>,
    pub notes: Option<String>,
}

/// Player update structure
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = players)]
pub struct UpdatePlayer {
    pub name: Option<String>,
    pub email: Option<Option<String>>,
    pub notes: Option<Option<String>>,
}

/// Database model for campaign-player associations
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = campaign_players)]
#[diesel(belongs_to(crate::models::campaign::Campaign, foreign_key = campaign_id))]
#[diesel(belongs_to(Player, foreign_key = player_id))]
pub struct CampaignPlayer {
    pub id: i32,
    pub campaign_id: i32,
    pub player_id: i32,
    pub joined_at: String,
    pub active: bool,
}

/// New campaign-player association for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = campaign_players)]
pub struct NewCampaignPlayer {
    pub campaign_id: i32,
    pub player_id: i32,
}

/// Campaign-player update structure
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = campaign_players)]
pub struct UpdateCampaignPlayer {
    pub active: Option<bool>,
}
