//! Module monster service.
//!
//! Manages monster associations with modules, including encounter grouping.
//! Provides methods to add, remove, and query monsters for a module,
//! with optional lookup of full monster data from the catalog.

use crate::connection::DbConnection;
use crate::dal::campaign::module_monsters::ModuleMonsterRepository;
use crate::error::Result;
use crate::models::campaign::module_monsters::{
    EncounterGroup, ModuleMonster, ModuleMonsterWithData, NewModuleMonster, UpdateModuleMonster,
};
use crate::services::monster_renderer::{render_monsters_file, MonsterData};
use crate::services::MonsterService;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Service for managing monster associations with modules.
pub struct ModuleMonsterService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleMonsterService<'a> {
    /// Create a new module monster service.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Add a monster to a module.
    ///
    /// # Arguments
    /// * `module_id` - The module to add the monster to
    /// * `monster_name` - Name of the monster (from catalog)
    /// * `monster_source` - Source book abbreviation (e.g., "MM")
    /// * `quantity` - Number of this monster type
    /// * `encounter_tag` - Optional encounter grouping tag
    pub fn add_monster(
        &mut self,
        module_id: i32,
        monster_name: String,
        monster_source: String,
        quantity: i32,
        encounter_tag: Option<String>,
    ) -> Result<ModuleMonster> {
        let mut repo = ModuleMonsterRepository::new(self.conn);

        // Check if this monster already exists in the module with the same encounter tag
        if let Some(existing) =
            repo.find_existing(module_id, &monster_name, &monster_source, encounter_tag.as_deref())?
        {
            // Update quantity instead of creating duplicate
            let update = UpdateModuleMonster {
                quantity: Some(existing.quantity + quantity),
                encounter_tag: None,
            };
            return repo.update(existing.id, update);
        }

        let new_monster = NewModuleMonster {
            module_id,
            monster_name,
            monster_source,
            quantity,
            encounter_tag,
        };

        repo.create(new_monster)
    }

    /// Remove a monster entry from a module.
    pub fn remove_monster(&mut self, monster_id: i32) -> Result<()> {
        let mut repo = ModuleMonsterRepository::new(self.conn);
        repo.delete(monster_id)
    }

    /// Update a monster entry (quantity or encounter tag).
    pub fn update_monster(
        &mut self,
        monster_id: i32,
        quantity: Option<i32>,
        encounter_tag: Option<Option<String>>,
    ) -> Result<ModuleMonster> {
        let mut repo = ModuleMonsterRepository::new(self.conn);
        let update = UpdateModuleMonster {
            quantity,
            encounter_tag,
        };
        repo.update(monster_id, update)
    }

    /// Get all monsters for a module.
    pub fn get_monsters_for_module(&mut self, module_id: i32) -> Result<Vec<ModuleMonster>> {
        let mut repo = ModuleMonsterRepository::new(self.conn);
        repo.list_by_module(module_id)
    }

    /// Get all monsters for a module with full monster data from catalog.
    pub fn get_monsters_with_data(
        &mut self,
        module_id: i32,
    ) -> Result<Vec<ModuleMonsterWithData>> {
        let mut repo = ModuleMonsterRepository::new(self.conn);
        let monsters = repo.list_by_module(module_id)?;

        debug!(
            "Loading monster data for {} monsters in module {}",
            monsters.len(),
            module_id
        );

        let mut result = Vec::with_capacity(monsters.len());

        for monster in monsters {
            let mut with_data: ModuleMonsterWithData = monster.into();

            // Look up full monster data from catalog
            let mut monster_svc = MonsterService::new(self.conn);
            match monster_svc
                .get_monster_by_name_and_source(&with_data.monster_name, &with_data.monster_source)
            {
                Ok(Some(full_monster)) => {
                    // Convert the monster to a JSON Value
                    match serde_json::to_value(&full_monster) {
                        Ok(json_value) => {
                            debug!(
                                "Found catalog data for {} ({})",
                                with_data.monster_name, with_data.monster_source
                            );
                            with_data.monster_data = Some(json_value);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to serialize monster {} ({}): {}",
                                with_data.monster_name, with_data.monster_source, e
                            );
                        }
                    }
                }
                Ok(None) => {
                    warn!(
                        "Monster not found in catalog: {} (source: {})",
                        with_data.monster_name, with_data.monster_source
                    );
                }
                Err(e) => {
                    warn!(
                        "Error looking up monster {} ({}): {}",
                        with_data.monster_name, with_data.monster_source, e
                    );
                }
            }

            result.push(with_data);
        }

        Ok(result)
    }

    /// Get monsters grouped by encounter tag.
    pub fn get_monsters_grouped_by_encounter(
        &mut self,
        module_id: i32,
    ) -> Result<Vec<EncounterGroup>> {
        let mut repo = ModuleMonsterRepository::new(self.conn);
        let grouped = repo.list_by_module_grouped(module_id)?;

        let mut result = Vec::with_capacity(grouped.len());

        for (tag, monsters) in grouped {
            let mut monsters_with_data = Vec::with_capacity(monsters.len());

            for m in monsters {
                let mut with_data: ModuleMonsterWithData = m.into();

                // Look up full monster data
                let mut monster_svc = MonsterService::new(self.conn);
                match monster_svc.get_monster_by_name_and_source(
                    &with_data.monster_name,
                    &with_data.monster_source,
                ) {
                    Ok(Some(full_monster)) => {
                        if let Ok(json_value) = serde_json::to_value(&full_monster) {
                            with_data.monster_data = Some(json_value);
                        }
                    }
                    Ok(None) => {
                        warn!(
                            "Monster not found in catalog: {} (source: {})",
                            with_data.monster_name, with_data.monster_source
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Error looking up monster {} ({}): {}",
                            with_data.monster_name, with_data.monster_source, e
                        );
                    }
                }

                monsters_with_data.push(with_data);
            }

            result.push(EncounterGroup {
                encounter_tag: tag,
                monsters: monsters_with_data,
            });
        }

        Ok(result)
    }

    /// Get distinct encounter tags for a module.
    pub fn get_encounter_tags(&mut self, module_id: i32) -> Result<Vec<Option<String>>> {
        let mut repo = ModuleMonsterRepository::new(self.conn);
        repo.get_encounter_tags(module_id)
    }

    /// Remove all monsters from a module.
    pub fn clear_module_monsters(&mut self, module_id: i32) -> Result<usize> {
        let mut repo = ModuleMonsterRepository::new(self.conn);
        repo.delete_by_module(module_id)
    }

    /// Sync module monsters to a markdown file on disk.
    ///
    /// Creates or updates a `monsters.md` file in the module directory containing
    /// full stat blocks for all monsters, grouped by encounter tag.
    ///
    /// # Arguments
    /// * `module_id` - The module to sync
    /// * `campaign_directory` - The campaign's root directory path
    /// * `module_number` - The module number (for directory naming)
    /// * `module_name` - The module name (for the file header)
    pub fn sync_monsters_to_file(
        &mut self,
        module_id: i32,
        campaign_directory: &str,
        module_number: i32,
        module_name: &str,
    ) -> Result<()> {
        // Get all monsters grouped by encounter
        let encounter_groups = self.get_monsters_grouped_by_encounter(module_id)?;

        // Convert to the format needed by the renderer
        let encounters: Vec<(Option<String>, Vec<MonsterData>)> = encounter_groups
            .into_iter()
            .map(|group| {
                let monsters: Vec<MonsterData> = group
                    .monsters
                    .into_iter()
                    .map(|m| MonsterData {
                        name: m.monster_name,
                        source: m.monster_source,
                        quantity: m.quantity,
                        full_data: m.monster_data,
                    })
                    .collect();
                (group.encounter_tag, monsters)
            })
            .collect();

        // Build the module directory path
        let module_dir = PathBuf::from(campaign_directory)
            .join("modules")
            .join(format!("module_{:02}", module_number));

        // Ensure directory exists
        fs::create_dir_all(&module_dir)?;

        // Render the markdown
        let markdown = render_monsters_file(&encounters, module_name);

        // Write to file
        let file_path = module_dir.join("monsters.md");
        fs::write(&file_path, markdown)?;

        Ok(())
    }
}
