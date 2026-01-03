//! Module NPC data access layer

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::module_npcs::{ModuleNpc, NewModuleNpc, UpdateModuleNpc};
use crate::schema::module_npcs;
use chrono::Utc;
use diesel::prelude::*;

/// Repository for module NPC operations
pub struct ModuleNpcRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleNpcRepository<'a> {
    /// Create a new module NPC repository
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Add an NPC to a module
    pub fn create(&mut self, new_npc: NewModuleNpc) -> Result<ModuleNpc> {
        diesel::insert_into(module_npcs::table)
            .values(&new_npc)
            .returning(ModuleNpc::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a module NPC by ID
    pub fn find_by_id(&mut self, id: i32) -> Result<Option<ModuleNpc>> {
        module_npcs::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Update a module NPC entry
    pub fn update(&mut self, id: i32, update: UpdateModuleNpc) -> Result<ModuleNpc> {
        diesel::update(module_npcs::table.find(id))
            .set((
                &update,
                module_npcs::updated_at.eq(Utc::now().to_rfc3339()),
            ))
            .returning(ModuleNpc::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Delete a module NPC entry
    pub fn delete(&mut self, id: i32) -> Result<()> {
        diesel::delete(module_npcs::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// List all NPCs for a module
    pub fn list_by_module(&mut self, module_id: i32) -> Result<Vec<ModuleNpc>> {
        module_npcs::table
            .filter(module_npcs::module_id.eq(module_id))
            .order_by((module_npcs::role, module_npcs::id))
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List NPCs for a module grouped by role
    pub fn list_by_module_grouped(
        &mut self,
        module_id: i32,
    ) -> Result<Vec<(Option<String>, Vec<ModuleNpc>)>> {
        let npcs = self.list_by_module(module_id)?;

        // Group by role
        let mut groups: Vec<(Option<String>, Vec<ModuleNpc>)> = Vec::new();

        for npc in npcs {
            let role = npc.role.clone();
            if let Some(group) = groups.iter_mut().find(|(r, _)| *r == role) {
                group.1.push(npc);
            } else {
                groups.push((role, vec![npc]));
            }
        }

        Ok(groups)
    }

    /// Find NPCs by role within a module
    pub fn find_by_role(
        &mut self,
        module_id: i32,
        role: Option<&str>,
    ) -> Result<Vec<ModuleNpc>> {
        let mut query = module_npcs::table
            .filter(module_npcs::module_id.eq(module_id))
            .into_boxed();

        match role {
            Some(r) => {
                query = query.filter(module_npcs::role.eq(r));
            }
            None => {
                query = query.filter(module_npcs::role.is_null());
            }
        }

        query.load(self.conn).map_err(Into::into)
    }

    /// Find NPCs by encounter tag within a module
    pub fn find_by_encounter_tag(
        &mut self,
        module_id: i32,
        encounter_tag: Option<&str>,
    ) -> Result<Vec<ModuleNpc>> {
        let mut query = module_npcs::table
            .filter(module_npcs::module_id.eq(module_id))
            .into_boxed();

        match encounter_tag {
            Some(tag) => {
                query = query.filter(module_npcs::encounter_tag.eq(tag));
            }
            None => {
                query = query.filter(module_npcs::encounter_tag.is_null());
            }
        }

        query.load(self.conn).map_err(Into::into)
    }

    /// Check if a character is already linked to a module
    pub fn find_existing(
        &mut self,
        module_id: i32,
        character_id: i32,
    ) -> Result<Option<ModuleNpc>> {
        module_npcs::table
            .filter(module_npcs::module_id.eq(module_id))
            .filter(module_npcs::character_id.eq(character_id))
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Delete all NPCs for a module
    pub fn delete_by_module(&mut self, module_id: i32) -> Result<usize> {
        diesel::delete(module_npcs::table.filter(module_npcs::module_id.eq(module_id)))
            .execute(self.conn)
            .map_err(Into::into)
    }

    /// Get distinct roles for a module
    pub fn get_roles(&mut self, module_id: i32) -> Result<Vec<Option<String>>> {
        module_npcs::table
            .filter(module_npcs::module_id.eq(module_id))
            .select(module_npcs::role)
            .distinct()
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Get distinct encounter tags for a module
    pub fn get_encounter_tags(&mut self, module_id: i32) -> Result<Vec<Option<String>>> {
        module_npcs::table
            .filter(module_npcs::module_id.eq(module_id))
            .select(module_npcs::encounter_tag)
            .distinct()
            .load(self.conn)
            .map_err(Into::into)
    }
}
