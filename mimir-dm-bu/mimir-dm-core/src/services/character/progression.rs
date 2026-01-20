//! Character progression service for level up operations

use super::{
    AsiOrFeat, CharacterService, ClassInfo, HpGainMethod, LevelUpOptions, MulticlassPrerequisites,
};
use crate::{
    connection::DbConnection,
    error::{DbError, Result},
    models::character::CharacterVersion,
};

/// Service for character progression operations (level up, ASI, multiclassing)
pub struct CharacterProgressionService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CharacterProgressionService<'a> {
    /// Create a new progression service
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Level up a character
    pub fn level_up(
        &mut self,
        character_id: i32,
        options: LevelUpOptions,
    ) -> Result<CharacterVersion> {
        // Get current character data
        let mut char_service = CharacterService::new(self.conn);
        let (_character, mut char_data) = char_service.get_character(character_id)?;

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
                        Self::apply_ability_increase(
                            &mut char_data.abilities,
                            ability1,
                            *increase1,
                        )?;

                        if let (Some(ability), Some(increase)) = (ability2, increase2) {
                            Self::apply_ability_increase(
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

        // Update snapshot reason
        let snapshot_reason = options
            .snapshot_reason
            .or_else(|| Some(format!("Leveled up to {}", char_data.level)));

        // Create new version
        let mut char_service = CharacterService::new(self.conn);
        char_service.update_character(character_id, char_data, snapshot_reason)
    }

    /// Apply ability score increase (helper method)
    fn apply_ability_increase(
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
}
