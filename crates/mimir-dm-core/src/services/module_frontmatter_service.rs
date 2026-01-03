//! Module frontmatter sync service.
//!
//! Handles bidirectional synchronization between module document frontmatter
//! and the database tables (module_monsters, module_npcs, module_items).
//!
//! Document is the source of truth - changes flow:
//! - Document → DB on load (parse frontmatter, sync to tables)
//! - DB → Document on UI change (update frontmatter section)

use crate::connection::DbConnection;
use crate::dal::campaign::module_items::ModuleItemRepository;
use crate::dal::campaign::module_monsters::ModuleMonsterRepository;
use crate::dal::campaign::module_npcs::ModuleNpcRepository;
use crate::dal::campaign::modules::ModuleRepository;
use crate::dal::character::CharacterRepository;
use crate::error::Result;
use crate::models::campaign::module_frontmatter::{
    ItemReference, ModuleFrontmatter, MonsterReference, NpcReference,
};
use crate::models::campaign::module_items::NewModuleItem;
use crate::models::campaign::module_monsters::NewModuleMonster;
use crate::models::campaign::module_npcs::NewModuleNpc;
use std::fs;
use std::path::Path;

/// Service for syncing module frontmatter with database tables.
pub struct ModuleFrontmatterService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleFrontmatterService<'a> {
    /// Create a new module frontmatter service.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Sync module data from document frontmatter to database.
    ///
    /// This parses the module's overview document, extracts frontmatter,
    /// and updates the database tables to match.
    ///
    /// Uses REPLACE strategy: clears existing data and inserts from frontmatter.
    pub fn sync_from_document(&mut self, module_id: i32, document_path: &Path) -> Result<SyncResult> {
        // Read the document
        let content = fs::read_to_string(document_path)?;

        // Parse frontmatter
        let frontmatter = match ModuleFrontmatter::parse_from_markdown(&content) {
            Some(fm) => fm,
            None => {
                return Ok(SyncResult {
                    monsters_synced: 0,
                    npcs_synced: 0,
                    items_synced: 0,
                    warnings: vec!["No frontmatter found in document".to_string()],
                });
            }
        };

        let mut warnings = Vec::new();

        // Get campaign_id for NPC lookups
        let module = ModuleRepository::new(self.conn).find_by_id(module_id)?;
        let campaign_id = module.map(|m| m.campaign_id);

        // Sync monsters (REPLACE strategy)
        let monsters_synced = self.sync_monsters(module_id, &frontmatter.monsters)?;

        // Sync NPCs (REPLACE strategy) - requires campaign_id for character lookup
        let npcs_synced = if let Some(cid) = campaign_id {
            self.sync_npcs(module_id, cid, &frontmatter.npcs, &mut warnings)?
        } else {
            warnings.push("Cannot sync NPCs: module has no campaign".to_string());
            0
        };

        // Sync items (REPLACE strategy)
        let items_synced = self.sync_items(module_id, &frontmatter.items)?;

        Ok(SyncResult {
            monsters_synced,
            npcs_synced,
            items_synced,
            warnings,
        })
    }

    /// Sync monsters from frontmatter to database.
    fn sync_monsters(&mut self, module_id: i32, monsters: &[MonsterReference]) -> Result<usize> {
        let mut repo = ModuleMonsterRepository::new(self.conn);

        // Clear existing monsters for this module
        repo.delete_by_module(module_id)?;

        // Insert monsters from frontmatter
        let mut count = 0;
        for monster in monsters {
            let new_monster = NewModuleMonster {
                module_id,
                monster_name: monster.name.clone(),
                monster_source: monster.source.clone(),
                quantity: monster.quantity,
                encounter_tag: monster.encounter.clone(),
            };
            repo.create(new_monster)?;
            count += 1;
        }

        Ok(count)
    }

    /// Sync NPCs from frontmatter to database.
    ///
    /// NPCs are looked up by name in the campaign's characters table.
    /// Characters that don't exist or aren't NPCs are skipped with warnings.
    fn sync_npcs(
        &mut self,
        module_id: i32,
        campaign_id: i32,
        npcs: &[NpcReference],
        warnings: &mut Vec<String>,
    ) -> Result<usize> {
        // Clear existing NPCs for this module
        {
            let mut npc_repo = ModuleNpcRepository::new(self.conn);
            npc_repo.delete_by_module(module_id)?;
        }

        // First, look up all character IDs
        let mut npc_data: Vec<(i32, Option<String>, Option<String>, Option<String>)> = Vec::new();
        for npc in npcs {
            let mut char_repo = CharacterRepository::new(self.conn);
            match char_repo.find_npc_by_name_in_campaign(campaign_id, &npc.name)? {
                Some(character) => {
                    npc_data.push((
                        character.id,
                        npc.role.clone(),
                        npc.encounter_tag.clone(),
                        npc.notes.clone(),
                    ));
                }
                None => {
                    warnings.push(format!(
                        "NPC '{}' not found in campaign (must be a character with is_npc=true)",
                        npc.name
                    ));
                }
            }
        }

        // Now insert the NPCs
        let mut count = 0;
        for (character_id, role, encounter_tag, notes) in npc_data {
            let mut npc_repo = ModuleNpcRepository::new(self.conn);
            let new_npc = NewModuleNpc {
                module_id,
                character_id,
                role,
                encounter_tag,
                notes,
            };
            npc_repo.create(new_npc)?;
            count += 1;
        }

        Ok(count)
    }

    /// Sync items from frontmatter to database.
    fn sync_items(&mut self, module_id: i32, items: &[ItemReference]) -> Result<usize> {
        let mut repo = ModuleItemRepository::new(self.conn);

        // Clear existing items for this module
        repo.delete_by_module(module_id)?;

        // Insert items from frontmatter
        let mut count = 0;
        for item in items {
            let new_item = NewModuleItem {
                module_id,
                location: item.location.clone(),
                name: item.name.clone(),
                source: item.source.clone(),
                quantity: item.quantity,
                notes: item.notes.clone(),
            };
            repo.create(new_item)?;
            count += 1;
        }

        Ok(count)
    }

    /// Update document frontmatter from database state.
    ///
    /// Reads current monsters/NPCs/items from database and updates
    /// the frontmatter section of the document.
    pub fn sync_to_document(&mut self, module_id: i32, document_path: &Path) -> Result<()> {
        // Read existing document
        let content = fs::read_to_string(document_path)?;

        // Get campaign_id for NPC character names
        let module = ModuleRepository::new(self.conn).find_by_id(module_id)?;
        let campaign_id = module.map(|m| m.campaign_id);

        // Build new frontmatter from database
        let monsters = self.build_monster_references(module_id)?;
        let npcs = if campaign_id.is_some() {
            self.build_npc_references(module_id)?
        } else {
            Vec::new()
        };
        let items = self.build_item_references(module_id)?;

        // Update the document
        let updated_content = self.update_frontmatter_in_document(
            &content,
            &monsters,
            &npcs,
            &items,
        )?;

        // Write back
        fs::write(document_path, updated_content)?;

        Ok(())
    }

    /// Build monster references from database.
    fn build_monster_references(&mut self, module_id: i32) -> Result<Vec<MonsterReference>> {
        let mut repo = ModuleMonsterRepository::new(self.conn);
        let monsters = repo.list_by_module(module_id)?;

        Ok(monsters
            .into_iter()
            .map(|m| MonsterReference {
                encounter: m.encounter_tag,
                name: m.monster_name,
                source: m.monster_source,
                quantity: m.quantity,
                notes: None, // Notes not stored in DB currently
            })
            .collect())
    }

    /// Build NPC references from database.
    fn build_npc_references(&mut self, module_id: i32) -> Result<Vec<NpcReference>> {
        let mut npc_repo = ModuleNpcRepository::new(self.conn);
        let npcs = npc_repo.list_by_module(module_id)?;

        let mut references = Vec::with_capacity(npcs.len());
        for npc in npcs {
            // Look up character name
            let mut char_repo = CharacterRepository::new(self.conn);
            if let Some(character) = char_repo.find_by_id(npc.character_id)? {
                references.push(NpcReference {
                    role: npc.role,
                    name: character.character_name,
                    encounter_tag: npc.encounter_tag,
                    notes: npc.notes,
                });
            }
        }

        Ok(references)
    }

    /// Build item references from database.
    fn build_item_references(&mut self, module_id: i32) -> Result<Vec<ItemReference>> {
        let mut repo = ModuleItemRepository::new(self.conn);
        let items = repo.list_by_module(module_id)?;

        Ok(items
            .into_iter()
            .map(|i| ItemReference {
                location: i.location,
                name: i.name,
                source: i.source,
                quantity: i.quantity,
                notes: i.notes,
            })
            .collect())
    }

    /// Update frontmatter section in document content.
    ///
    /// Parses existing frontmatter, updates the monsters/npcs/items arrays,
    /// and reconstructs the document.
    fn update_frontmatter_in_document(
        &self,
        content: &str,
        monsters: &[MonsterReference],
        npcs: &[NpcReference],
        items: &[ItemReference],
    ) -> Result<String> {
        use gray_matter::Matter;
        use gray_matter::engine::YAML;

        let matter = Matter::<YAML>::new();

        // Parse existing document
        let parsed = matter.parse::<serde_yaml::Value>(content)
            .map_err(|e| crate::error::DbError::InvalidData(format!("Failed to parse frontmatter: {}", e)))?;

        // Get existing frontmatter or create empty
        let mut frontmatter: serde_yaml::Value = parsed.data
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

        // Update the arrays in frontmatter
        if let serde_yaml::Value::Mapping(ref mut map) = frontmatter {
            // Update monsters
            if !monsters.is_empty() {
                let monsters_yaml = serde_yaml::to_value(monsters)
                    .unwrap_or(serde_yaml::Value::Sequence(vec![]));
                map.insert(
                    serde_yaml::Value::String("monsters".to_string()),
                    monsters_yaml,
                );
            } else {
                map.remove(&serde_yaml::Value::String("monsters".to_string()));
            }

            // Update npcs
            if !npcs.is_empty() {
                let npcs_yaml = serde_yaml::to_value(npcs)
                    .unwrap_or(serde_yaml::Value::Sequence(vec![]));
                map.insert(
                    serde_yaml::Value::String("npcs".to_string()),
                    npcs_yaml,
                );
            } else {
                map.remove(&serde_yaml::Value::String("npcs".to_string()));
            }

            // Update items
            if !items.is_empty() {
                let items_yaml = serde_yaml::to_value(items)
                    .unwrap_or(serde_yaml::Value::Sequence(vec![]));
                map.insert(
                    serde_yaml::Value::String("items".to_string()),
                    items_yaml,
                );
            } else {
                map.remove(&serde_yaml::Value::String("items".to_string()));
            }
        }

        // Serialize frontmatter back to YAML
        let yaml_str = serde_yaml::to_string(&frontmatter)
            .map_err(|e| crate::error::DbError::InvalidData(format!("Failed to serialize frontmatter: {}", e)))?;

        // Reconstruct document
        Ok(format!("---\n{}---\n{}", yaml_str, parsed.content))
    }
}

/// Result of syncing from document to database.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Number of monsters synced
    pub monsters_synced: usize,
    /// Number of NPCs synced
    pub npcs_synced: usize,
    /// Number of items synced
    pub items_synced: usize,
    /// Warnings (e.g., NPCs not found)
    pub warnings: Vec<String>,
}
