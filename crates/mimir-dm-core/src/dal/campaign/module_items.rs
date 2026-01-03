//! Module item data access layer

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::campaign::module_items::{ModuleItem, NewModuleItem, UpdateModuleItem};
use crate::schema::module_items;
use chrono::Utc;
use diesel::prelude::*;

/// Repository for module item operations
pub struct ModuleItemRepository<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleItemRepository<'a> {
    /// Create a new module item repository
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Add an item to a module
    pub fn create(&mut self, new_item: NewModuleItem) -> Result<ModuleItem> {
        diesel::insert_into(module_items::table)
            .values(&new_item)
            .returning(ModuleItem::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Find a module item by ID
    pub fn find_by_id(&mut self, id: i32) -> Result<Option<ModuleItem>> {
        module_items::table
            .find(id)
            .first(self.conn)
            .optional()
            .map_err(Into::into)
    }

    /// Update a module item entry
    pub fn update(&mut self, id: i32, update: UpdateModuleItem) -> Result<ModuleItem> {
        diesel::update(module_items::table.find(id))
            .set((
                &update,
                module_items::updated_at.eq(Utc::now().to_rfc3339()),
            ))
            .returning(ModuleItem::as_returning())
            .get_result(self.conn)
            .map_err(Into::into)
    }

    /// Delete a module item entry
    pub fn delete(&mut self, id: i32) -> Result<()> {
        diesel::delete(module_items::table.find(id)).execute(self.conn)?;
        Ok(())
    }

    /// List all items for a module
    pub fn list_by_module(&mut self, module_id: i32) -> Result<Vec<ModuleItem>> {
        module_items::table
            .filter(module_items::module_id.eq(module_id))
            .order_by((module_items::location, module_items::name))
            .load(self.conn)
            .map_err(Into::into)
    }

    /// List items for a module grouped by location
    pub fn list_by_module_grouped(
        &mut self,
        module_id: i32,
    ) -> Result<Vec<(Option<String>, Vec<ModuleItem>)>> {
        let items = self.list_by_module(module_id)?;

        // Group by location
        let mut groups: Vec<(Option<String>, Vec<ModuleItem>)> = Vec::new();

        for item in items {
            let location = item.location.clone();
            if let Some(group) = groups.iter_mut().find(|(l, _)| *l == location) {
                group.1.push(item);
            } else {
                groups.push((location, vec![item]));
            }
        }

        Ok(groups)
    }

    /// Find items by location within a module
    pub fn find_by_location(
        &mut self,
        module_id: i32,
        location: Option<&str>,
    ) -> Result<Vec<ModuleItem>> {
        let mut query = module_items::table
            .filter(module_items::module_id.eq(module_id))
            .into_boxed();

        match location {
            Some(loc) => {
                query = query.filter(module_items::location.eq(loc));
            }
            None => {
                query = query.filter(module_items::location.is_null());
            }
        }

        query
            .order_by(module_items::name)
            .load(self.conn)
            .map_err(Into::into)
    }

    /// Check if an item already exists in a module (by name and source)
    pub fn find_existing(
        &mut self,
        module_id: i32,
        name: &str,
        source: &str,
        location: Option<&str>,
    ) -> Result<Option<ModuleItem>> {
        let mut query = module_items::table
            .filter(module_items::module_id.eq(module_id))
            .filter(module_items::name.eq(name))
            .filter(module_items::source.eq(source))
            .into_boxed();

        match location {
            Some(loc) => {
                query = query.filter(module_items::location.eq(loc));
            }
            None => {
                query = query.filter(module_items::location.is_null());
            }
        }

        query.first(self.conn).optional().map_err(Into::into)
    }

    /// Delete all items for a module
    pub fn delete_by_module(&mut self, module_id: i32) -> Result<usize> {
        diesel::delete(module_items::table.filter(module_items::module_id.eq(module_id)))
            .execute(self.conn)
            .map_err(Into::into)
    }

    /// Get distinct locations for a module
    pub fn get_locations(&mut self, module_id: i32) -> Result<Vec<Option<String>>> {
        module_items::table
            .filter(module_items::module_id.eq(module_id))
            .select(module_items::location)
            .distinct()
            .load(self.conn)
            .map_err(Into::into)
    }
}
