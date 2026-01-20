//! Workflow card data access layer

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::workflow_cards::{
    NewWorkflowCard, NewWorkflowCardTag, UpdateWorkflowCard, WorkflowCard,
};
use crate::schema::{workflow_card_tags, workflow_cards};
use chrono::Utc;
use diesel::prelude::*;

/// Repository for workflow card operations
pub struct WorkflowCardRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> WorkflowCardRepository<'a> {
    /// Create a new workflow card repository
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new workflow card
    pub fn create(&mut self, mut new_card: NewWorkflowCard) -> Result<WorkflowCard> {
        // Generate UUID if not provided
        if new_card.id.is_empty() {
            new_card.id = uuid::Uuid::new_v4().to_string();
        }

        diesel::insert_into(workflow_cards::table)
            .values(&new_card)
            .returning(WorkflowCard::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a workflow card by ID
    pub fn find_by_id(&mut self, id: &str) -> Result<Option<WorkflowCard>> {
        workflow_cards::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Update a workflow card
    pub fn update(&mut self, id: &str, update: UpdateWorkflowCard) -> Result<WorkflowCard> {
        diesel::update(workflow_cards::table.find(id))
            .set(&update)
            .returning(WorkflowCard::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Move a card to a new workflow state
    pub fn move_to_state(&mut self, id: &str, new_state: &str) -> Result<WorkflowCard> {
        // Transition validation is handled by BoardDefinition in the service layer

        let update = UpdateWorkflowCard {
            workflow_state: Some(new_state.to_string()),
            last_moved_at: Some(Utc::now().to_rfc3339()),
            ..Default::default()
        };

        self.update(id, update)
    }

    /// Delete a workflow card
    pub fn delete(&mut self, id: &str) -> Result<()> {
        diesel::delete(workflow_cards::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// List cards by board type
    pub fn list_by_board(&mut self, board_type: &str) -> Result<Vec<WorkflowCard>> {
        workflow_cards::table
            .filter(workflow_cards::board_type.eq(board_type))
            .order_by((workflow_cards::priority, workflow_cards::created_at))
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List cards by board type and state
    pub fn list_by_board_and_state(
        &mut self,
        board_type: &str,
        state: &str,
    ) -> Result<Vec<WorkflowCard>> {
        workflow_cards::table
            .filter(workflow_cards::board_type.eq(board_type))
            .filter(workflow_cards::workflow_state.eq(state))
            .order_by((workflow_cards::priority, workflow_cards::created_at))
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List cards for a specific campaign
    pub fn list_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<WorkflowCard>> {
        workflow_cards::table
            .filter(workflow_cards::campaign_id.eq(campaign_id))
            .order_by((workflow_cards::priority, workflow_cards::created_at))
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List cards for a specific module
    pub fn list_by_module(&mut self, module_id: i32) -> Result<Vec<WorkflowCard>> {
        workflow_cards::table
            .filter(workflow_cards::module_id.eq(module_id))
            .order_by((workflow_cards::priority, workflow_cards::created_at))
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List cards for a specific session
    pub fn list_by_session(&mut self, session_id: i32) -> Result<Vec<WorkflowCard>> {
        workflow_cards::table
            .filter(workflow_cards::session_id.eq(session_id))
            .order_by((workflow_cards::priority, workflow_cards::created_at))
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Add a tag to a card
    pub fn add_tag(&mut self, card_id: &str, tag: &str) -> Result<()> {
        let new_tag = NewWorkflowCardTag {
            card_id: card_id.to_string(),
            tag: tag.to_string(),
        };

        diesel::insert_into(workflow_card_tags::table)
            .values(&new_tag)
            .on_conflict_do_nothing()
            .execute(self.conn)?;

        Ok(())
    }

    /// Remove a tag from a card
    pub fn remove_tag(&mut self, card_id: &str, tag: &str) -> Result<()> {
        diesel::delete(
            workflow_card_tags::table
                .filter(workflow_card_tags::card_id.eq(card_id))
                .filter(workflow_card_tags::tag.eq(tag)),
        )
        .execute(self.conn)?;

        Ok(())
    }

    /// Get all tags for a card
    pub fn get_tags(&mut self, card_id: &str) -> Result<Vec<String>> {
        workflow_card_tags::table
            .filter(workflow_card_tags::card_id.eq(card_id))
            .select(workflow_card_tags::tag)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Find cards by tag
    pub fn find_by_tag(&mut self, tag: &str) -> Result<Vec<WorkflowCard>> {
        let card_ids: Vec<String> = workflow_card_tags::table
            .filter(workflow_card_tags::tag.eq(tag))
            .select(workflow_card_tags::card_id)
            .load(self.conn)?;

        workflow_cards::table
            .filter(workflow_cards::id.eq_any(card_ids))
            .order_by((workflow_cards::priority, workflow_cards::created_at))
            .load(self.conn)
            .map_err(Into::into)
    }
}
