//! Module item service.
//!
//! Manages item/treasure associations with modules, including location grouping.
//! Items are catalog references looked up by name and source.

use crate::connection::DbConnection;
use crate::dal::campaign::module_items::ModuleItemRepository;
use crate::error::Result;
use crate::models::campaign::module_items::{
    LocationGroup, ModuleItem, ModuleItemWithData, NewModuleItem, UpdateModuleItem,
};
use crate::services::ItemService;

/// Service for managing item associations with modules.
pub struct ModuleItemService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleItemService<'a> {
    /// Create a new module item service.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Add an item to a module.
    ///
    /// # Arguments
    /// * `module_id` - The module to add the item to
    /// * `name` - Item name (from catalog)
    /// * `source` - Source book abbreviation (e.g., "DMG") or "campaign" for custom
    /// * `quantity` - Number of this item
    /// * `location` - Optional location tag (boss_chamber, hidden_cache, etc.)
    /// * `notes` - Optional notes about this item's role
    pub fn add_item(
        &mut self,
        module_id: i32,
        name: String,
        source: String,
        quantity: i32,
        location: Option<String>,
        notes: Option<String>,
    ) -> Result<ModuleItem> {
        let mut repo = ModuleItemRepository::new(self.conn);

        // Check if this item already exists in the module at the same location
        if let Some(existing) =
            repo.find_existing(module_id, &name, &source, location.as_deref())?
        {
            // Update quantity instead of creating duplicate
            let update = UpdateModuleItem {
                quantity: Some(existing.quantity + quantity),
                location: None,
                notes: None,
            };
            return repo.update(existing.id, update);
        }

        let new_item = NewModuleItem {
            module_id,
            name,
            source,
            quantity,
            location,
            notes,
        };

        repo.create(new_item)
    }

    /// Remove an item entry from a module.
    pub fn remove_item(&mut self, item_id: i32) -> Result<()> {
        let mut repo = ModuleItemRepository::new(self.conn);
        repo.delete(item_id)
    }

    /// Update an item entry (quantity, location, or notes).
    pub fn update_item(
        &mut self,
        item_id: i32,
        quantity: Option<i32>,
        location: Option<Option<String>>,
        notes: Option<Option<String>>,
    ) -> Result<ModuleItem> {
        let mut repo = ModuleItemRepository::new(self.conn);
        let update = UpdateModuleItem {
            quantity,
            location,
            notes,
        };
        repo.update(item_id, update)
    }

    /// Get all items for a module.
    pub fn get_items_for_module(&mut self, module_id: i32) -> Result<Vec<ModuleItem>> {
        let mut repo = ModuleItemRepository::new(self.conn);
        repo.list_by_module(module_id)
    }

    /// Get all items for a module with full item data from catalog.
    pub fn get_items_with_data(&mut self, module_id: i32) -> Result<Vec<ModuleItemWithData>> {
        let mut repo = ModuleItemRepository::new(self.conn);
        let items = repo.list_by_module(module_id)?;

        let mut result = Vec::with_capacity(items.len());

        for item in items {
            let mut with_data: ModuleItemWithData = item.into();

            // Look up full item data from catalog
            let mut item_svc = ItemService::new(self.conn);
            if let Ok(Some(full_item)) =
                item_svc.get_item_by_name_and_source(&with_data.name, &with_data.source)
            {
                if let Ok(json_value) = serde_json::to_value(&full_item) {
                    with_data.item_data = Some(json_value);
                }
            }

            result.push(with_data);
        }

        Ok(result)
    }

    /// Get items grouped by location.
    pub fn get_items_grouped_by_location(&mut self, module_id: i32) -> Result<Vec<LocationGroup>> {
        let mut repo = ModuleItemRepository::new(self.conn);
        let grouped = repo.list_by_module_grouped(module_id)?;

        let mut result = Vec::with_capacity(grouped.len());

        for (location, items) in grouped {
            let mut items_with_data = Vec::with_capacity(items.len());

            for item in items {
                let mut with_data: ModuleItemWithData = item.into();

                // Look up full item data
                let mut item_svc = ItemService::new(self.conn);
                if let Ok(Some(full_item)) =
                    item_svc.get_item_by_name_and_source(&with_data.name, &with_data.source)
                {
                    if let Ok(json_value) = serde_json::to_value(&full_item) {
                        with_data.item_data = Some(json_value);
                    }
                }

                items_with_data.push(with_data);
            }

            result.push(LocationGroup {
                location,
                items: items_with_data,
            });
        }

        Ok(result)
    }

    /// Get distinct locations for a module.
    pub fn get_locations(&mut self, module_id: i32) -> Result<Vec<Option<String>>> {
        let mut repo = ModuleItemRepository::new(self.conn);
        repo.get_locations(module_id)
    }

    /// Remove all items from a module.
    pub fn clear_module_items(&mut self, module_id: i32) -> Result<usize> {
        let mut repo = ModuleItemRepository::new(self.conn);
        repo.delete_by_module(module_id)
    }
}
