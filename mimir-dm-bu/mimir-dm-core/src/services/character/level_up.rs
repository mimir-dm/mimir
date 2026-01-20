//! Character level up logic and progression
//!
//! Handles level advancement including HP calculation, ASI, feats, and multiclassing.

// Level up types have documented variants but many internal fields
#![allow(missing_docs)]

use crate::connection::DbConnection;
use crate::error::{DbError, Result};
use crate::models::character::data::AbilityScores;
use crate::services::ClassService;
use serde::{Deserialize, Serialize};

/// Options for leveling up a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelUpOptions {
    /// Class to level up in (allows multiclassing)
    pub class_name: String,

    /// Source book for the class (e.g., "PHB", "XGE")
    #[serde(default = "default_source")]
    pub class_source: String,

    /// HP gain method
    pub hp_method: HpGainMethod,

    /// Ability score improvement or feat selection (if applicable at this level)
    pub asi_or_feat: Option<AsiOrFeat>,

    /// Subclass choice (if this is the level where subclass is chosen)
    pub subclass_choice: Option<String>,

    /// Reason for this level up (e.g., "Leveled up after defeating dragon")
    pub snapshot_reason: Option<String>,
}

fn default_source() -> String {
    "PHB".to_string()
}

/// Method for gaining HP on level up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HpGainMethod {
    /// Roll the hit die (value is the roll result)
    Roll(i32),

    /// Take the average (rounded up)
    Average,
}

/// Ability Score Improvement or Feat selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsiOrFeat {
    /// Improve two ability scores by 1 each, or one by 2
    AbilityScoreImprovement {
        ability1: String,
        increase1: i32,
        ability2: Option<String>,
        increase2: Option<i32>,
    },

    /// Take a feat instead of ASI
    Feat(String),
}

/// Class information for level progression
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub hit_die: String,
    pub hit_die_value: i32,
    pub spellcasting_type: Option<SpellcastingType>,
    pub asi_levels: Vec<i32>,
}

/// Type of spellcasting progression
#[derive(Debug, Clone, PartialEq)]
pub enum SpellcastingType {
    Full,    // Wizard, Cleric, etc.
    Half,    // Paladin, Ranger
    Third,   // Eldritch Knight, Arcane Trickster
    Warlock, // Unique progression
}

/// Multiclassing prerequisites (minimum ability scores)
pub struct MulticlassPrerequisites {
    pub class_name: String,
    pub required_abilities: Vec<(String, i32)>,
}

impl LevelUpOptions {
    /// Validate HP gain for a given hit die
    pub fn validate_hp_gain(&self, hit_die_value: i32) -> Result<()> {
        match &self.hp_method {
            HpGainMethod::Roll(value) => {
                if *value < 1 || *value > hit_die_value {
                    return Err(DbError::InvalidData(format!(
                        "HP roll {} is invalid for hit die d{}",
                        value, hit_die_value
                    )));
                }
            }
            HpGainMethod::Average => {}
        }
        Ok(())
    }

