//! Character management services
//!
//! This module contains services for managing D&D characters, organized by responsibility:
//!
//! - [`CharacterService`] - Core CRUD, inventory, spells, and version management
//! - [`CharacterProgressionService`] - Level up, ASI, multiclassing
//! - [`CharacterSpellService`] - Spell learning and preparation
//!
//! Supporting modules:
//! - [`renderer`] - Markdown rendering for character sheets
//! - [`level_up`] - Level up types and utilities
//! - [`creation`] - Character builder
//! - [`spell_management`] - Spell slot calculations

pub mod creation;
pub mod level_up;
pub mod progression;
pub mod renderer;
pub mod spell_management;
pub mod spells;

pub use creation::{AbilityScoreMethod, CharacterBuilder};
pub use level_up::{AsiOrFeat, ClassInfo, HpGainMethod, LevelUpOptions, MulticlassPrerequisites};
pub use progression::CharacterProgressionService;
pub use renderer::{CharacterRenderer, MarkdownRenderer};
pub use spell_management::{
    calculate_spell_attack_bonus, calculate_spell_save_dc, calculate_spell_slots, RestType,
};
pub use spells::CharacterSpellService;

use crate::{
    connection::DbConnection,
    dal::character::{CharacterRepository, CharacterVersionRepository},
    error::{DbError, Result},
    models::character::{
        Character, CharacterData, CharacterVersion, NewCharacter, NewCharacterVersion,
        UpdateCharacter,
    },
};
use std::fs;
use std::path::{Path, PathBuf};

