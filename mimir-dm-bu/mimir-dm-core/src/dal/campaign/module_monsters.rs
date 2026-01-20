//! Module monster data access layer

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::module_monsters::{ModuleMonster, NewModuleMonster, UpdateModuleMonster};
use crate::schema::module_monsters;
use chrono::Utc;
use diesel::prelude::*;

/// Repository for module monster operations
pub struct ModuleMonsterRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleMonsterRepository<'a> {
    /// Create a new module monster repository
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Add a monster to a module
    pub fn create(&mut self, new_monster: NewModuleMonster) -> Result<ModuleMonster> {
        diesel::insert_into(module_monsters::table)
            .values(&new_monster)
            .returning(ModuleMonster::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a module monster by ID
    pub fn find_by_id(&mut self, id: i32) -> Result<Option<ModuleMonster>> {
        module_monsters::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Update a module monster entry
    pub fn update(&mut self, id: i32, update: UpdateModuleMonster) -> Result<ModuleMonster> {
        diesel::update(module_monsters::table.find(id))
            .set((
                &update,
                module_monsters::updated_at.eq(Utc::now().to_rfc3339()),
            ))
            .returning(ModuleMonster::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Delete a module monster entry
    pub fn delete(&mut self, id: i32) -> Result<()> {
        diesel::delete(module_monsters::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// List all monsters for a module
    pub fn list_by_module(&mut self, module_id: i32) -> Result<Vec<ModuleMonster>> {
        module_monsters::table
            .filter(module_monsters::module_id.eq(module_id))
            .order_by((module_monsters::encounter_tag, module_monsters::monster_name))
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List monsters for a module grouped by encounter tag
    pub fn list_by_module_grouped(
        &mut self,
        module_id: i32,
    ) -> Result<Vec<(Option<String>, Vec<ModuleMonster>)>> {
        let monsters = self.list_by_module(module_id)?;

        // Group by encounter_tag
        let mut groups: Vec<(Option<String>, Vec<ModuleMonster>)> = Vec::new();

        for monster in monsters {
            let tag = monster.encounter_tag.clone();
            if let Some(group) = groups.iter_mut().find(|(t, _)| *t == tag) {
                group.1.push(monster);
            } else {
                groups.push((tag, vec![monster]));
            }
        }

        Ok(groups)
    }

    /// Find monsters by encounter tag within a module
    pub fn find_by_encounter(
        &mut self,
        module_id: i32,
        encounter_tag: Option<&str>,
    ) -> Result<Vec<ModuleMonster>> {
        let mut query = module_monsters::table
            .filter(module_monsters::module_id.eq(module_id))
            .into_boxed();

        match encounter_tag {
            Some(tag) => {
                query = query.filter(module_monsters::encounter_tag.eq(tag));
            }
            None => {
                query = query.filter(module_monsters::encounter_tag.is_null());
            }
        }

        query
            .order_by(module_monsters::monster_name)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Check if a monster already exists in a module (by name and source)
    pub fn find_existing(
        &mut self,
        module_id: i32,
        monster_name: &str,
        monster_source: &str,
        encounter_tag: Option<&str>,
    ) -> Result<Option<ModuleMonster>> {
        let mut query = module_monsters::table
            .filter(module_monsters::module_id.eq(module_id))
            .filter(module_monsters::monster_name.eq(monster_name))
            .filter(module_monsters::monster_source.eq(monster_source))
            .into_boxed();

        match encounter_tag {
            Some(tag) => {
                query = query.filter(module_monsters::encounter_tag.eq(tag));
            }
            None => {
                query = query.filter(module_monsters::encounter_tag.is_null());
            }
        }

        query.first(self.conn).optional().map_err(Into::into)
    }

    /// Delete all monsters for a module
    pub fn delete_by_module(&mut self, module_id: i32) -> Result<usize> {
        diesel::delete(module_monsters::table.filter(module_monsters::module_id.eq(module_id)))
            .execute(self.conn)
            .map_err(Into::into)
    }

    /// Get distinct encounter tags for a module
    pub fn get_encounter_tags(&mut self, module_id: i32) -> Result<Vec<Option<String>>> {
        module_monsters::table
            .filter(module_monsters::module_id.eq(module_id))
            .select(module_monsters::encounter_tag)
            .distinct()
            .load(self.conn)
            .map_err(Into::into)
    }
}
