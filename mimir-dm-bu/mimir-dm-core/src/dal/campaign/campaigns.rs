//! Campaign data access layer

use crate::connection::DbConnection;
use crate::dal::traits::CampaignRepositoryTrait;
use crate::error::Result;
use crate::models::campaign::campaigns::{Campaign, NewCampaign, UpdateCampaign};
use crate::schema::campaigns;
use chrono::Utc;
use diesel::prelude::*;

/// Repository for campaign operations
pub struct CampaignRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CampaignRepository<'a> {
    /// Create a new campaign repository
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new campaign
    pub fn create(&mut self, new_campaign: NewCampaign) -> Result<Campaign> {
        diesel::insert_into(campaigns::table)
            .values(&new_campaign)
            .returning(Campaign::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a campaign by ID
    pub fn find_by_id(&mut self, id: i32) -> Result<Option<Campaign>> {
        campaigns::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Update a campaign
    pub fn update(&mut self, id: i32, update: UpdateCampaign) -> Result<Campaign> {
        // Update last_activity_at
        let mut update = update;
        update.last_activity_at = Some(Utc::now().to_rfc3339());

        diesel::update(campaigns::table.find(id))
            .set(&update)
            .returning(Campaign::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Transition a campaign to a new status
    pub fn transition_status(&mut self, id: i32, new_status: &str) -> Result<Campaign> {
        let update = UpdateCampaign {
            status: Some(new_status.to_string()),
            last_activity_at: Some(Utc::now().to_rfc3339()),
            ..Default::default()
        };

        self.update(id, update)
    }

    /// Delete a campaign
    pub fn delete(&mut self, id: i32) -> Result<()> {
        diesel::delete(campaigns::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// List all campaigns
    pub fn list(&mut self) -> Result<Vec<Campaign>> {
        campaigns::table
            .order_by(campaigns::last_activity_at.desc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List campaigns by status
    pub fn list_by_status(&mut self, status: &str) -> Result<Vec<Campaign>> {
        campaigns::table
            .filter(campaigns::status.eq(status))
            .order_by(campaigns::last_activity_at.desc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List active campaigns (not archived)
    pub fn list_active(&mut self) -> Result<Vec<Campaign>> {
        campaigns::table
            .filter(campaigns::archived_at.is_null())
            .order_by(campaigns::last_activity_at.desc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List archived campaigns
    pub fn list_archived(&mut self) -> Result<Vec<Campaign>> {
        campaigns::table
            .filter(campaigns::archived_at.is_not_null())
            .order_by(campaigns::archived_at.desc())
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Archive a campaign
    pub fn archive(&mut self, id: i32) -> Result<Campaign> {
        let update = UpdateCampaign {
            archived_at: Some(Some(Utc::now().to_rfc3339())),
            last_activity_at: Some(Utc::now().to_rfc3339()),
            ..Default::default()
        };

        self.update(id, update)
    }

    /// Unarchive a campaign
    pub fn unarchive(&mut self, id: i32) -> Result<Campaign> {
        let update = UpdateCampaign {
            archived_at: Some(None),
            last_activity_at: Some(Utc::now().to_rfc3339()),
            ..Default::default()
        };

        self.update(id, update)
    }
}

// =============================================================================
// Trait Implementation
// =============================================================================

impl<'a> CampaignRepositoryTrait for CampaignRepository<'a> {
    fn create(&mut self, new_campaign: NewCampaign) -> Result<Campaign> {
        CampaignRepository::create(self, new_campaign)
    }

    fn find_by_id(&mut self, id: i32) -> Result<Option<Campaign>> {
        CampaignRepository::find_by_id(self, id)
    }

    fn update(&mut self, id: i32, update: UpdateCampaign) -> Result<Campaign> {
        CampaignRepository::update(self, id, update)
    }

    fn transition_status(&mut self, id: i32, new_status: &str) -> Result<Campaign> {
        CampaignRepository::transition_status(self, id, new_status)
    }

    fn delete(&mut self, id: i32) -> Result<()> {
        CampaignRepository::delete(self, id)
    }

    fn list(&mut self) -> Result<Vec<Campaign>> {
        CampaignRepository::list(self)
    }

    fn list_by_status(&mut self, status: &str) -> Result<Vec<Campaign>> {
        CampaignRepository::list_by_status(self, status)
    }

    fn list_active(&mut self) -> Result<Vec<Campaign>> {
        CampaignRepository::list_active(self)
    }

    fn list_archived(&mut self) -> Result<Vec<Campaign>> {
        CampaignRepository::list_archived(self)
    }

    fn archive(&mut self, id: i32) -> Result<Campaign> {
        CampaignRepository::archive(self, id)
    }

    fn unarchive(&mut self, id: i32) -> Result<Campaign> {
        CampaignRepository::unarchive(self, id)
    }
}