/// Service for character management operations
pub struct CharacterService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CharacterService<'a> {
    /// Create a new character service
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new character
    pub fn create_character(
        &mut self,
        campaign_id: Option<i32>,
        player_id: Option<i32>,
        is_npc: bool,
        base_directory: &str,
        character_data: CharacterData,
    ) -> Result<Character> {
        // Validate inputs
        if character_data.character_name.trim().is_empty() {
            return Err(DbError::InvalidData(
                "Character name cannot be empty".to_string(),
            ));
        }

        // Serialize character data to YAML (always needed for database)
        let yaml_data = serde_yaml::to_string(&character_data).map_err(|e| {
            DbError::InvalidData(format!("Failed to serialize character data: {}", e))
        })?;

        let version_number = 1;
        let (directory_path, file_path_str) = if campaign_id.is_some() && !base_directory.is_empty()
        {
            // Character is assigned to a campaign with valid directory - create files
            let char_dir =
                self.create_character_directory(base_directory, &character_data.character_name)?;
            let directory_path = char_dir.to_string_lossy().to_string();
            let file_path = self.get_version_file_path(
                &char_dir,
                &character_data.character_name,
                version_number,
            );

            // Generate markdown
            let renderer = MarkdownRenderer::new();
            let markdown = renderer.render(&character_data);

            // Write files to disk
            self.write_character_files(&file_path, &yaml_data, &markdown)?;

            (directory_path, file_path.to_string_lossy().to_string())
        } else {
            // Character is unassigned or no directory specified - skip file creation, store only in database
            ("".to_string(), "".to_string())
        };

        // Create database record for character
        let character_id = {
            let mut char_repo = CharacterRepository::new(self.conn);
            let new_character = NewCharacter {
                campaign_id,
                player_id,
                character_name: character_data.character_name.clone(),
                is_npc: Some(is_npc),
                directory_path,
                class: Some(character_data.primary_class_name().to_string()),
                race: Some(character_data.race.clone()),
            };
            char_repo.create(new_character)?.id
        };

        // Create version record
        {
            let mut ver_repo = CharacterVersionRepository::new(self.conn);
            let new_version = NewCharacterVersion {
                character_id,
                version_number,
                file_path: file_path_str,
                character_data: yaml_data,
                snapshot_reason: character_data.snapshot_reason.clone(),
                level: character_data.level,
            };
            ver_repo.create(new_version)?;
        }

        // Update character's current_level to match the created version
        if character_data.level != 1 {
            let mut char_repo = CharacterRepository::new(self.conn);
            let update = UpdateCharacter {
                character_name: None,
                is_npc: None,
                current_level: Some(character_data.level),
                current_version: None,
                updated_at: None,
                campaign_id: None,
                directory_path: None,
            };
            char_repo.update(character_id, update)?;
        }

        // Return the character
        let mut char_repo = CharacterRepository::new(self.conn);
        let character = char_repo
            .find_by_id(character_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Character".to_string(),
                id: character_id.to_string(),
            })?;

        Ok(character)
    }

    /// Get a character by ID with its latest version data
    pub fn get_character(&mut self, character_id: i32) -> Result<(Character, CharacterData)> {
        let mut char_repo = CharacterRepository::new(self.conn);
        let character = char_repo
            .find_by_id(character_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Character".to_string(),
                id: character_id.to_string(),
            })?;

        let mut ver_repo = CharacterVersionRepository::new(self.conn);
        let version = ver_repo
            .find_latest(character_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "CharacterVersion".to_string(),
                id: format!("character_id={}", character_id),
            })?;

        let character_data: CharacterData = serde_yaml::from_str(&version.character_data)
            .map_err(|e| DbError::InvalidData(format!("Failed to parse character data: {}", e)))?;

        Ok((character, character_data))
    }

    /// Update a character (creates a new version)
    pub fn update_character(
        &mut self,
        character_id: i32,
        character_data: CharacterData,
        snapshot_reason: Option<String>,
    ) -> Result<CharacterVersion> {
        // Get character and version number
        let character = {
            let mut char_repo = CharacterRepository::new(self.conn);
            char_repo
                .find_by_id(character_id)?
                .ok_or_else(|| DbError::NotFound {
                    entity_type: "Character".to_string(),
                    id: character_id.to_string(),
                })?
        };

        let version_number = {
            let mut ver_repo = CharacterVersionRepository::new(self.conn);
            ver_repo.get_next_version_number(character_id)?
        };

        // Serialize character data to YAML
        let yaml_data = serde_yaml::to_string(&character_data).map_err(|e| {
            DbError::InvalidData(format!("Failed to serialize character data: {}", e))
        })?;

        // Only write files if character has a directory assigned
        let file_path_str = if !character.directory_path.is_empty() {
            let char_dir = Path::new(&character.directory_path);
            let file_path = self.get_version_file_path(
                char_dir,
                &character_data.character_name,
                version_number,
            );

            // Generate markdown
            let renderer = MarkdownRenderer::new();
            let markdown = renderer.render(&character_data);

            // Write files
            self.write_character_files(&file_path, &yaml_data, &markdown)?;
            file_path.to_string_lossy().to_string()
        } else {
            String::new()
        };

        // Create version record
        let new_version = NewCharacterVersion {
            character_id,
            version_number,
            file_path: file_path_str,
            character_data: yaml_data,
            snapshot_reason,
            level: character_data.level,
        };

        let version = {
            let mut ver_repo = CharacterVersionRepository::new(self.conn);
            ver_repo.create(new_version)?
        };

        // Update character metadata
        let update = UpdateCharacter {
            character_name: None,
            is_npc: None,
            current_level: Some(character_data.level),
            current_version: Some(version_number),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            campaign_id: None,
            directory_path: None,
        };

        let mut char_repo = CharacterRepository::new(self.conn);
        char_repo.update(character_id, update)?;

        Ok(version)
    }

    /// Delete a character and all its files
    pub fn delete_character(&mut self, character_id: i32) -> Result<()> {
        let mut char_repo = CharacterRepository::new(self.conn);
        let character = char_repo
            .find_by_id(character_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Character".to_string(),
                id: character_id.to_string(),
            })?;

        // Delete directory and all files
        let char_dir = Path::new(&character.directory_path);
        if char_dir.exists() {
            fs::remove_dir_all(char_dir)?;
        }

        // Delete from database (cascades to character_versions)
        char_repo.delete(character_id)?;

        Ok(())
    }

    /// Assign a character to a campaign
    /// Creates character files in the campaign directory structure
    pub fn assign_to_campaign(
        &mut self,
        character_id: i32,
        campaign_id: i32,
        campaign_directory: &str,
    ) -> Result<Character> {
        // Get the character
        let character = {
            let mut char_repo = CharacterRepository::new(self.conn);
            char_repo
                .find_by_id(character_id)?
                .ok_or_else(|| DbError::NotFound {
                    entity_type: "Character".to_string(),
                    id: character_id.to_string(),
                })?
        };

        // Verify character is not already assigned
        if character.campaign_id.is_some() {
            return Err(DbError::InvalidData(
                "Character is already assigned to a campaign".to_string(),
            ));
        }

        // Create character directory
        let char_dir =
            self.create_character_directory(campaign_directory, &character.character_name)?;
        let directory_path = char_dir.to_string_lossy().to_string();

        // Get all versions and write them to files
        let versions = {
            let mut ver_repo = CharacterVersionRepository::new(self.conn);
            ver_repo.list_for_character(character_id)?
        };

        let renderer = MarkdownRenderer::new();

        for version in &versions {
            // Parse character data
            let character_data: CharacterData = serde_yaml::from_str(&version.character_data)
                .map_err(|e| {
                    DbError::InvalidData(format!("Failed to parse character data: {}", e))
                })?;

            // Generate file path and content
            let file_path = self.get_version_file_path(
                &char_dir,
                &character.character_name,
                version.version_number,
            );
            let markdown = renderer.render(&character_data);

            // Write file
            self.write_character_files(&file_path, &version.character_data, &markdown)?;

            // Update version's file_path in database
            let mut ver_repo = CharacterVersionRepository::new(self.conn);
            ver_repo.update_file_path(version.id, file_path.to_string_lossy().to_string())?;
        }

        // Update character with campaign_id and directory_path
        let update = UpdateCharacter {
            character_name: None,
            is_npc: None,
            current_level: None,
            current_version: None,
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            campaign_id: Some(Some(campaign_id)),
            directory_path: Some(directory_path),
        };

        let mut char_repo = CharacterRepository::new(self.conn);
        char_repo.update(character_id, update)
    }

    /// List all characters (including unassigned)
    pub fn list_all_characters(&mut self) -> Result<Vec<Character>> {
        let mut char_repo = CharacterRepository::new(self.conn);
        char_repo.list_all()
    }

    /// List all characters for a campaign
    pub fn list_characters_for_campaign(&mut self, campaign_id: i32) -> Result<Vec<Character>> {
        let mut char_repo = CharacterRepository::new(self.conn);
        char_repo.list_for_campaign(campaign_id)
    }

    /// List NPCs for a campaign
    pub fn list_npcs_for_campaign(&mut self, campaign_id: i32) -> Result<Vec<Character>> {
        let mut char_repo = CharacterRepository::new(self.conn);
        char_repo.list_npcs(campaign_id)
    }

    /// List player characters (non-NPCs) for a campaign
    pub fn list_pcs_for_campaign(&mut self, campaign_id: i32) -> Result<Vec<Character>> {
        let mut char_repo = CharacterRepository::new(self.conn);
        char_repo.list_pcs(campaign_id)
    }

    /// Get all versions for a character
    pub fn get_character_versions(&mut self, character_id: i32) -> Result<Vec<CharacterVersion>> {
        let mut ver_repo = CharacterVersionRepository::new(self.conn);
        ver_repo.list_for_character(character_id)
    }

    /// Get a specific version of a character
    pub fn get_character_version(
        &mut self,
        character_id: i32,
        version_number: i32,
    ) -> Result<CharacterData> {
        let mut ver_repo = CharacterVersionRepository::new(self.conn);
        let version = ver_repo
            .find_by_character_and_version(character_id, version_number)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "CharacterVersion".to_string(),
                id: format!("character_id={}, version={}", character_id, version_number),
            })?;

        let character_data: CharacterData = serde_yaml::from_str(&version.character_data)
            .map_err(|e| DbError::InvalidData(format!("Failed to parse character data: {}", e)))?;

        Ok(character_data)
    }

    /// Level up a character
    pub fn level_up_character(
        &mut self,
        character_id: i32,
        options: LevelUpOptions,
    ) -> Result<CharacterVersion> {
        // Get current character data
        let (_character, mut char_data) = self.get_character(character_id)?;

        // Get class info from database
        let class_info = ClassInfo::get(self.conn, &options.class_name, &options.class_source)?;

        // Check if this is multiclassing (adding a new class or leveling existing one)
        let is_multiclass = !char_data.has_class(&options.class_name);

        // Validate multiclass prerequisites
        if is_multiclass {
            if let Some(prereqs) =
                MulticlassPrerequisites::get(self.conn, &options.class_name, &options.class_source)?
            {
                prereqs.check(&char_data.abilities)?;
            }
        }

        // Validate HP gain
        options.validate_hp_gain(class_info.hit_die_value)?;

        // Calculate HP gain
        let con_modifier = char_data.abilities.con_modifier();
        let hp_gain = match options.hp_method {
            HpGainMethod::Roll(value) => value + con_modifier,
            HpGainMethod::Average => class_info.average_hp_gain() + con_modifier,
        };

        // Update HP
        char_data.max_hp += hp_gain;
        char_data.current_hp += hp_gain;

        // Increment total level
        char_data.level += 1;

        // Update class levels and hit dice
        if is_multiclass {
            // Adding a new class
            use crate::models::character::data::ClassLevel;
            char_data.classes.push(ClassLevel {
                class_name: options.class_name.clone(),
                level: 1,
                subclass: None,
                hit_dice_type: class_info.hit_die.clone(),
                hit_dice_remaining: 1,
            });
        } else {
            // Leveling existing class
            if let Some(class_level) = char_data.get_class_mut(&options.class_name) {
                class_level.level += 1;
                class_level.hit_dice_remaining += 1;
            }
        }

        // Check if this level grants ASI/Feat
        if class_info.asi_levels.contains(&char_data.level) {
            if let Some(choice) = &options.asi_or_feat {
                options.validate_asi_or_feat()?;

                match choice {
                    AsiOrFeat::AbilityScoreImprovement {
                        ability1,
                        increase1,
                        ability2,
                        increase2,
                    } => {
                        // Apply ability score increases (cap at 20)
                        self.apply_ability_increase(
                            &mut char_data.abilities,
                            ability1,
                            *increase1,
                        )?;

                        if let (Some(ability), Some(increase)) = (ability2, increase2) {
                            self.apply_ability_increase(
                                &mut char_data.abilities,
                                ability,
                                *increase,
                            )?;
                        }
                    }
                    AsiOrFeat::Feat(feat_name) => {
                        // Add feat to character
                        if !char_data.feats.contains(feat_name) {
                            char_data.feats.push(feat_name.clone());
                        }
                    }
                }
            }
        }

        // Update subclass if provided
        if let Some(subclass) = &options.subclass_choice {
            if let Some(class_level) = char_data.get_class_mut(&options.class_name) {
                class_level.subclass = Some(subclass.clone());
            }
        }

        // Update spell slots if character is a spellcaster
        // For now, we'll skip this complex logic and leave it for future enhancement

        // Update snapshot reason
        let snapshot_reason = options
            .snapshot_reason
            .or_else(|| Some(format!("Leveled up to {}", char_data.level)));

        // Create new version
        self.update_character(character_id, char_data, snapshot_reason)
    }

    // Helper method to apply ability score increase
    fn apply_ability_increase(
        &self,
        abilities: &mut crate::models::character::data::AbilityScores,
        ability_name: &str,
        increase: i32,
    ) -> Result<()> {
        let score = match ability_name.to_lowercase().as_str() {
            "strength" => &mut abilities.strength,
            "dexterity" => &mut abilities.dexterity,
            "constitution" => &mut abilities.constitution,
            "intelligence" => &mut abilities.intelligence,
            "wisdom" => &mut abilities.wisdom,
            "charisma" => &mut abilities.charisma,
            _ => {
                return Err(DbError::InvalidData(format!(
                    "Unknown ability: {}",
                    ability_name
                )))
            }
        };

        // Apply increase (cap at 20)
        *score = (*score + increase).min(20);
        Ok(())
    }

    // ===== Spell Management Methods =====

    /// Add a spell to a character's known spells
    ///
    /// Validates that the spell exists and is available for the character's class
    pub fn add_spell_to_known(
        &mut self,
        character_id: i32,
        spell_name: &str,
        spell_source: &str,
        is_cantrip: bool,
    ) -> Result<CharacterVersion> {
        let (_character, mut char_data) = self.get_character(character_id)?;

        // Get spell details from database
        let spell = crate::services::SpellService::get_spell_details(self.conn, spell_name, spell_source)?
            .ok_or_else(|| DbError::InvalidData(format!(
                "Spell '{}' from '{}' not found in database. Please import the appropriate rulebook first.",
                spell_name, spell_source
            )))?;

        // Validate spell is available for any of character's classes
        let class_names: Vec<String> = char_data
            .classes
            .iter()
            .map(|c| c.class_name.clone())
            .collect();
        let mut valid_for_any_class = false;
        for class_name in &class_names {
            if spell_management::validate_spell_for_class(self.conn, &spell, class_name)? {
                valid_for_any_class = true;
                break;
            }
        }
        if !valid_for_any_class {
            return Err(DbError::InvalidData(format!(
                "Spell '{}' is not available for any of character's classes: {}",
                spell_name,
                char_data.class_string()
            )));
        }

        // Add to appropriate list
        let spell_ref = crate::models::character::SpellReference::new(spell_name, spell_source);

        // Helper to check if spell already exists
        let spell_exists = |list: &[crate::models::character::SpellReference]| {
            list.iter().any(|s| s.name == spell_ref.name && s.source == spell_ref.source)
        };

        if is_cantrip {
            if !spell_exists(&char_data.spells.cantrips) {
                char_data.spells.cantrips.push(spell_ref);
            }
        } else if !spell_exists(&char_data.spells.known_spells) {
            char_data.spells.known_spells.push(spell_ref);
        }

        // Create new version
        let snapshot_reason = Some(format!("Learned spell: {}", spell_name));
        self.update_character(character_id, char_data, snapshot_reason)
    }

    /// Prepare spells for a character
    ///
    /// Validates preparation limits based on class and ability scores
    pub fn prepare_spells(
        &mut self,
        character_id: i32,
        spells: Vec<crate::models::character::SpellReference>,
        spellcasting_ability: &str,
    ) -> Result<CharacterVersion> {
        let (_character, mut char_data) = self.get_character(character_id)?;

        // Helper to check if a spell is in a list
        let spell_in_list = |spell: &crate::models::character::SpellReference,
                            list: &[crate::models::character::SpellReference]| {
            list.iter().any(|s| s.name == spell.name && s.source == spell.source)
        };

        // Calculate maximum prepared spells
        // Formula: spellcasting ability modifier + character level
        let ability_mod =
            spell_management::calculate_spell_attack_bonus(&char_data, spellcasting_ability)
                - char_data.proficiency_bonus();
        let max_prepared = (ability_mod + char_data.level).max(1);

        // Validate spell count (not including cantrips)
        let non_cantrip_count = spells
            .iter()
            .filter(|spell| !spell_in_list(spell, &char_data.spells.cantrips))
            .count();

        if non_cantrip_count > max_prepared as usize {
            return Err(DbError::InvalidData(format!(
                "Cannot prepare {} spells. Maximum is {} (ability modifier {} + level {})",
                non_cantrip_count, max_prepared, ability_mod, char_data.level
            )));
        }

        // Validate all spells are known
        for spell in &spells {
            if !spell_in_list(spell, &char_data.spells.cantrips)
                && !spell_in_list(spell, &char_data.spells.known_spells)
            {
                return Err(DbError::InvalidData(format!(
                    "Spell '{}' from '{}' is not known by this character",
                    spell.name, spell.source
                )));
            }
        }

        // Update prepared spells
        char_data.spells.prepared_spells = spells;

        // Create new version
        let snapshot_reason = Some("Updated prepared spells".to_string());
        self.update_character(character_id, char_data, snapshot_reason)
    }

    /// Cast a spell, consuming the appropriate spell slot
    pub fn cast_spell(
        &mut self,
        character_id: i32,
        spell_name: &str,
        spell_level: i32,
        is_ritual: bool,
    ) -> Result<CharacterVersion> {
        let (_character, mut char_data) = self.get_character(character_id)?;

        // Ritual casting doesn't consume slots
        if is_ritual {
            let snapshot_reason = Some(format!("Cast {} as ritual", spell_name));
            return self.update_character(character_id, char_data, snapshot_reason);
        }

        // Check if character has prepared the spell (or if it's a known spell for spontaneous casters)
        // For now, just check if spell is in known_spells or prepared_spells

        // Consume spell slot
        if let Some(slots) = char_data.spells.spell_slots.get_mut(&spell_level) {
            if !slots.expend(1) {
                return Err(DbError::InvalidData(format!(
                    "No level {} spell slots remaining",
                    spell_level
                )));
            }
        } else {
            return Err(DbError::InvalidData(format!(
                "Character has no level {} spell slots",
                spell_level
            )));
        }

        // Create new version
        let snapshot_reason = Some(format!("Cast {} (level {})", spell_name, spell_level));
        self.update_character(character_id, char_data, snapshot_reason)
    }

    /// Rest and restore spell slots
    pub fn rest(&mut self, character_id: i32, rest_type: RestType) -> Result<CharacterVersion> {
        let (_character, mut char_data) = self.get_character(character_id)?;

        match rest_type {
            RestType::Short => {
                // Short rest restores warlock pact magic slots
                if char_data.has_class("Warlock") {
                    for slots in char_data.spells.spell_slots.values_mut() {
                        slots.recover_all();
                    }
                }
                // Note: Class-specific short rest features (Arcane Recovery, etc.) are listed
                // on character sheets but actual resource tracking is handled at the table/VTT.
            }
            RestType::Long => {
                // Long rest restores all spell slots for all classes
                for slots in char_data.spells.spell_slots.values_mut() {
                    slots.recover_all();
                }

                // Also restore HP
                char_data.current_hp = char_data.max_hp;

                // Restore hit dice (restore half of max for each class, minimum 1 total)
                let total_restored = (char_data.level / 2).max(1);
                let mut remaining_to_restore = total_restored;

                for class_level in &mut char_data.classes {
                    let max_for_class = class_level.level;
                    let can_restore = max_for_class - class_level.hit_dice_remaining;
                    let to_restore = can_restore.min(remaining_to_restore);
                    class_level.hit_dice_remaining += to_restore;
                    remaining_to_restore -= to_restore;
                    if remaining_to_restore <= 0 {
                        break;
                    }
                }
            }
        }

        // Create new version
        let snapshot_reason = Some(format!("{:?} rest completed", rest_type));
        self.update_character(character_id, char_data, snapshot_reason)
    }

    // ===== Inventory Management Methods =====

    /// Add an item to character's inventory
    ///
    /// Validates that the item exists in the catalog_items database
    pub fn add_item(
        &mut self,
        character_id: i32,
        item_name: &str,
        item_source: &str,
        quantity: i32,
        notes: Option<String>,
    ) -> Result<CharacterVersion> {
        let (_character, mut char_data) = self.get_character(character_id)?;

        // Query item from database to get weight and value
        let mut item_service = crate::services::ItemService::new(self.conn);
        let item = item_service
            .get_item_by_name_and_source(item_name, item_source)
            .map_err(|e| DbError::InvalidData(format!("Failed to get item: {}", e)))?
            .ok_or_else(|| {
                DbError::InvalidData(format!(
                    "Item '{}' from '{}' not found in database. Please import the appropriate rulebook first.",
                    item_name, item_source
                ))
            })?;

        // Get weight and value from item
        let weight = item.weight.map(|w| w as f64).unwrap_or(0.0);
        let value = item.value.unwrap_or(0.0);

        // Build notes from item properties/entries, combined with user notes
        let item_description = build_item_description(&item);
        let final_notes = match (notes, item_description) {
            (Some(user_notes), Some(item_desc)) => Some(format!("{}\n\n{}", item_desc, user_notes)),
            (Some(user_notes), None) => Some(user_notes),
            (None, Some(item_desc)) => Some(item_desc),
            (None, None) => None,
        };

        // Check if item already exists in inventory
        let existing_item = char_data.inventory.iter_mut().find(|i| i.name == item_name);

        if let Some(existing) = existing_item {
            // Item exists - add to quantity
            existing.quantity += quantity;
        } else {
            // New item - add to inventory
            char_data
                .inventory
                .push(crate::models::character::data::InventoryItem {
                    name: item_name.to_string(),
                    source: Some(item_source.to_string()),
                    quantity,
                    weight,
                    value,
                    notes: final_notes,
                });
        }

        // Create new version
        let snapshot_reason = Some(format!("Added {} x{} to inventory", item_name, quantity));
        self.update_character(character_id, char_data, snapshot_reason)
    }

    /// Remove an item from character's inventory
    pub fn remove_item(
        &mut self,
        character_id: i32,
        item_name: &str,
        quantity: i32,
    ) -> Result<CharacterVersion> {
        let (_character, mut char_data) = self.get_character(character_id)?;

        // Find the item
        let item_index = char_data
            .inventory
            .iter()
            .position(|i| i.name == item_name)
            .ok_or_else(|| {
                DbError::InvalidData(format!("Item '{}' not found in inventory", item_name))
            })?;

        // Reduce quantity or remove item
        let item = &mut char_data.inventory[item_index];
        if item.quantity <= quantity {
            // Remove item entirely
            char_data.inventory.remove(item_index);
        } else {
            // Reduce quantity
            item.quantity -= quantity;
        }

        // Create new version
        let snapshot_reason = Some(format!(
            "Removed {} x{} from inventory",
            item_name, quantity
        ));
        self.update_character(character_id, char_data, snapshot_reason)
    }

    /// Update character's currency
    ///
    /// Adds or subtracts currency. Use negative values to subtract.
    pub fn update_currency(
        &mut self,
        character_id: i32,
        copper: i32,
        silver: i32,
        electrum: i32,
        gold: i32,
        platinum: i32,
    ) -> Result<CharacterVersion> {
        let (_character, mut char_data) = self.get_character(character_id)?;

        // Update currency values
        char_data.currency.copper += copper;
        char_data.currency.silver += silver;
        char_data.currency.electrum += electrum;
        char_data.currency.gold += gold;
        char_data.currency.platinum += platinum;

        // Validate no negative currency
        if char_data.currency.copper < 0
            || char_data.currency.silver < 0
            || char_data.currency.electrum < 0
            || char_data.currency.gold < 0
            || char_data.currency.platinum < 0
        {
            return Err(DbError::InvalidData(
                "Currency cannot be negative".to_string(),
            ));
        }

        // Create new version
        let snapshot_reason = Some("Updated currency".to_string());
        self.update_character(character_id, char_data, snapshot_reason)
    }

    /// Update character's equipped items
    pub fn update_equipped(
        &mut self,
        character_id: i32,
        armor: Option<String>,
        shield: Option<String>,
        main_hand: Option<String>,
        off_hand: Option<String>,
    ) -> Result<CharacterVersion> {
        let (_character, mut char_data) = self.get_character(character_id)?;

        // Update equipped items
        char_data.equipped.armor = armor;
        char_data.equipped.shield = shield;
        char_data.equipped.main_hand = main_hand;
        char_data.equipped.off_hand = off_hand;

        // Create new version
        let snapshot_reason = Some("Updated equipped items".to_string());
        self.update_character(character_id, char_data, snapshot_reason)
    }

    // Helper methods

    fn create_character_directory(
        &self,
        campaign_dir: &str,
        character_name: &str,
    ) -> Result<PathBuf> {
        let campaign_path = Path::new(campaign_dir);
        let characters_dir = campaign_path.join("characters");

        // Create characters directory if it doesn't exist
        if !characters_dir.exists() {
            fs::create_dir_all(&characters_dir)?;
        }

        // Create character-specific directory
        let char_dir = characters_dir.join(character_name);
        if !char_dir.exists() {
            fs::create_dir_all(&char_dir)?;
        }

        Ok(char_dir)
    }

    fn get_version_file_path(
        &self,
        char_dir: &Path,
        character_name: &str,
        version: i32,
    ) -> PathBuf {
        char_dir.join(format!("{}-{:03}.md", character_name, version))
    }

    fn write_character_files(
        &self,
        file_path: &Path,
        yaml_data: &str,
        markdown: &str,
    ) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write YAML data as a comment at the top of the markdown file
        let full_content = format!(
            "<!--\nCharacter Data (YAML):\n{}\n-->\n\n{}",
            yaml_data, markdown
        );

        fs::write(file_path, full_content)?;

        Ok(())
    }
}

