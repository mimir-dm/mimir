//! Character creation module using database-backed race and background services
//!
//! Implements step-by-step character creation with data from uploaded rule books.

// Builder patterns have many fields - documentation would be verbose
#![allow(missing_docs)]

use crate::connection::DbConnection;
use crate::error::{DbError, Result};
use crate::models::catalog::{Background, Race};
use crate::models::character::data::{
    AbilityScores, Appearance, CharacterData, ClassLevel, Currency, EquippedItems, FeatureReference,
    InventoryItem, Personality, Proficiencies, RoleplayNotes, SpellData,
};
use crate::services::{BackgroundService, ClassService, RaceService};
use serde::{Deserialize, Serialize};

/// Method for determining ability scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityScoreMethod {
    /// Point buy system (27 points)
    PointBuy {
        strength: i32,
        dexterity: i32,
        constitution: i32,
        intelligence: i32,
        wisdom: i32,
        charisma: i32,
    },

    /// Standard array: 15, 14, 13, 12, 10, 8
    StandardArray {
        strength: i32,
        dexterity: i32,
        constitution: i32,
        intelligence: i32,
        wisdom: i32,
        charisma: i32,
    },

    /// Manual/rolled scores
    Manual {
        strength: i32,
        dexterity: i32,
        constitution: i32,
        intelligence: i32,
        wisdom: i32,
        charisma: i32,
    },
}

/// Character builder for step-by-step creation
pub struct CharacterBuilder<'a> {
    conn: &'a mut DbConnection,

    // Required fields
    character_name: Option<String>,
    player_id: Option<i32>,

    // Step 1: Race (from database)
    race_name: Option<String>,
    race_source: Option<String>,
    subrace_name: Option<String>,

    // Step 2: Class (from database)
    class: Option<String>,
    class_source: Option<String>,
    subclass: Option<String>,

    // Step 3: Ability Scores
    base_abilities: Option<AbilityScores>,
    ability_method: Option<AbilityScoreMethod>,

    // Step 4: Background (from database)
    background_name: Option<String>,
    background_source: Option<String>,

    // Optional fields
    alignment: Option<String>,
    personality: Personality,

    // Derived/calculated fields
    proficiencies: Proficiencies,
    starting_equipment: Vec<InventoryItem>,
}

