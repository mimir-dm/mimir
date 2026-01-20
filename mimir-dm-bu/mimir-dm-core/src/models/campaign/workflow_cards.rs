//! Workflow card database models and operations

use crate::schema::{workflow_card_tags, workflow_cards};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Database model for workflow cards
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = workflow_cards)]
#[diesel(belongs_to(crate::models::campaign::campaigns::Campaign))]
#[diesel(belongs_to(crate::models::campaign::modules::Module))]
pub struct WorkflowCard {
    pub id: String,
    pub board_type: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub last_moved_at: String,
    pub workflow_state: String,
    pub campaign_id: Option<i32>,
    pub module_id: Option<i32>,
    pub session_id: Option<i32>,
    pub priority: i32,
}

/// New workflow card for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = workflow_cards)]
pub struct NewWorkflowCard {
    pub id: String,
    pub board_type: String,
    pub title: String,
    pub description: Option<String>,
    pub workflow_state: String,
    pub campaign_id: Option<i32>,
    pub module_id: Option<i32>,
    pub session_id: Option<i32>,
    pub priority: i32,
}

/// Workflow card update structure
#[derive(Debug, Clone, Default, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = workflow_cards)]
pub struct UpdateWorkflowCard {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub workflow_state: Option<String>,
    pub last_moved_at: Option<String>,
    pub priority: Option<i32>,
}

/// Database model for workflow card tags
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Associations)]
#[diesel(table_name = workflow_card_tags)]
#[diesel(belongs_to(WorkflowCard, foreign_key = card_id))]
#[diesel(primary_key(card_id, tag))]
pub struct WorkflowCardTag {
    pub card_id: String,
    pub tag: String,
}

/// New tag for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = workflow_card_tags)]
pub struct NewWorkflowCardTag {
    pub card_id: String,
    pub tag: String,
}

impl WorkflowCard {
    // Transition validation is handled by BoardDefinition in the service layer
}