/// Build a description string from an item's properties and entries
fn build_item_description(item: &crate::models::catalog::item::Item) -> Option<String> {
    let mut parts = Vec::new();

    // Add weapon properties if present
    if let Some(ref props) = item.property {
        let prop_names: Vec<&str> = props
            .iter()
            .filter_map(|p| match p.as_str() {
                "A" => Some("Ammunition"),
                "F" => Some("Finesse"),
                "H" => Some("Heavy"),
                "L" => Some("Light"),
                "LD" => Some("Loading"),
                "R" => Some("Reach"),
                "S" => Some("Special"),
                "T" => Some("Thrown"),
                "2H" => Some("Two-Handed"),
                "V" => Some("Versatile"),
                "M" => Some("Martial"),
                "AF" => Some("Ammunition (Firearm)"),
                "RLD" => Some("Reload"),
                "BF" => Some("Burst Fire"),
                _ => None,
            })
            .collect();
        if !prop_names.is_empty() {
            parts.push(format!("Properties: {}", prop_names.join(", ")));
        }
    }

    // Add damage info if present
    if let Some(ref dmg) = item.dmg1 {
        let dmg_type = item.dmg_type.as_deref().unwrap_or("");
        parts.push(format!("Damage: {} {}", dmg, dmg_type));
    }

    // Add range if present
    if let Some(ref range) = item.range {
        parts.push(format!("Range: {}", range));
    }

    // Add entries/description if present
    if !item.entries.is_empty() {
        let text = extract_entries_text_typed(&item.entries);
        if !text.is_empty() {
            parts.push(text);
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(". "))
    }
}

