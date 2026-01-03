//! Module NPC service.
//!
//! Manages NPC associations with modules. NPCs are characters with is_npc=true,
//! linked by character_id rather than catalog lookup.

use crate::connection::DbConnection;
use crate::dal::campaign::module_npcs::ModuleNpcRepository;
use crate::dal::character::CharacterRepository;
use crate::error::{DbError, Result};
use crate::models::campaign::module_npcs::{
    ModuleNpc, ModuleNpcWithCharacter, NewModuleNpc, RoleGroup, UpdateModuleNpc,
};

/// Service for managing NPC associations with modules.
pub struct ModuleNpcService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleNpcService<'a> {
    /// Create a new module NPC service.
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Add an NPC to a module by character ID.
    ///
    /// # Arguments
    /// * `module_id` - The module to add the NPC to
    /// * `character_id` - ID of the character (must have is_npc=true)
    /// * `role` - Optional role (quest_giver, antagonist, ally, etc.)
    /// * `encounter_tag` - Optional encounter/scene grouping tag
    /// * `notes` - Optional notes about this NPC's role
    pub fn add_npc(
        &mut self,
        module_id: i32,
        character_id: i32,
        role: Option<String>,
        encounter_tag: Option<String>,
        notes: Option<String>,
    ) -> Result<ModuleNpc> {
        let mut repo = ModuleNpcRepository::new(self.conn);

        // Check if this character is already linked to the module
        if let Some(existing) = repo.find_existing(module_id, character_id)? {
            // Update existing entry instead of creating duplicate
            let update = UpdateModuleNpc {
                role: role.map(Some),
                encounter_tag: encounter_tag.map(Some),
                notes: notes.map(Some),
            };
            return repo.update(existing.id, update);
        }

        let new_npc = NewModuleNpc {
            module_id,
            character_id,
            role,
            encounter_tag,
            notes,
        };

        repo.create(new_npc)
    }

    /// Add an NPC to a module by character name.
    ///
    /// Looks up the character by name in the campaign and links it.
    /// Returns an error if the character is not found or is not an NPC.
    pub fn add_npc_by_name(
        &mut self,
        module_id: i32,
        campaign_id: i32,
        character_name: &str,
        role: Option<String>,
        encounter_tag: Option<String>,
        notes: Option<String>,
    ) -> Result<ModuleNpc> {
        // Look up the NPC by name (already filters to is_npc=1)
        let mut char_repo = CharacterRepository::new(self.conn);
        let character = char_repo
            .find_npc_by_name_in_campaign(campaign_id, character_name)?
            .ok_or_else(|| {
                DbError::NotFound {
                    entity_type: "NPC".to_string(),
                    id: character_name.to_string(),
                }
            })?;

        self.add_npc(module_id, character.id, role, encounter_tag, notes)
    }

    /// Remove an NPC entry from a module.
    pub fn remove_npc(&mut self, npc_id: i32) -> Result<()> {
        let mut repo = ModuleNpcRepository::new(self.conn);
        repo.delete(npc_id)
    }

    /// Update an NPC entry (role, encounter_tag, or notes).
    pub fn update_npc(
        &mut self,
        npc_id: i32,
        role: Option<Option<String>>,
        encounter_tag: Option<Option<String>>,
        notes: Option<Option<String>>,
    ) -> Result<ModuleNpc> {
        let mut repo = ModuleNpcRepository::new(self.conn);
        let update = UpdateModuleNpc {
            role,
            encounter_tag,
            notes,
        };
        repo.update(npc_id, update)
    }

    /// Get all NPCs for a module.
    pub fn get_npcs_for_module(&mut self, module_id: i32) -> Result<Vec<ModuleNpc>> {
        let mut repo = ModuleNpcRepository::new(self.conn);
        repo.list_by_module(module_id)
    }

    /// Get all NPCs for a module with character data.
    pub fn get_npcs_with_character_data(
        &mut self,
        module_id: i32,
    ) -> Result<Vec<ModuleNpcWithCharacter>> {
        let mut repo = ModuleNpcRepository::new(self.conn);
        let npcs = repo.list_by_module(module_id)?;

        let mut result = Vec::with_capacity(npcs.len());

        for npc in npcs {
            let mut char_repo = CharacterRepository::new(self.conn);
            if let Some(character) = char_repo.find_by_id(npc.character_id)? {
                result.push(ModuleNpcWithCharacter::from_parts(npc, &character));
            }
        }

        Ok(result)
    }

    /// Get NPCs grouped by role.
    pub fn get_npcs_grouped_by_role(&mut self, module_id: i32) -> Result<Vec<RoleGroup>> {
        let mut repo = ModuleNpcRepository::new(self.conn);
        let grouped = repo.list_by_module_grouped(module_id)?;

        let mut result = Vec::with_capacity(grouped.len());

        for (role, npcs) in grouped {
            let mut npcs_with_character = Vec::with_capacity(npcs.len());

            for npc in npcs {
                let mut char_repo = CharacterRepository::new(self.conn);
                if let Some(character) = char_repo.find_by_id(npc.character_id)? {
                    npcs_with_character.push(ModuleNpcWithCharacter::from_parts(npc, &character));
                }
            }

            result.push(RoleGroup {
                role,
                npcs: npcs_with_character,
            });
        }

        Ok(result)
    }

    /// Get distinct roles for a module.
    pub fn get_roles(&mut self, module_id: i32) -> Result<Vec<Option<String>>> {
        let mut repo = ModuleNpcRepository::new(self.conn);
        repo.get_roles(module_id)
    }

    /// Get distinct encounter tags for a module.
    pub fn get_encounter_tags(&mut self, module_id: i32) -> Result<Vec<Option<String>>> {
        let mut repo = ModuleNpcRepository::new(self.conn);
        repo.get_encounter_tags(module_id)
    }

    /// Remove all NPCs from a module.
    pub fn clear_module_npcs(&mut self, module_id: i32) -> Result<usize> {
        let mut repo = ModuleNpcRepository::new(self.conn);
        repo.delete_by_module(module_id)
    }
}
