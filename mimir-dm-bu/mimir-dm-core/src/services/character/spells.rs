//! Character spell management service

use super::{spell_management, CharacterService, RestType};
use crate::{
    connection::DbConnection,
    error::{DbError, Result},
    models::character::CharacterVersion,
};

/// Service for character spell operations
pub struct CharacterSpellService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CharacterSpellService<'a> {
    /// Create a new spell service
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

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
        let mut char_service = CharacterService::new(self.conn);
        let (_character, mut char_data) = char_service.get_character(character_id)?;

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
        let mut char_service = CharacterService::new(self.conn);
        char_service.update_character(character_id, char_data, snapshot_reason)
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
        let mut char_service = CharacterService::new(self.conn);
        let (_character, mut char_data) = char_service.get_character(character_id)?;

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
        let mut char_service = CharacterService::new(self.conn);
        char_service.update_character(character_id, char_data, snapshot_reason)
    }

    /// Cast a spell, consuming the appropriate spell slot
    pub fn cast_spell(
        &mut self,
        character_id: i32,
        spell_name: &str,
        spell_level: i32,
        is_ritual: bool,
    ) -> Result<CharacterVersion> {
        let mut char_service = CharacterService::new(self.conn);
        let (_character, mut char_data) = char_service.get_character(character_id)?;

        // Ritual casting doesn't consume slots
        if is_ritual {
            let snapshot_reason = Some(format!("Cast {} as ritual", spell_name));
            let mut char_service = CharacterService::new(self.conn);
            return char_service.update_character(character_id, char_data, snapshot_reason);
        }

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
        let mut char_service = CharacterService::new(self.conn);
        char_service.update_character(character_id, char_data, snapshot_reason)
    }

    /// Rest and restore spell slots
    pub fn rest(&mut self, character_id: i32, rest_type: RestType) -> Result<CharacterVersion> {
        let mut char_service = CharacterService::new(self.conn);
        let (_character, mut char_data) = char_service.get_character(character_id)?;

        match rest_type {
            RestType::Short => {
                // Short rest restores warlock spell slots
                // Check if character has warlock levels
                if char_data.has_class("Warlock") {
                    // Restore all spell slots for warlock
                    for slots in char_data.spells.spell_slots.values_mut() {
                        slots.recover_all();
                    }
                }
                // Other classes may have features that restore on short rest (e.g., Arcane Recovery)
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
        let mut char_service = CharacterService::new(self.conn);
        char_service.update_character(character_id, char_data, snapshot_reason)
    }
}