/// Extract plain text from 5etools entries array (typed Entry version)
fn extract_entries_text_typed(entries: &[crate::models::catalog::types::Entry]) -> String {
    use crate::models::catalog::types::{Entry, EntryObject};
    let raw = entries
        .iter()
        .filter_map(|entry| match entry {
            Entry::Text(s) => Some(s.clone()),
            Entry::Object(obj) => match obj {
                EntryObject::Entries { entries, .. } => Some(extract_entries_text_typed(entries)),
                EntryObject::List { items, .. } => Some(extract_entries_text_typed(items)),
                EntryObject::Item { name, entries, .. } => {
                    if let Some(entries) = entries {
                        Some(format!("{}: {}", name, extract_entries_text_typed(entries)))
                    } else {
                        Some(name.clone())
                    }
                }
                _ => None,
            },
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Clean 5etools tags like {@damage 1d6}, {@dice 2d6}, {@hit 5}, etc.
    clean_5etools_tags(&raw)
}

/// Remove 5etools formatting tags and extract just the content
fn clean_5etools_tags(text: &str) -> String {
    let mut result = text.to_string();

    // Pattern: {@tagname content} or {@tagname content|display}
    // We want to extract either 'display' if present, otherwise 'content'
    loop {
        if let Some(start) = result.find("{@") {
            if let Some(end) = result[start..].find('}') {
                let tag_content = &result[start + 2..start + end];
                // Find the tag name (first word)
                let replacement = if let Some(space_pos) = tag_content.find(' ') {
                    let content = &tag_content[space_pos + 1..];
                    // Check for pipe - if present, use text after pipe
                    if let Some(pipe_pos) = content.find('|') {
                        content[pipe_pos + 1..].to_string()
                    } else {
                        content.to_string()
                    }
                } else {
                    String::new()
                };
                result = format!("{}{}{}", &result[..start], replacement, &result[start + end + 1..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::establish_connection;
    use crate::models::character::data::*;
    use diesel::prelude::*;
    use tempfile::TempDir;

    fn setup_test_db() -> DbConnection {
        let mut conn = establish_connection(":memory:").expect("Failed to create in-memory DB");
        crate::run_migrations(&mut conn).expect("Failed to run migrations");
        insert_test_classes(&mut conn);
        conn
    }

    fn insert_test_classes(conn: &mut DbConnection) {
        use diesel::prelude::*;

        // Insert Fighter class
        let fighter_json = r#"{
            "name": "Fighter",
            "source": "PHB",
            "hd": {"number": 1, "faces": 10},
            "proficiency": ["str", "con"],
            "casterProgression": null
        }"#;

        diesel::insert_into(crate::schema::catalog_classes::table)
            .values((
                crate::schema::catalog_classes::name.eq("Fighter"),
                crate::schema::catalog_classes::source.eq("PHB"),
                crate::schema::catalog_classes::hit_dice.eq("d10"),
                crate::schema::catalog_classes::full_class_json.eq(fighter_json),
            ))
            .execute(conn)
            .expect("Failed to insert Fighter class");

        // Insert Barbarian class with multiclass requirements
        let barbarian_json = r#"{
            "name": "Barbarian",
            "source": "PHB",
            "hd": {"number": 1, "faces": 12},
            "proficiency": ["str", "con"],
            "casterProgression": null,
            "multiclassing": {"requirements": {"str": 13}}
        }"#;

        diesel::insert_into(crate::schema::catalog_classes::table)
            .values((
                crate::schema::catalog_classes::name.eq("Barbarian"),
                crate::schema::catalog_classes::source.eq("PHB"),
                crate::schema::catalog_classes::hit_dice.eq("d12"),
                crate::schema::catalog_classes::full_class_json.eq(barbarian_json),
            ))
            .execute(conn)
            .expect("Failed to insert Barbarian class");

        // Insert Monk class with multiclass requirements
        let monk_json = r#"{
            "name": "Monk",
            "source": "PHB",
            "hd": {"number": 1, "faces": 8},
            "proficiency": ["str", "dex"],
            "casterProgression": null,
            "multiclassing": {"requirements": {"dex": 13, "wis": 13}}
        }"#;

        diesel::insert_into(crate::schema::catalog_classes::table)
            .values((
                crate::schema::catalog_classes::name.eq("Monk"),
                crate::schema::catalog_classes::source.eq("PHB"),
                crate::schema::catalog_classes::hit_dice.eq("d8"),
                crate::schema::catalog_classes::full_class_json.eq(monk_json),
            ))
            .execute(conn)
            .expect("Failed to insert Monk class");
    }

    fn create_test_campaign(conn: &mut DbConnection) -> i32 {
        diesel::insert_into(crate::schema::campaigns::table)
            .values((
                crate::schema::campaigns::name.eq("Test Campaign"),
                crate::schema::campaigns::status.eq("active"),
                crate::schema::campaigns::directory_path.eq("/test"),
            ))
            .returning(crate::models::campaign::Campaign::as_returning())
            .get_result(conn)
            .expect("Failed to create campaign")
            .id
    }

    fn create_test_player(conn: &mut DbConnection) -> i32 {
        diesel::insert_into(crate::schema::players::table)
            .values((crate::schema::players::name.eq("Test Player"),))
            .returning(crate::models::player::Player::as_returning())
            .get_result(conn)
            .expect("Failed to create player")
            .id
    }

    fn create_test_character_data() -> CharacterData {
        use crate::models::character::data::ClassLevel;
        CharacterData {
            character_name: "Test Character".to_string(),
            player_id: Some(1),
            level: 1,
            experience_points: 0,
            version: 1,
            snapshot_reason: Some("Initial creation".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            race: "Human".to_string(),
            subrace: None,
            classes: vec![ClassLevel {
                class_name: "Fighter".to_string(),
                level: 1,
                subclass: None,
                hit_dice_type: "d10".to_string(),
                hit_dice_remaining: 1,
            }],
            background: "Soldier".to_string(),
            alignment: Some("Lawful Good".to_string()),
            abilities: AbilityScores {
                strength: 16,
                dexterity: 14,
                constitution: 15,
                intelligence: 10,
                wisdom: 12,
                charisma: 8,
            },
            max_hp: 12,
            current_hp: 12,
            proficiencies: Proficiencies {
                skills: vec!["Athletics".to_string(), "Intimidation".to_string()],
                saves: vec!["Strength".to_string(), "Constitution".to_string()],
                armor: vec!["Light Armor".to_string(), "Medium Armor".to_string()],
                weapons: vec!["Simple Weapons".to_string(), "Martial Weapons".to_string()],
                tools: vec![],
                languages: vec!["Common".to_string()],
            },
            class_features: vec![],
            feats: vec![],
            spells: SpellData::default(),
            inventory: vec![],
            currency: Currency::default(),
            speed: 30, // Human speed
            equipped: EquippedItems {
                armor: None,
                shield: None,
                main_hand: None,
                off_hand: None,
            },
            personality: Personality {
                traits: None,
                ideals: None,
                bonds: None,
                flaws: None,
            },
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: Vec::new(),
            legendary_action_count: None,
        }
    }

    #[test]
    fn test_create_character() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let character = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        assert_eq!(character.character_name, "Test Character");
        assert_eq!(character.campaign_id, Some(campaign_id));
        assert_eq!(character.player_id, Some(player_id));
        assert_eq!(character.is_npc, false);
        assert_eq!(character.current_level, 1);
        assert_eq!(character.current_version, 1);

        // Verify directory structure was created
        let char_dir = Path::new(campaign_dir)
            .join("characters")
            .join("Test Character");
        assert!(char_dir.exists());

        // Verify file was created
        let file_path = char_dir.join("Test Character-001.md");
        assert!(file_path.exists());

        // Verify file content contains both YAML and markdown
        let content = fs::read_to_string(&file_path).expect("Failed to read file");
        assert!(content.contains("Character Data (YAML)"));
        assert!(content.contains("# Test Character"));
    }

    #[test]
    fn test_create_character_empty_name() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.character_name = "".to_string();
        character_data.player_id = Some(player_id);

        let result =
            service.create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_character() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        let (character, data) = service
            .get_character(created.id)
            .expect("Failed to get character");

        assert_eq!(character.id, created.id);
        assert_eq!(data.character_name, "Test Character");
        assert_eq!(data.level, 1);
        assert_eq!(data.race, "Human");
        assert_eq!(data.primary_class_name(), "Fighter");
    }

    #[test]
    fn test_get_character_not_found() {
        let mut conn = setup_test_db();
        let mut service = CharacterService::new(&mut conn);

        let result = service.get_character(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_character() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(
                Some(campaign_id),
                Some(player_id),
                false,
                campaign_dir,
                character_data.clone(),
            )
            .expect("Failed to create character");

        // Update character (level up)
        character_data.level = 2;
        character_data.experience_points = 300;
        character_data.max_hp = 20;
        character_data.current_hp = 20;

        let version = service
            .update_character(
                created.id,
                character_data,
                Some("Level up to 2".to_string()),
            )
            .expect("Failed to update character");

        assert_eq!(version.version_number, 2);
        assert_eq!(version.snapshot_reason, Some("Level up to 2".to_string()));
        assert_eq!(version.level, 2);

        // Verify character metadata was updated
        let (character, data) = service
            .get_character(created.id)
            .expect("Failed to get character");
        assert_eq!(character.current_level, 2);
        assert_eq!(character.current_version, 2);
        assert_eq!(data.level, 2);
        assert_eq!(data.experience_points, 300);

        // Verify version 2 file was created
        let char_dir = Path::new(&character.directory_path);
        let file_path = char_dir.join("Test Character-002.md");
        assert!(file_path.exists());
    }

    #[test]
    fn test_delete_character() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        let char_dir = Path::new(&created.directory_path);
        assert!(char_dir.exists());

        service
            .delete_character(created.id)
            .expect("Failed to delete character");

        // Verify directory was removed
        assert!(!char_dir.exists());

        // Verify database record was removed
        let result = service.get_character(created.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_characters_for_campaign() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);

        // Create multiple characters
        let mut char1 = create_test_character_data();
        char1.character_name = "Character 1".to_string();
        char1.player_id = Some(player_id);
        service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, char1)
            .unwrap();

        let mut char2 = create_test_character_data();
        char2.character_name = "Character 2".to_string();
        char2.player_id = Some(player_id);
        service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, char2)
            .unwrap();

        let characters = service
            .list_characters_for_campaign(campaign_id)
            .expect("Failed to list characters");

        assert_eq!(characters.len(), 2);
    }

    #[test]
    fn test_get_character_versions() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(
                Some(campaign_id),
                Some(player_id),
                false,
                campaign_dir,
                character_data.clone(),
            )
            .expect("Failed to create character");

        // Create a second version
        character_data.level = 2;
        service
            .update_character(created.id, character_data, Some("Level 2".to_string()))
            .unwrap();

        let versions = service
            .get_character_versions(created.id)
            .expect("Failed to get versions");

        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version_number, 1);
        assert_eq!(versions[1].version_number, 2);
    }

    #[test]
    fn test_get_character_version() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(
                Some(campaign_id),
                Some(player_id),
                false,
                campaign_dir,
                character_data.clone(),
            )
            .expect("Failed to create character");

        // Create a second version
        character_data.level = 2;
        service
            .update_character(created.id, character_data, Some("Level 2".to_string()))
            .unwrap();

        // Get version 1
        let v1_data = service
            .get_character_version(created.id, 1)
            .expect("Failed to get version 1");
        assert_eq!(v1_data.level, 1);

        // Get version 2
        let v2_data = service
            .get_character_version(created.id, 2)
            .expect("Failed to get version 2");
        assert_eq!(v2_data.level, 2);
    }

    #[test]
    fn test_get_character_version_not_found() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        // Try to get non-existent version
        let result = service.get_character_version(created.id, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_level_up_with_hp_roll() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        // Level up with HP roll
        let level_up_options = LevelUpOptions {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Roll(8),
            asi_or_feat: None,
            subclass_choice: None,
            snapshot_reason: Some("Test level up".to_string()),
        };

        let version = service
            .level_up_character(created.id, level_up_options)
            .expect("Failed to level up");

        assert_eq!(version.version_number, 2);
        assert_eq!(version.level, 2);

        // Get updated character
        let (_character, data) = service
            .get_character(created.id)
            .expect("Failed to get character");
        assert_eq!(data.level, 2);
        // HP should be 12 (initial) + 8 (roll) + 2 (CON modifier) = 22
        assert_eq!(data.max_hp, 22);
    }

    #[test]
    fn test_level_up_with_average_hp() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        // Level up with average HP
        let level_up_options = LevelUpOptions {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Average,
            asi_or_feat: None,
            subclass_choice: None,
            snapshot_reason: None,
        };

        service
            .level_up_character(created.id, level_up_options)
            .expect("Failed to level up");

        // Get updated character
        let (_character, data) = service
            .get_character(created.id)
            .expect("Failed to get character");
        assert_eq!(data.level, 2);
        // HP should be 12 (initial) + 6 (average for d10) + 2 (CON modifier) = 20
        assert_eq!(data.max_hp, 20);
    }

    #[test]
    fn test_level_up_with_asi() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        // Level up to 2, 3 (no ASI)
        for _ in 0..2 {
            let options = LevelUpOptions {
                class_name: "Fighter".to_string(),
                class_source: "PHB".to_string(),
                hp_method: HpGainMethod::Average,
                asi_or_feat: None,
                subclass_choice: None,
                snapshot_reason: None,
            };
            service.level_up_character(created.id, options).unwrap();
        }

        // Level up to 4 with ASI
        let options = LevelUpOptions {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Average,
            asi_or_feat: Some(AsiOrFeat::AbilityScoreImprovement {
                ability1: "Strength".to_string(),
                increase1: 2,
                ability2: None,
                increase2: None,
            }),
            subclass_choice: None,
            snapshot_reason: Some("Level 4 with ASI".to_string()),
        };

        service
            .level_up_character(created.id, options)
            .expect("Failed to level up with ASI");

        // Get updated character
        let (_character, data) = service
            .get_character(created.id)
            .expect("Failed to get character");
        assert_eq!(data.level, 4);
        // Strength should be 18 (16 + 2)
        assert_eq!(data.abilities.strength, 18);
    }

    #[test]
    fn test_level_up_with_feat() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        // Level up to 4 with feat
        for _ in 0..2 {
            let options = LevelUpOptions {
                class_name: "Fighter".to_string(),
                class_source: "PHB".to_string(),
                hp_method: HpGainMethod::Average,
                asi_or_feat: None,
                subclass_choice: None,
                snapshot_reason: None,
            };
            service.level_up_character(created.id, options).unwrap();
        }

        let options = LevelUpOptions {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Average,
            asi_or_feat: Some(AsiOrFeat::Feat("Great Weapon Master".to_string())),
            subclass_choice: None,
            snapshot_reason: None,
        };

        service
            .level_up_character(created.id, options)
            .expect("Failed to level up with feat");

        // Get updated character
        let (_character, data) = service
            .get_character(created.id)
            .expect("Failed to get character");
        assert_eq!(data.level, 4);
        assert!(data.feats.contains(&"Great Weapon Master".to_string()));
    }

    #[test]
    fn test_level_up_multiclass_valid() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);
        // Set STR to 13 to meet Barbarian multiclass prerequisite
        character_data.abilities.strength = 16;

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        // Multiclass into Barbarian
        let options = LevelUpOptions {
            class_name: "Barbarian".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Average,
            asi_or_feat: None,
            subclass_choice: None,
            snapshot_reason: Some("Multiclass to Barbarian".to_string()),
        };

        service
            .level_up_character(created.id, options)
            .expect("Failed to multiclass");

        // Get updated character
        let (_character, data) = service
            .get_character(created.id)
            .expect("Failed to get character");
        assert_eq!(data.level, 2);
        assert!(data.has_class("Barbarian"));
    }

    #[test]
    fn test_level_up_multiclass_invalid_prerequisites() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);
        // DEX is 14, but Monk requires DEX 13 AND WIS 13
        // WIS is only 12, so should fail
        character_data.abilities.dexterity = 14;
        character_data.abilities.wisdom = 12;

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        // Try to multiclass into Monk (should fail due to low WIS)
        let options = LevelUpOptions {
            class_name: "Monk".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Average,
            asi_or_feat: None,
            subclass_choice: None,
            snapshot_reason: None,
        };

        let result = service.level_up_character(created.id, options);
        assert!(result.is_err());
    }

    #[test]
    fn test_level_up_ability_score_cap_at_20() {
        let mut conn = setup_test_db();
        let campaign_id = create_test_campaign(&mut conn);
        let player_id = create_test_player(&mut conn);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let campaign_dir = temp_dir.path().to_str().unwrap();

        let mut service = CharacterService::new(&mut conn);
        let mut character_data = create_test_character_data();
        character_data.player_id = Some(player_id);
        // Start with STR 19
        character_data.abilities.strength = 19;

        let created = service
            .create_character(Some(campaign_id), Some(player_id), false, campaign_dir, character_data)
            .expect("Failed to create character");

        // Level up to 4 with +2 STR
        for _ in 0..2 {
            let options = LevelUpOptions {
                class_name: "Fighter".to_string(),
                class_source: "PHB".to_string(),
                hp_method: HpGainMethod::Average,
                asi_or_feat: None,
                subclass_choice: None,
                snapshot_reason: None,
            };
            service.level_up_character(created.id, options).unwrap();
        }

        let options = LevelUpOptions {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Average,
            asi_or_feat: Some(AsiOrFeat::AbilityScoreImprovement {
                ability1: "Strength".to_string(),
                increase1: 2,
                ability2: None,
                increase2: None,
            }),
            subclass_choice: None,
            snapshot_reason: None,
        };

        service
            .level_up_character(created.id, options)
            .expect("Failed to level up");

        // Get updated character
        let (_character, data) = service
            .get_character(created.id)
            .expect("Failed to get character");
        // STR should be capped at 20 (not 21)
        assert_eq!(data.abilities.strength, 20);
    }
}