    /// Validate ASI/Feat choice
    pub fn validate_asi_or_feat(&self) -> Result<()> {
        if let Some(choice) = &self.asi_or_feat {
            match choice {
                AsiOrFeat::AbilityScoreImprovement {
                    ability1,
                    increase1,
                    ability2,
                    increase2,
                } => {
                    // Validate ability names
                    if !Self::is_valid_ability(ability1) {
                        return Err(DbError::InvalidData(format!(
                            "Invalid ability score: {}",
                            ability1
                        )));
                    }

                    // Validate increases
                    if *increase1 < 1 || *increase1 > 2 {
                        return Err(DbError::InvalidData(
                            "Ability score increase must be 1 or 2".to_string(),
                        ));
                    }

                    if let (Some(ability), Some(increase)) = (ability2, increase2) {
                        if !Self::is_valid_ability(ability) {
                            return Err(DbError::InvalidData(format!(
                                "Invalid ability score: {}",
                                ability
                            )));
                        }

                        if *increase < 1 || *increase > 2 {
                            return Err(DbError::InvalidData(
                                "Ability score increase must be 1 or 2".to_string(),
                            ));
                        }

                        // Total increase must be 2
                        if increase1 + increase > 2 {
                            return Err(DbError::InvalidData(
                                "Total ability score increase must be exactly 2".to_string(),
                            ));
                        }
                    }
                }
                AsiOrFeat::Feat(feat_name) => {
                    if feat_name.trim().is_empty() {
                        return Err(DbError::InvalidData(
                            "Feat name cannot be empty".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn is_valid_ability(name: &str) -> bool {
        matches!(
            name.to_lowercase().as_str(),
            "strength" | "dexterity" | "constitution" | "intelligence" | "wisdom" | "charisma"
        )
    }
}

impl ClassInfo {
    /// Get class information by name from database
    pub fn get(conn: &mut DbConnection, class_name: &str, source: &str) -> Result<Self> {
        let mut class_service = ClassService::new(conn);

        let class = class_service
            .get_class_by_name_and_source(class_name, source)
            .map_err(|e| DbError::InvalidData(format!("Failed to get class: {}", e)))?
            .ok_or_else(|| {
                DbError::InvalidData(format!(
                    "Class '{}' from '{}' not found in database. Please import the appropriate rulebook first.",
                    class_name, source
                ))
            })?;

        // Parse hit die from typed HitDice struct
        let (hit_die, hit_die_value) = if let Some(hd) = &class.hd {
            let faces = hd.faces as i32;
            (format!("d{}", faces), faces)
        } else {
            ("d6".to_string(), 6)
        };

        // Parse spellcasting type
        let spellcasting_type = if let Some(caster_prog) = &class.caster_progression {
            match caster_prog.as_str() {
                "full" => Some(SpellcastingType::Full),
                "1/2" | "half" => Some(SpellcastingType::Half),
                "1/3" | "third" => Some(SpellcastingType::Third),
                "pact" => Some(SpellcastingType::Warlock),
                _ => None,
            }
        } else {
            None
        };

        // Parse ASI levels from class table groups
        let asi_levels = Self::parse_asi_levels(&class);

        Ok(ClassInfo {
            name: class.name.clone(),
            hit_die,
            hit_die_value,
            spellcasting_type,
            asi_levels,
        })
    }

    /// Parse ASI levels from class table groups
    fn parse_asi_levels(class: &crate::models::catalog::Class) -> Vec<i32> {
        // Default ASI levels for most classes
        let default_asi = vec![4, 8, 12, 16, 19];

        // Check if class table groups contain ASI information
        if let Some(table_groups) = &class.class_table_groups {
            for group in table_groups {
                // Look for "Ability Score Improvement" or similar columns
                if let Some(col_labels) = &group.col_labels {
                    for label in col_labels {
                        if label.contains("Ability Score") || label.contains("ASI") {
                            // Found ASI column, now extract levels from rows
                            if let Some(rows) = &group.rows {
                                let mut levels = Vec::new();
                                for (idx, row) in rows.iter().enumerate() {
                                    // Check if this row has an ASI marker
                                    for cell in row {
                                        if let Some(cell_str) = cell.as_str() {
                                            if cell_str.contains("Ability Score")
                                                || cell_str == "✓"
                                            {
                                                levels.push((idx + 1) as i32);
                                            }
                                        }
                                    }
                                }
                                if !levels.is_empty() {
                                    return levels;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Special case for Fighter (gets more ASIs)
        if class.name.to_lowercase() == "fighter" {
            return vec![4, 6, 8, 12, 14, 16, 19];
        }

        // Special case for Rogue (gets extra ASI at level 10)
        if class.name.to_lowercase() == "rogue" {
            return vec![4, 8, 10, 12, 16, 19];
        }

        default_asi
    }

    /// Calculate average HP gain for this class
    pub fn average_hp_gain(&self) -> i32 {
        (self.hit_die_value / 2) + 1
    }
}

impl MulticlassPrerequisites {
    /// Get multiclass prerequisites for a class from database
    pub fn get(conn: &mut DbConnection, class_name: &str, source: &str) -> Result<Option<Self>> {
        let mut class_service = ClassService::new(conn);

        let class = class_service
            .get_class_by_name_and_source(class_name, source)
            .map_err(|e| DbError::InvalidData(format!("Failed to get class: {}", e)))?;

        if let Some(class) = class {
            // Parse multiclassing requirements from typed Multiclassing struct
            if let Some(multiclassing) = &class.multiclassing {
                if let Some(requirements) = &multiclassing.requirements {
                    let mut required_abilities = Vec::new();

                    // Extract ability requirements from typed struct
                    if let Some(min) = requirements.str {
                        required_abilities.push(("Strength".to_string(), min));
                    }
                    if let Some(min) = requirements.dex {
                        required_abilities.push(("Dexterity".to_string(), min));
                    }
                    if let Some(min) = requirements.con {
                        required_abilities.push(("Constitution".to_string(), min));
                    }
                    if let Some(min) = requirements.int {
                        required_abilities.push(("Intelligence".to_string(), min));
                    }
                    if let Some(min) = requirements.wis {
                        required_abilities.push(("Wisdom".to_string(), min));
                    }
                    if let Some(min) = requirements.cha {
                        required_abilities.push(("Charisma".to_string(), min));
                    }

                    // Handle OR requirements - for simplicity, just include all alternatives
                    if let Some(or_reqs) = &requirements.or {
                        for or_req in or_reqs {
                            if let Some(min) = or_req.str {
                                required_abilities.push(("Strength".to_string(), min));
                            }
                            if let Some(min) = or_req.dex {
                                required_abilities.push(("Dexterity".to_string(), min));
                            }
                            if let Some(min) = or_req.cha {
                                required_abilities.push(("Charisma".to_string(), min));
                            }
                            // Add other abilities as needed
                        }
                    }

                    if !required_abilities.is_empty() {
                        return Ok(Some(Self {
                            class_name: class.name.clone(),
                            required_abilities,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Expand ability abbreviation to full name
    fn expand_ability_name(abbr: &str) -> String {
        match abbr.to_lowercase().as_str() {
            "str" => "Strength",
            "dex" => "Dexterity",
            "con" => "Constitution",
            "int" => "Intelligence",
            "wis" => "Wisdom",
            "cha" => "Charisma",
            _ => abbr, // Return as-is if not recognized
        }
        .to_string()
    }

    /// Check if character meets prerequisites for this class
    pub fn check(&self, abilities: &AbilityScores) -> Result<()> {
        for (ability_name, min_score) in &self.required_abilities {
            let score = match ability_name.to_lowercase().as_str() {
                "strength" => abilities.strength,
                "dexterity" => abilities.dexterity,
                "constitution" => abilities.constitution,
                "intelligence" => abilities.intelligence,
                "wisdom" => abilities.wisdom,
                "charisma" => abilities.charisma,
                _ => {
                    return Err(DbError::InvalidData(format!(
                        "Unknown ability: {}",
                        ability_name
                    )))
                }
            };

            if score < *min_score {
                return Err(DbError::InvalidData(format!(
                    "Multiclass prerequisite not met: {} requires {} {} (character has {})",
                    self.class_name, ability_name, min_score, score
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::establish_connection;

    fn setup_test_db() -> DbConnection {
        let mut conn = establish_connection(":memory:").expect("Failed to create in-memory DB");
        crate::run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    fn insert_test_class(
        conn: &mut DbConnection,
        class_name: &str,
        hit_die: i32,
        caster_prog: Option<&str>,
        multiclass_req: Option<&str>,
    ) {
        use diesel::prelude::*;

        let class_json = format!(
            r#"{{
            "name": "{}",
            "source": "PHB",
            "hd": {{"number": 1, "faces": {}}},
            "proficiency": ["str", "con"],
            "casterProgression": {},
            "multiclassing": {}
        }}"#,
            class_name,
            hit_die,
            caster_prog
                .map(|c| format!(r#""{}""#, c))
                .unwrap_or("null".to_string()),
            multiclass_req.unwrap_or("null")
        );

        diesel::insert_into(crate::schema::catalog_classes::table)
            .values((
                crate::schema::catalog_classes::name.eq(class_name),
                crate::schema::catalog_classes::source.eq("PHB"),
                crate::schema::catalog_classes::hit_dice.eq(format!("d{}", hit_die)),
                crate::schema::catalog_classes::caster_progression.eq(caster_prog),
                crate::schema::catalog_classes::full_class_json.eq(class_json),
            ))
            .execute(conn)
            .expect("Failed to insert test class");
    }

    #[test]
    fn test_hp_gain_validation() {
        let options = LevelUpOptions {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Roll(10),
            asi_or_feat: None,
            subclass_choice: None,
            snapshot_reason: None,
        };

        assert!(options.validate_hp_gain(10).is_ok());
        assert!(options.validate_hp_gain(12).is_ok());

        let bad_options = LevelUpOptions {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Roll(11),
            asi_or_feat: None,
            subclass_choice: None,
            snapshot_reason: None,
        };

        assert!(bad_options.validate_hp_gain(10).is_err());
    }

    #[test]
    fn test_asi_validation() {
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

        assert!(options.validate_asi_or_feat().is_ok());

        let bad_options = LevelUpOptions {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            hp_method: HpGainMethod::Average,
            asi_or_feat: Some(AsiOrFeat::AbilityScoreImprovement {
                ability1: "Strength".to_string(),
                increase1: 2,
                ability2: Some("Dexterity".to_string()),
                increase2: Some(1),
            }),
            subclass_choice: None,
            snapshot_reason: None,
        };

        assert!(bad_options.validate_asi_or_feat().is_err());
    }

    #[test]
    fn test_multiclass_prerequisites() {
        let mut conn = setup_test_db();
        insert_test_class(
            &mut conn,
            "Barbarian",
            12,
            None,
            Some(r#"{"requirements": {"str": 13}}"#),
        );
        insert_test_class(
            &mut conn,
            "Monk",
            8,
            None,
            Some(r#"{"requirements": {"dex": 13, "wis": 13}}"#),
        );

        let abilities = AbilityScores {
            strength: 15,
            dexterity: 12,
            constitution: 14,
            intelligence: 10,
            wisdom: 13,
            charisma: 8,
        };

        let barbarian_prereqs = MulticlassPrerequisites::get(&mut conn, "Barbarian", "PHB")
            .expect("Failed to get prereqs")
            .expect("Barbarian prereqs not found");
        assert!(barbarian_prereqs.check(&abilities).is_ok());

        let monk_prereqs = MulticlassPrerequisites::get(&mut conn, "Monk", "PHB")
            .expect("Failed to get prereqs")
            .expect("Monk prereqs not found");
        assert!(monk_prereqs.check(&abilities).is_err()); // Needs DEX 13
    }

    #[test]
    fn test_class_info() {
        let mut conn = setup_test_db();
        insert_test_class(&mut conn, "Fighter", 10, None, None);
        insert_test_class(&mut conn, "Wizard", 6, Some("full"), None);

        let fighter = ClassInfo::get(&mut conn, "Fighter", "PHB").expect("Failed to get Fighter");
        assert_eq!(fighter.hit_die_value, 10);
        assert_eq!(fighter.average_hp_gain(), 6);

        let wizard = ClassInfo::get(&mut conn, "Wizard", "PHB").expect("Failed to get Wizard");
        assert_eq!(wizard.hit_die_value, 6);
        assert_eq!(wizard.average_hp_gain(), 4);
        assert_eq!(wizard.spellcasting_type, Some(SpellcastingType::Full));
    }
}
