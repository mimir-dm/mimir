//! Campaign database models and operations

use crate::schema::campaigns;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for campaigns
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = campaigns)]
pub struct Campaign {
    pub id: i32,
    pub name: String,
    pub status: String,
    pub directory_path: String,
    pub created_at: String,
    pub session_zero_date: Option<String>,
    pub first_session_date: Option<String>,
    pub last_activity_at: String,
    pub archived_at: Option<String>,
}

/// New campaign for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = campaigns)]
pub struct NewCampaign {
    pub name: String,
    pub status: String,
    pub directory_path: String,
}

/// Campaign update structure
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = campaigns)]
pub struct UpdateCampaign {
    pub name: Option<String>,
    pub status: Option<String>,
    pub directory_path: Option<String>,
    pub session_zero_date: Option<Option<String>>,
    pub first_session_date: Option<Option<String>>,
    pub last_activity_at: Option<String>,
    pub archived_at: Option<Option<String>>,
}

impl Campaign {
    // Transition validation is handled by BoardDefinition in the service layer
}
