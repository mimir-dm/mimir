//! Module database models and operations

use crate::schema::modules;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for modules
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = modules)]
#[diesel(belongs_to(crate::models::campaign::campaigns::Campaign))]
pub struct Module {
    pub id: i32,
    pub campaign_id: i32,
    pub name: String,
    pub module_number: i32,
    pub status: String,
    pub expected_sessions: i32,
    pub actual_sessions: i32,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// New module for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = modules)]
pub struct NewModule {
    pub campaign_id: i32,
    pub name: String,
    pub module_number: i32,
    pub status: String,
    pub expected_sessions: i32,
}

/// Module update structure
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = modules)]
pub struct UpdateModule {
    pub name: Option<String>,
    pub status: Option<String>,
    pub expected_sessions: Option<i32>,
    pub actual_sessions: Option<i32>,
    pub started_at: Option<Option<String>>,
    pub completed_at: Option<Option<String>>,
}

impl Module {
    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f32 {
        if self.expected_sessions == 0 {
            return 0.0;
        }
        (self.actual_sessions as f32 / self.expected_sessions as f32) * 100.0
    }

    /// Check if module has reached 60% completion (trigger for next module planning)
    pub fn should_trigger_next_module(&self) -> bool {
        self.completion_percentage() >= 60.0
    }

    // Transition validation is handled by BoardDefinition in the service layer
}
