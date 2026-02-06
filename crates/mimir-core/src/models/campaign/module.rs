//! Module Model
//!
//! Organizational container for adventure chapters within a campaign.

use crate::schema::modules;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use ts_rs::TS;

/// A module - an adventure chapter container within a campaign.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export, export_to = "bindings/"))]
#[diesel(table_name = modules)]
pub struct Module {
    /// Unique module ID (UUID)
    pub id: String,
    /// Campaign this module belongs to
    pub campaign_id: String,
    /// Module name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Module ordering number within the campaign
    pub module_number: i32,
    /// ISO8601 timestamp of creation
    pub created_at: String,
    /// ISO8601 timestamp of last update
    pub updated_at: String,
}

/// Data for inserting a new module.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = modules)]
pub struct NewModule<'a> {
    pub id: &'a str,
    pub campaign_id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub module_number: i32,
}

impl<'a> NewModule<'a> {
    /// Create a new module.
    pub fn new(id: &'a str, campaign_id: &'a str, name: &'a str, module_number: i32) -> Self {
        Self {
            id,
            campaign_id,
            name,
            description: None,
            module_number,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }
}

/// Data for updating a module.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = modules)]
pub struct UpdateModule<'a> {
    pub name: Option<&'a str>,
    pub description: Option<Option<&'a str>>,
    pub module_number: Option<i32>,
    pub updated_at: Option<&'a str>,
}

impl<'a> UpdateModule<'a> {
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

    /// Create an update to change the module number.
    pub fn set_module_number(module_number: i32, updated_at: &'a str) -> Self {
        Self {
            module_number: Some(module_number),
            updated_at: Some(updated_at),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_module() {
        let module = NewModule::new("mod-id", "campaign-id", "Chapter 1", 1);
        assert_eq!(module.id, "mod-id");
        assert_eq!(module.campaign_id, "campaign-id");
        assert_eq!(module.name, "Chapter 1");
        assert_eq!(module.module_number, 1);
        assert!(module.description.is_none());
    }

    #[test]
    fn test_new_module_with_description() {
        let module = NewModule::new("mod-id", "campaign-id", "Chapter 1", 1)
            .with_description("The beginning of the adventure");
        assert_eq!(module.description, Some("The beginning of the adventure"));
    }

    #[test]
    fn test_update_module_name() {
        let update = UpdateModule::set_name("New Chapter Name", "2024-01-20T12:00:00Z");
        assert_eq!(update.name, Some("New Chapter Name"));
        assert!(update.updated_at.is_some());
    }

    #[test]
    fn test_update_module_number() {
        let update = UpdateModule::set_module_number(5, "2024-01-20T12:00:00Z");
        assert_eq!(update.module_number, Some(5));
    }
}