impl<'a> CharacterBuilder<'a> {
    /// Create a new character builder
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self {
            conn,
            character_name: None,
            player_id: None,
            race_name: None,
            race_source: None,
            subrace_name: None,
            class: None,
            class_source: None,
            subclass: None,
            base_abilities: None,
            ability_method: None,
            background_name: None,
            background_source: None,
            alignment: None,
            personality: Personality::default(),
            proficiencies: Proficiencies::default(),
            starting_equipment: Vec::new(),
        }
    }

    /// Set character name and player (player_id is optional for NPCs)
    pub fn set_identity(mut self, character_name: String, player_id: Option<i32>) -> Self {
        self.character_name = Some(character_name);
        self.player_id = player_id;
        self
    }

    /// Set race by name and source (looks up from database)
    pub fn set_race(
        mut self,
        race_name: &str,
        source: &str,
        subrace: Option<String>,
    ) -> Result<Self> {
        // Verify race exists in database
        let race_json = RaceService::get_race_details(self.conn, race_name, source)?
            .ok_or_else(|| {
                DbError::InvalidData(format!("Race '{}' from '{}' not found in database. Please import the appropriate rulebook first.", race_name, source))
            })?;

        // Parse to verify it's valid
        let _race: Race = serde_json::from_str(&race_json)
            .map_err(|e| DbError::InvalidData(format!("Failed to parse race data: {}", e)))?;

        self.race_name = Some(race_name.to_string());
        self.race_source = Some(source.to_string());
        self.subrace_name = subrace;

        Ok(self)
    }

    /// Set race name directly without validation (for NPCs using monster/creature types)
    /// This allows using any string as a race, useful when the race is a monster
    /// type that isn't in the standard race catalog (e.g., "Goblin", "Yeti").
    /// Note: Racial ability bonuses will not be applied for unvalidated races.
    pub fn set_race_name_only(mut self, race_name: &str, source: &str) -> Self {
        self.race_name = Some(race_name.to_string());
        self.race_source = Some(source.to_string());
        self.subrace_name = None;
        self
    }

    /// Set class by name and source (looks up from database)
    pub fn set_class(
        mut self,
        class: &str,
        source: &str,
        subclass: Option<String>,
    ) -> Result<Self> {
        // Validate class exists in database
        let _class_info = super::level_up::ClassInfo::get(self.conn, class, source)?;

        self.class = Some(class.to_string());
        self.class_source = Some(source.to_string());
        self.subclass = subclass;

        // Add class proficiencies
        self.add_class_proficiencies(class);

        Ok(self)
    }

    /// Set ability scores
    pub fn set_ability_scores(mut self, method: AbilityScoreMethod) -> Result<Self> {
        // Validate the method
        method.validate()?;

        // Get base scores from method
        let base_scores = method.to_ability_scores();

        self.base_abilities = Some(base_scores);
        self.ability_method = Some(method);

        Ok(self)
    }

    /// Set background by name and source (looks up from database)
    pub fn set_background(mut self, background_name: &str, source: &str) -> Result<Self> {
        // Verify background exists in database
        let mut bg_service = BackgroundService::new(self.conn);
        let catalog_bg = bg_service.get_background_by_name_and_source(background_name, source)
            .map_err(|e| DbError::InvalidData(format!("Background '{}' from '{}' not found in database. Please import the appropriate rulebook first. Error: {}", background_name, source, e)))?
            .ok_or_else(|| DbError::InvalidData(format!("Background '{}' from '{}' not found in database. Please import the appropriate rulebook first.", background_name, source)))?;

        // Parse to get proficiencies
        let background: Background = serde_json::from_str(&catalog_bg.full_background_json)
            .map_err(|e| DbError::InvalidData(format!("Failed to parse background data: {}", e)))?;

        // Extract skill proficiencies from background
        for skill_value in &background.skill_proficiencies {
            match skill_value {
                crate::models::catalog::types::ProficiencyItem::Simple(skill) => {
                    let skill_name = titlecase(skill);
                    if !self.proficiencies.skills.contains(&skill_name) {
                        self.proficiencies.skills.push(skill_name);
                    }
                }
                crate::models::catalog::types::ProficiencyItem::Keyed(obj) => {
                    for (skill, _) in obj.iter() {
                        if skill != "any" && skill != "choose" {
                            let skill_name = titlecase(skill);
                            if !self.proficiencies.skills.contains(&skill_name) {
                                self.proficiencies.skills.push(skill_name);
                            }
                        }
                    }
                }
                _ => {} // Skip Flag and Choice variants for now
            }
        }

        // Extract tool proficiencies
        for tool_value in &background.tool_proficiencies {
            match tool_value {
                crate::models::catalog::types::ProficiencyItem::Simple(tool_str) => {
                    if !self.proficiencies.tools.contains(tool_str) {
                        self.proficiencies.tools.push(tool_str.clone());
                    }
                }
                crate::models::catalog::types::ProficiencyItem::Keyed(obj) => {
                    for (tool, _) in obj.iter() {
                        if tool != "choose" {
                            let tool_name = titlecase(tool);
                            if !self.proficiencies.tools.contains(&tool_name) {
                                self.proficiencies.tools.push(tool_name);
                            }
                        }
                    }
                }
                _ => {} // Skip Flag and Choice variants
            }
        }

        self.background_name = Some(background_name.to_string());
        self.background_source = Some(source.to_string());

        Ok(self)
    }

    /// Set alignment (optional)
    pub fn set_alignment(mut self, alignment: String) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Set personality traits
    pub fn set_personality(mut self, personality: Personality) -> Self {
        self.personality = personality;
        self
    }

    /// Add additional skill proficiency
    pub fn add_skill_proficiency(mut self, skill: String) -> Self {
        if !self.proficiencies.skills.contains(&skill) {
            self.proficiencies.skills.push(skill);
        }
        self
    }

    /// Add starting equipment
    pub fn add_equipment(mut self, item: InventoryItem) -> Self {
        self.starting_equipment.push(item);
        self
    }

    /// Build the final CharacterData
    pub fn build(mut self) -> Result<CharacterData> {
        // Validate required fields
        let character_name = self
            .character_name
            .ok_or_else(|| DbError::InvalidData("Character name is required".to_string()))?;

        // player_id is optional for NPCs
        let player_id = self.player_id;

        let race_name = self
            .race_name
            .ok_or_else(|| DbError::InvalidData("Race is required".to_string()))?;

        let race_source = self
            .race_source
            .ok_or_else(|| DbError::InvalidData("Race source is required".to_string()))?;

        let class = self
            .class
            .ok_or_else(|| DbError::InvalidData("Class is required".to_string()))?;

        let background_name = self
            .background_name
            .ok_or_else(|| DbError::InvalidData("Background is required".to_string()))?;

        let mut base_abilities = self
            .base_abilities
            .ok_or_else(|| DbError::InvalidData("Ability scores are required".to_string()))?;

        // Get race data from database and apply racial bonuses if available
        // For NPCs using monster races, race data may not exist in the race catalog
        let race_opt = RaceService::get_race_details(self.conn, &race_name, &race_source)?
            .and_then(|json| serde_json::from_str::<Race>(&json).ok());

        // Apply racial ability score bonuses if race data is available
        if let Some(ref race) = race_opt {
            if let Some(abilities) = &race.ability {
                for ability_value in abilities {
                    if let crate::models::catalog::types::AbilityBonus::Fixed(bonuses) = ability_value
                    {
                        if let Some(str_val) = bonuses.get("str") {
                            base_abilities.strength += *str_val;
                        }
                        if let Some(dex_val) = bonuses.get("dex") {
                            base_abilities.dexterity += *dex_val;
                        }
                        if let Some(con_val) = bonuses.get("con") {
                            base_abilities.constitution += *con_val;
                        }
                        if let Some(int_val) = bonuses.get("int") {
                            base_abilities.intelligence += *int_val;
                        }
                        if let Some(wis_val) = bonuses.get("wis") {
                            base_abilities.wisdom += *wis_val;
                        }
                        if let Some(cha_val) = bonuses.get("cha") {
                            base_abilities.charisma += *cha_val;
                        }
                    }
                    // Choice variants are handled during character creation UI
                }
            }
        }

        // Extract speed from race if available, otherwise use default
        let speed = if let Some(ref race) = race_opt {
            if let Some(speed_value) = &race.speed {
                if let Some(speed_num) = speed_value.as_i64() {
                    speed_num as i32
                } else if let Some(obj) = speed_value.as_object() {
                    obj.get("walk").and_then(|v| v.as_i64()).unwrap_or(30) as i32
                } else {
                    30
                }
            } else {
                30
            }
        } else {
            30
        };

        // Extract languages from race if available
        if let Some(ref race) = race_opt {
            if let Some(lang_profs) = &race.language_proficiencies {
                for lang_value in lang_profs {
                    match lang_value {
                        crate::models::catalog::types::ProficiencyItem::Simple(lang) => {
                            let lang_name = titlecase(lang);
                            if !self.proficiencies.languages.contains(&lang_name) {
                                self.proficiencies.languages.push(lang_name);
                            }
                        }
                        crate::models::catalog::types::ProficiencyItem::Keyed(obj) => {
                            for (lang, _) in obj.iter() {
                                if lang != "anyStandard" && lang != "choose" && lang != "any" {
                                    let lang_name = titlecase(lang);
                                    if !self.proficiencies.languages.contains(&lang_name) {
                                        self.proficiencies.languages.push(lang_name);
                                    }
                                }
                            }
                        }
                        _ => {} // Skip Flag and Choice variants
                    }
                }
            }
        }

        // Always add Common if not present
        if !self.proficiencies.languages.contains(&"Common".to_string()) {
            self.proficiencies.languages.push("Common".to_string());
        }

        let class_source = self
            .class_source
            .ok_or_else(|| DbError::InvalidData("Class source is required".to_string()))?;

        // Get class info for hit die from database
        let class_info = super::level_up::ClassInfo::get(self.conn, &class, &class_source)?;

        // Calculate starting HP: max hit die + CON modifier
        let max_hp = class_info.hit_die_value + base_abilities.con_modifier();

        // Fetch class features for level 1
        let mut class_features = Vec::new();
        {
            let mut class_service = ClassService::new(self.conn);
            if let Ok(features) =
                class_service.get_class_features_up_to_level(&class, &class_source, 1)
            {
                for (feature_name, feature_level, feature_source) in features {
                    class_features.push(FeatureReference {
                        name: feature_name,
                        class_name: class.clone(),
                        subclass_name: None,
                        source: feature_source,
                        level: feature_level,
                    });
                }
            }

            // Also fetch subclass features if a subclass is selected
            if let Some(ref subclass_name) = self.subclass {
                if let Ok(sub_features) = class_service.get_subclass_features_up_to_level(
                    &class,
                    subclass_name,
                    &class_source,
                    1,
                ) {
                    for (feature_name, feature_level, feature_source) in sub_features {
                        class_features.push(FeatureReference {
                            name: feature_name,
                            class_name: class.clone(),
                            subclass_name: Some(subclass_name.clone()),
                            source: feature_source,
                            level: feature_level,
                        });
                    }
                }
            }
        }

        let mut character_data = CharacterData {
            character_name,
            player_id,
            level: 1,
            experience_points: 0,
            version: 1,
            snapshot_reason: Some("Initial character creation".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            race: race_name,
            subrace: self.subrace_name,
            classes: vec![ClassLevel {
                class_name: class,
                level: 1,
                subclass: self.subclass,
                hit_dice_type: class_info.hit_die,
                hit_dice_remaining: 1,
            }],
            background: background_name,
            alignment: self.alignment,
            abilities: base_abilities,
            max_hp,
            current_hp: max_hp,
            speed,
            proficiencies: self.proficiencies,
            class_features,
            feats: Vec::new(),
            spells: SpellData::default(),
            inventory: self.starting_equipment,
            currency: Currency::default(),
            equipped: EquippedItems::default(),
            personality: self.personality,
            // New character data fields
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
            // NPC fields default to None
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: Vec::new(),
            legendary_action_count: None,
        };

        // Initialize spell slots from class data
        if let Ok(spell_slots) =
            super::spell_management::calculate_spell_slots(self.conn, &character_data)
        {
            character_data.spells.spell_slots = spell_slots;
        }

        Ok(character_data)
    }

    // Helper method to add class proficiencies (same as before)
    fn add_class_proficiencies(&mut self, class: &str) {
        match class.to_lowercase().as_str() {
            "barbarian" => {
                self.proficiencies.armor.extend(vec![
                    "Light armor".to_string(),
                    "Medium armor".to_string(),
                    "Shields".to_string(),
                ]);
                self.proficiencies.weapons.extend(vec![
                    "Simple weapons".to_string(),
                    "Martial weapons".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Strength".to_string(), "Constitution".to_string()]);
            }
            "bard" => {
                self.proficiencies.armor.push("Light armor".to_string());
                self.proficiencies.weapons.extend(vec![
                    "Simple weapons".to_string(),
                    "Hand crossbows".to_string(),
                    "Longswords".to_string(),
                    "Rapiers".to_string(),
                    "Shortswords".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Dexterity".to_string(), "Charisma".to_string()]);
            }
            "cleric" => {
                self.proficiencies.armor.extend(vec![
                    "Light armor".to_string(),
                    "Medium armor".to_string(),
                    "Shields".to_string(),
                ]);
                self.proficiencies
                    .weapons
                    .push("Simple weapons".to_string());
                self.proficiencies
                    .saves
                    .extend(vec!["Wisdom".to_string(), "Charisma".to_string()]);
            }
            "druid" => {
                self.proficiencies.armor.extend(vec![
                    "Light armor".to_string(),
                    "Medium armor".to_string(),
                    "Shields".to_string(),
                ]);
                self.proficiencies.weapons.extend(vec![
                    "Clubs".to_string(),
                    "Daggers".to_string(),
                    "Darts".to_string(),
                    "Javelins".to_string(),
                    "Maces".to_string(),
                    "Quarterstaffs".to_string(),
                    "Scimitars".to_string(),
                    "Sickles".to_string(),
                    "Slings".to_string(),
                    "Spears".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Intelligence".to_string(), "Wisdom".to_string()]);
            }
            "fighter" => {
                self.proficiencies
                    .armor
                    .extend(vec!["All armor".to_string(), "Shields".to_string()]);
                self.proficiencies.weapons.extend(vec![
                    "Simple weapons".to_string(),
                    "Martial weapons".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Strength".to_string(), "Constitution".to_string()]);
            }
            "monk" => {
                self.proficiencies.weapons.extend(vec![
                    "Simple weapons".to_string(),
                    "Shortswords".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Strength".to_string(), "Dexterity".to_string()]);
            }
            "paladin" => {
                self.proficiencies
                    .armor
                    .extend(vec!["All armor".to_string(), "Shields".to_string()]);
                self.proficiencies.weapons.extend(vec![
                    "Simple weapons".to_string(),
                    "Martial weapons".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Wisdom".to_string(), "Charisma".to_string()]);
            }
            "ranger" => {
                self.proficiencies.armor.extend(vec![
                    "Light armor".to_string(),
                    "Medium armor".to_string(),
                    "Shields".to_string(),
                ]);
                self.proficiencies.weapons.extend(vec![
                    "Simple weapons".to_string(),
                    "Martial weapons".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Strength".to_string(), "Dexterity".to_string()]);
            }
            "rogue" => {
                self.proficiencies.armor.push("Light armor".to_string());
                self.proficiencies.weapons.extend(vec![
                    "Simple weapons".to_string(),
                    "Hand crossbows".to_string(),
                    "Longswords".to_string(),
                    "Rapiers".to_string(),
                    "Shortswords".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Dexterity".to_string(), "Intelligence".to_string()]);
            }
            "sorcerer" => {
                self.proficiencies.weapons.extend(vec![
                    "Daggers".to_string(),
                    "Darts".to_string(),
                    "Slings".to_string(),
                    "Quarterstaffs".to_string(),
                    "Light crossbows".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Constitution".to_string(), "Charisma".to_string()]);
            }
            "warlock" => {
                self.proficiencies.armor.push("Light armor".to_string());
                self.proficiencies
                    .weapons
                    .push("Simple weapons".to_string());
                self.proficiencies
                    .saves
                    .extend(vec!["Wisdom".to_string(), "Charisma".to_string()]);
            }
            "wizard" => {
                self.proficiencies.weapons.extend(vec![
                    "Daggers".to_string(),
                    "Darts".to_string(),
                    "Slings".to_string(),
                    "Quarterstaffs".to_string(),
                    "Light crossbows".to_string(),
                ]);
                self.proficiencies
                    .saves
                    .extend(vec!["Intelligence".to_string(), "Wisdom".to_string()]);
            }
            _ => {}
        }
    }
}

impl AbilityScoreMethod {
    /// Validate the ability score method
    pub fn validate(&self) -> Result<()> {
        match self {
            AbilityScoreMethod::PointBuy {
                strength,
                dexterity,
                constitution,
                intelligence,
                wisdom,
                charisma,
            } => {
                let scores = vec![
                    *strength,
                    *dexterity,
                    *constitution,
                    *intelligence,
                    *wisdom,
                    *charisma,
                ];

                // All scores must be between 8 and 15
                for score in &scores {
                    if *score < 8 || *score > 15 {
                        return Err(DbError::InvalidData(format!(
                            "Point buy scores must be between 8 and 15, got {}",
                            score
                        )));
                    }
                }

                // Calculate total points spent
                let total_cost: i32 = scores.iter().map(|s| point_buy_cost(*s)).sum();

                if total_cost != 27 {
                    return Err(DbError::InvalidData(format!(
                        "Point buy must total exactly 27 points, got {}",
                        total_cost
                    )));
                }
            }
            AbilityScoreMethod::StandardArray {
                strength,
                dexterity,
                constitution,
                intelligence,
                wisdom,
                charisma,
            } => {
                let mut scores = vec![
                    *strength,
                    *dexterity,
                    *constitution,
                    *intelligence,
                    *wisdom,
                    *charisma,
                ];
                scores.sort();

                let expected = vec![8, 10, 12, 13, 14, 15];

                if scores != expected {
                    return Err(DbError::InvalidData(
                        "Standard array must use exactly: 15, 14, 13, 12, 10, 8".to_string(),
                    ));
                }
            }
            AbilityScoreMethod::Manual {
                strength,
                dexterity,
                constitution,
                intelligence,
                wisdom,
                charisma,
            } => {
                let scores = vec![
                    *strength,
                    *dexterity,
                    *constitution,
                    *intelligence,
                    *wisdom,
                    *charisma,
                ];

                // All scores must be between 1 and 20
                for score in &scores {
                    if *score < 1 || *score > 20 {
                        return Err(DbError::InvalidData(format!(
                            "Manual scores must be between 1 and 20, got {}",
                            score
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert to AbilityScores struct
    pub fn to_ability_scores(&self) -> AbilityScores {
        match self {
            AbilityScoreMethod::PointBuy {
                strength,
                dexterity,
                constitution,
                intelligence,
                wisdom,
                charisma,
            }
            | AbilityScoreMethod::StandardArray {
                strength,
                dexterity,
                constitution,
                intelligence,
                wisdom,
                charisma,
            }
            | AbilityScoreMethod::Manual {
                strength,
                dexterity,
                constitution,
                intelligence,
                wisdom,
                charisma,
            } => AbilityScores {
                strength: *strength,
                dexterity: *dexterity,
                constitution: *constitution,
                intelligence: *intelligence,
                wisdom: *wisdom,
                charisma: *charisma,
            },
        }
    }
}

/// Calculate point buy cost for a given score
fn point_buy_cost(score: i32) -> i32 {
    match score {
        8 => 0,
        9 => 1,
        10 => 2,
        11 => 3,
        12 => 4,
        13 => 5,
        14 => 7,
        15 => 9,
        _ => 0,
    }
}

/// Convert string to title case
fn titlecase(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::establish_connection;
    use diesel::prelude::*;

    fn setup_test_db() -> DbConnection {
        let mut conn = establish_connection(":memory:").expect("Failed to create in-memory DB");
        crate::run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    fn insert_test_race(conn: &mut DbConnection) {
        // Insert a simple test race
        let race_json = r#"{
            "name": "Human",
            "source": "PHB",
            "ability": [{"str": 1, "dex": 1, "con": 1, "int": 1, "wis": 1, "cha": 1}],
            "speed": 30,
            "size": ["M"],
            "languageProficiencies": [{"common": true, "anyStandard": 1}]
        }"#;

        diesel::insert_into(crate::schema::catalog_races::table)
            .values((
                crate::schema::catalog_races::name.eq("Human"),
                crate::schema::catalog_races::source.eq("PHB"),
                crate::schema::catalog_races::full_race_json.eq(race_json),
            ))
            .execute(conn)
            .expect("Failed to insert test race");
    }

    fn insert_test_class(conn: &mut DbConnection, class_name: &str, hit_die: i32) {
        // Insert a test class
        let class_json = format!(
            r#"{{
            "name": "{}",
            "source": "PHB",
            "hd": {{"number": 1, "faces": {}}},
            "proficiency": ["str", "con"],
            "casterProgression": null
        }}"#,
            class_name, hit_die
        );

        diesel::insert_into(crate::schema::catalog_classes::table)
            .values((
                crate::schema::catalog_classes::name.eq(class_name),
                crate::schema::catalog_classes::source.eq("PHB"),
                crate::schema::catalog_classes::hit_dice.eq(format!("d{}", hit_die)),
                crate::schema::catalog_classes::primary_ability.eq("Strength"),
                crate::schema::catalog_classes::full_class_json.eq(class_json),
            ))
            .execute(conn)
            .expect("Failed to insert test class");
    }

    fn insert_test_background(conn: &mut DbConnection) {
        // Insert a simple test background
        let background_json = r#"{
            "name": "Soldier",
            "source": "PHB",
            "skillProficiencies": [{"athletics": true, "intimidation": true}],
            "languageProficiencies": [],
            "toolProficiencies": [{"gamingSet": 1}],
            "startingEquipment": []
        }"#;

        diesel::insert_into(crate::schema::catalog_backgrounds::table)
            .values((
                crate::schema::catalog_backgrounds::name.eq("Soldier"),
                crate::schema::catalog_backgrounds::source.eq("PHB"),
                crate::schema::catalog_backgrounds::skills.eq("Athletics, Intimidation"),
                crate::schema::catalog_backgrounds::languages.eq(""),
                crate::schema::catalog_backgrounds::tools.eq("Gaming Set"),
                crate::schema::catalog_backgrounds::feature.eq("Military Rank"),
                crate::schema::catalog_backgrounds::full_background_json.eq(background_json),
            ))
            .execute(conn)
            .expect("Failed to insert test background");
    }

    #[test]
    fn test_character_builder_with_point_buy() {
        let mut conn = setup_test_db();
        insert_test_race(&mut conn);
        insert_test_class(&mut conn, "Fighter", 10);
        insert_test_background(&mut conn);

        let ability_scores = AbilityScoreMethod::PointBuy {
            strength: 15,
            dexterity: 14,
            constitution: 13,
            intelligence: 12,
            wisdom: 10,
            charisma: 8,
        };

        let character_data = CharacterBuilder::new(&mut conn)
            .set_identity("Test Character".to_string(), Some(1))
            .set_race("Human", "PHB", None)
            .expect("Failed to set race")
            .set_class("Fighter", "PHB", None)
            .expect("Failed to set class")
            .set_ability_scores(ability_scores)
            .expect("Failed to set abilities")
            .set_background("Soldier", "PHB")
            .expect("Failed to set background")
            .set_alignment("Lawful Good".to_string())
            .build()
            .expect("Failed to build character");

        assert_eq!(character_data.character_name, "Test Character");
        assert_eq!(character_data.race, "Human");
        assert_eq!(character_data.primary_class_name(), "Fighter");
        assert_eq!(character_data.background, "Soldier");
        assert_eq!(character_data.level, 1);

        // Check racial bonuses were applied (+1 to all)
        assert_eq!(character_data.abilities.strength, 16);
        assert_eq!(character_data.abilities.dexterity, 15);
        assert_eq!(character_data.abilities.constitution, 14);
        assert_eq!(character_data.abilities.intelligence, 13);
        assert_eq!(character_data.abilities.wisdom, 11);
        assert_eq!(character_data.abilities.charisma, 9);
    }

    #[test]
    fn test_character_builder_with_standard_array() {
        let mut conn = setup_test_db();
        insert_test_race(&mut conn);
        insert_test_class(&mut conn, "Wizard", 6);
        insert_test_background(&mut conn);

        let ability_scores = AbilityScoreMethod::StandardArray {
            strength: 15,
            dexterity: 14,
            constitution: 13,
            intelligence: 12,
            wisdom: 10,
            charisma: 8,
        };

        let character_data = CharacterBuilder::new(&mut conn)
            .set_identity("Test Character 2".to_string(), Some(1))
            .set_race("Human", "PHB", None)
            .expect("Failed to set race")
            .set_class("Wizard", "PHB", None)
            .expect("Failed to set class")
            .set_background("Soldier", "PHB")
            .expect("Failed to set background")
            .set_ability_scores(ability_scores)
            .expect("Failed to set abilities")
            .build()
            .expect("Failed to build character");

        assert_eq!(character_data.character_name, "Test Character 2");
        assert_eq!(character_data.primary_class_name(), "Wizard");

        // Wizard should have d6 hit die
        assert_eq!(character_data.classes[0].hit_dice_type, "d6");
    }

    #[test]
    fn test_point_buy_validation_too_many_points() {
        let ability_scores = AbilityScoreMethod::PointBuy {
            strength: 15,
            dexterity: 15,
            constitution: 15,
            intelligence: 15,
            wisdom: 15,
            charisma: 15,
        };

        let result = ability_scores.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_point_buy_validation_too_few_points() {
        let ability_scores = AbilityScoreMethod::PointBuy {
            strength: 8,
            dexterity: 8,
            constitution: 8,
            intelligence: 8,
            wisdom: 8,
            charisma: 8,
        };

        let result = ability_scores.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_standard_array_validation_invalid() {
        let ability_scores = AbilityScoreMethod::StandardArray {
            strength: 16,
            dexterity: 14,
            constitution: 13,
            intelligence: 12,
            wisdom: 10,
            charisma: 8,
        };

        let result = ability_scores.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_character_builder_missing_race() {
        let mut conn = setup_test_db();
        insert_test_background(&mut conn);

        let _ability_scores = AbilityScoreMethod::StandardArray {
            strength: 15,
            dexterity: 14,
            constitution: 13,
            intelligence: 12,
            wisdom: 10,
            charisma: 8,
        };

        let result = CharacterBuilder::new(&mut conn)
            .set_identity("Test Character".to_string(), Some(1))
            .set_race("Elf", "PHB", None);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("not found in database"));
        }
    }

    #[test]
    fn test_character_builder_missing_background() {
        let mut conn = setup_test_db();
        insert_test_race(&mut conn);
        insert_test_class(&mut conn, "Fighter", 10);

        let _ability_scores = AbilityScoreMethod::StandardArray {
            strength: 15,
            dexterity: 14,
            constitution: 13,
            intelligence: 12,
            wisdom: 10,
            charisma: 8,
        };

        let result = CharacterBuilder::new(&mut conn)
            .set_identity("Test Character".to_string(), Some(1))
            .set_race("Human", "PHB", None)
            .expect("Failed to set race")
            .set_class("Fighter", "PHB", None)
            .expect("Failed to set class")
            .set_background("Noble", "PHB");

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("not found in database"));
        }
    }

    #[test]
    fn test_character_builder_incomplete() {
        let mut conn = setup_test_db();

        let result = CharacterBuilder::new(&mut conn).build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Character name is required"));
    }

    #[test]
    fn test_hp_calculation_different_classes() {
        let mut conn = setup_test_db();
        insert_test_race(&mut conn);
        insert_test_class(&mut conn, "Fighter", 10);
        insert_test_class(&mut conn, "Wizard", 6);
        insert_test_background(&mut conn);

        let ability_scores = AbilityScoreMethod::Manual {
            strength: 10,
            dexterity: 10,
            constitution: 16,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };

        // Fighter with d10 hit die and CON 16 (+3 modifier)
        let fighter = CharacterBuilder::new(&mut conn)
            .set_identity("Fighter Test".to_string(), Some(1))
            .set_race("Human", "PHB", None)
            .expect("Failed to set race")
            .set_class("Fighter", "PHB", None)
            .expect("Failed to set class")
            .set_background("Soldier", "PHB")
            .expect("Failed to set background")
            .set_ability_scores(ability_scores.clone())
            .expect("Failed to set abilities")
            .build()
            .expect("Failed to build character");

        assert_eq!(fighter.max_hp, 13); // 10 + 3

        // Wizard with d6 hit die and CON 16 (+3 modifier)
        let wizard = CharacterBuilder::new(&mut conn)
            .set_identity("Wizard Test".to_string(), Some(1))
            .set_race("Human", "PHB", None)
            .expect("Failed to set race")
            .set_class("Wizard", "PHB", None)
            .expect("Failed to set class")
            .set_background("Soldier", "PHB")
            .expect("Failed to set background")
            .set_ability_scores(ability_scores)
            .expect("Failed to set abilities")
            .build()
            .expect("Failed to build character");

        assert_eq!(wizard.max_hp, 9); // 6 + 3
    }
}
