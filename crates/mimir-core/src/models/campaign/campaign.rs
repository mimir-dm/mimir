//! Campaign Model
//!
//! Top-level container for all campaign data.

use crate::schema::campaigns;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use ts_rs::TS;

/// A campaign - the top-level container for D&D campaign data.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "bindings/"))]
#[diesel(table_name = campaigns)]
pub struct Campaign {
    /// Unique campaign ID (UUID)
    pub id: String,
    /// Campaign name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// ISO8601 timestamp when archived, NULL means active
    pub archived_at: Option<String>,
    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

impl Campaign {
    /// Check if the campaign is archived.
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

/// Data for inserting a new campaign.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = campaigns)]
pub struct NewCampaign<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> NewCampaign<'a> {
    /// Create a new campaign.
    pub fn new(id: &'a str, name: &'a str) -> Self {
        Self {
            id,
            name,
            description: None,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
}

/// Data for updating a campaign.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = campaigns)]
pub struct UpdateCampaign<'a> {
    pub name: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub archived_at: Option<Option<&'a str>>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateCampaign<'a> {
    /// Create an update to change the name.
    pub fn set_name(name: &'a str, updated_at: &'a str) -> Self {
        Self {
            name: Some(name),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Create an update to change the description.
    pub fn set_description(description: Option<&'a str>, updated_at: &'a str) -> Self {
        Self {
            description: Some(description),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }

    /// Create an update to archive the campaign.
    pub fn archive(archived_at: &'a str) -> Self {
        Self {
            archived_at: Some(Some(archived_at)),
            updated_at: Some(archived_at),
            ..Default::default()
        }
    }

    /// Create an update to unarchive the campaign.
    pub fn unarchive(updated_at: &'a str) -> Self {
        Self {
            archived_at: Some(None),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_campaign() {
        let campaign = NewCampaign::new("test-id", "Test Campaign");
        assert_eq!(campaign.id, "test-id");
        assert_eq!(campaign.name, "Test Campaign");
        assert!(campaign.description.is_none());
    }

    #[test]
    fn test_new_campaign_with_description() {
        let campaign = NewCampaign::new("test-id", "Test Campaign")
            .with_description("A test description");
        assert_eq!(campaign.description, Some("A test description"));
    }

    #[test]
    fn test_update_campaign_name() {
        let update = UpdateCampaign::set_name("New Name", "2024-01-20T12:00:00Z");
        assert_eq!(update.name, Some("New Name"));
        assert!(update.updated_at.is_some());
    }

    #[test]
    fn test_update_campaign_archive() {
        let update = UpdateCampaign::archive("2024-01-20T12:00:00Z");
        assert_eq!(update.archived_at, Some(Some("2024-01-20T12:00:00Z")));
    }

    #[test]
    fn test_update_campaign_unarchive() {
        let update = UpdateCampaign::unarchive("2024-01-20T12:00:00Z");
        assert_eq!(update.archived_at, Some(None));
    }
}
