//! Module data access layer

use crate::connection::DbConnection;
use crate::dal::traits::ModuleRepositoryTrait;
use crate::error::Result;
use crate::models::campaign::modules::{Module, NewModule, UpdateModule};
use crate::schema::modules;
use chrono::Utc;
use diesel::prelude::*;

/// Repository for module operations
pub struct ModuleRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleRepository<'a> {
    /// Create a new module repository
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new module
    pub fn create(&mut self, new_module: NewModule) -> Result<Module> {
        diesel::insert_into(modules::table)
            .values(&new_module)
            .returning(Module::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a module by ID
    pub fn find_by_id(&mut self, id: i32) -> Result<Option<Module>> {
        modules::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Update a module
    pub fn update(&mut self, id: i32, update: UpdateModule) -> Result<Module> {
        diesel::update(modules::table.find(id))
            .set(&update)
            .returning(Module::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Transition a module to a new status
    pub fn transition_status(&mut self, id: i32, new_status: &str) -> Result<Module> {
        // Get the module to check current state
        let module = self
            .find_by_id(id)?
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        // Transition validation is handled by BoardDefinition in the service layer

        let mut update = UpdateModule {
            status: Some(new_status.to_string()),
            ..Default::default()
        };

        // Set timestamps based on status transitions
        match new_status {
            "active" => {
                if module.started_at.is_none() {
                    update.started_at = Some(Some(Utc::now().to_rfc3339()));
                }
            }
            "completed" => {
                update.completed_at = Some(Some(Utc::now().to_rfc3339()));
            }
            _ => {}
        }

        self.update(id, update)
    }

    /// Increment session count for a module
    pub fn increment_sessions(&mut self, id: i32) -> Result<Module> {
        let module = self
            .find_by_id(id)?
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        let mut update = UpdateModule {
            actual_sessions: Some(module.actual_sessions + 1),
            ..Default::default()
        };

        // Auto-start if this is the first session
        if module.actual_sessions == 0 && module.started_at.is_none() {
            update.started_at = Some(Some(Utc::now().to_rfc3339()));
        }

        self.update(id, update)
    }

    /// Delete a module
    pub fn delete(&mut self, id: i32) -> Result<()> {
        diesel::delete(modules::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// List all modules for a campaign
    pub fn list_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Module>> {
        modules::table
            .filter(modules::campaign_id.eq(campaign_id))
            .order_by(modules::module_number)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List modules by status for a campaign
    pub fn list_by_campaign_and_status(
        &mut self,
        campaign_id: i32,
        status: &str,
    ) -> Result<Vec<Module>> {
        modules::table
            .filter(modules::campaign_id.eq(campaign_id))
            .filter(modules::status.eq(status))
            .order_by(modules::module_number)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Find modules that should trigger next module planning (60% complete)
    pub fn find_modules_needing_next(&mut self, campaign_id: i32) -> Result<Vec<Module>> {
        let modules = modules::table
            .filter(modules::campaign_id.eq(campaign_id))
            .filter(modules::status.eq("active"))
            .load::<Module>(self.conn)?;

        Ok(modules
            .into_iter()
            .filter(|m| m.should_trigger_next_module())
            .collect())
    }

    /// Get the next module number for a campaign
    pub fn get_next_module_number(&mut self, campaign_id: i32) -> Result<i32> {
        let max_number = modules::table
            .filter(modules::campaign_id.eq(campaign_id))
            .select(diesel::dsl::max(modules::module_number))
            .first::<Option<i32>>(self.conn)?
            .unwrap_or(0);

        Ok(max_number + 1)
    }
}

// =============================================================================
// Trait Implementation
// =============================================================================

impl<'a> ModuleRepositoryTrait for ModuleRepository<'a> {
    fn create(&mut self, new_module: NewModule) -> Result<Module> {
        ModuleRepository::create(self, new_module)
    }

    fn find_by_id(&mut self, id: i32) -> Result<Option<Module>> {
        ModuleRepository::find_by_id(self, id)
    }

    fn update(&mut self, id: i32, update: UpdateModule) -> Result<Module> {
        ModuleRepository::update(self, id, update)
    }

    fn transition_status(&mut self, id: i32, new_status: &str) -> Result<Module> {
        ModuleRepository::transition_status(self, id, new_status)
    }

    fn increment_sessions(&mut self, id: i32) -> Result<Module> {
        ModuleRepository::increment_sessions(self, id)
    }

    fn delete(&mut self, id: i32) -> Result<()> {
        ModuleRepository::delete(self, id)
    }

    fn list_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Module>> {
        ModuleRepository::list_by_campaign(self, campaign_id)
    }

    fn list_by_campaign_and_status(
        &mut self,
        campaign_id: i32,
        status: &str,
    ) -> Result<Vec<Module>> {
        ModuleRepository::list_by_campaign_and_status(self, campaign_id, status)
    }

    fn find_modules_needing_next(&mut self, campaign_id: i32) -> Result<Vec<Module>> {
        ModuleRepository::find_modules_needing_next(self, campaign_id)
    }

    fn get_next_module_number(&mut self, campaign_id: i32) -> Result<i32> {
        ModuleRepository::get_next_module_number(self, campaign_id)
    }
}
