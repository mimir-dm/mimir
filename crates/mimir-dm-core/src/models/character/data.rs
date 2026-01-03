//! Character data structures for YAML serialization
//!
//! These structures represent the complete character state stored in character_data column.

use serde::{Deserialize, Serialize};

/// Default humanoid walking speed in feet
pub const DEFAULT_SPEED: i32 = 30;

fn default_speed() -> i32 {
    DEFAULT_SPEED
}

/// Ability scores with helper methods for modifiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbilityScores {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl AbilityScores {
    /// Calculate ability modifier from score (uses floor division for negative values)
    pub fn modifier(score: i32) -> i32 {
        let diff = score - 10;
        if diff >= 0 {
            diff / 2
        } else {
            // Floor division for negative numbers
            (diff - 1) / 2
        }
    }

    pub fn str_modifier(&self) -> i32 {
        Self::modifier(self.strength)
    }

    pub fn dex_modifier(&self) -> i32 {
        Self::modifier(self.dexterity)
    }

    pub fn con_modifier(&self) -> i32 {
        Self::modifier(self.constitution)
    }

    pub fn int_modifier(&self) -> i32 {
        Self::modifier(self.intelligence)
    }

    pub fn wis_modifier(&self) -> i32 {
        Self::modifier(self.wisdom)
    }

    pub fn cha_modifier(&self) -> i32 {
        Self::modifier(self.charisma)
    }
}

/// Proficiency tracking for skills, saves, armor, weapons, tools, languages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Proficiencies {
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub saves: Vec<String>,
    #[serde(default)]
    pub armor: Vec<String>,
    #[serde(default)]
    pub weapons: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub languages: Vec<String>,
}

/// Spell slot tracking for a specific spell level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpellSlots {
    pub max: i32,
    pub current: i32,
}

impl SpellSlots {
    pub fn new(max: i32) -> Self {
        Self { max, current: max }
    }

    pub fn expend(&mut self, count: i32) -> bool {
        if self.current >= count {
            self.current -= count;
            true
        } else {
            false
        }
    }

    pub fn recover(&mut self, count: i32) {
        self.current = (self.current + count).min(self.max);
    }

    pub fn recover_all(&mut self) {
        self.current = self.max;
    }
}

/// Reference to a spell with source information for unambiguous lookup
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct SpellReference {
    /// Spell name as it appears in the source
    pub name: String,
    /// Source book code (e.g., "PHB", "XGE")
    pub source: String,
}

// Custom deserializer to handle both old string format and new object format
impl<'de> serde::Deserialize<'de> for SpellReference {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct SpellRefVisitor;

        impl<'de> Visitor<'de> for SpellRefVisitor {
            type Value = SpellReference;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a spell reference object")
            }

            // Handle old format: just a string (spell name)
            fn visit_str<E>(self, value: &str) -> std::result::Result<SpellReference, E>
            where
                E: de::Error,
            {
                Ok(SpellReference {
                    name: value.to_string(),
                    source: "PHB".to_string(), // Default source for legacy data
                })
            }

            // Handle new format: object with name and source
            fn visit_map<M>(self, mut map: M) -> std::result::Result<SpellReference, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut source: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => name = Some(map.next_value()?),
                        "source" => source = Some(map.next_value()?),
                        _ => { let _: serde::de::IgnoredAny = map.next_value()?; }
                    }
                }

                Ok(SpellReference {
                    name: name.ok_or_else(|| de::Error::missing_field("name"))?,
                    source: source.unwrap_or_else(|| "PHB".to_string()),
                })
            }
        }

        deserializer.deserialize_any(SpellRefVisitor)
    }
}

impl std::fmt::Display for SpellReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.source)
    }
}

impl SpellReference {
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
        }
    }
}

/// Reference to a class/subclass feature with source information for unambiguous lookup
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct FeatureReference {
    /// Feature name as it appears in the source
    pub name: String,
    /// Class name this feature belongs to
    pub class_name: String,
    /// Subclass short name (if this is a subclass feature)
    #[serde(default)]
    pub subclass_name: Option<String>,
    /// Source book code (e.g., "PHB", "XGE")
    pub source: String,
    /// Level at which the feature was gained
    pub level: i32,
}

// Custom deserializer to handle both old string format and new object format
impl<'de> serde::Deserialize<'de> for FeatureReference {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct FeatureRefVisitor;

        impl<'de> Visitor<'de> for FeatureRefVisitor {
            type Value = FeatureReference;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a feature reference object")
            }

            // Handle old format: just a string (feature name)
            fn visit_str<E>(self, value: &str) -> std::result::Result<FeatureReference, E>
            where
                E: de::Error,
            {
                Ok(FeatureReference {
                    name: value.to_string(),
                    class_name: "Unknown".to_string(), // Default for legacy data
                    subclass_name: None,
                    source: "PHB".to_string(), // Default source for legacy data
                    level: 1,
                })
            }

            // Handle new format: object with all fields
            fn visit_map<M>(self, mut map: M) -> std::result::Result<FeatureReference, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut class_name: Option<String> = None;
                let mut subclass_name: Option<String> = None;
                let mut source: Option<String> = None;
                let mut level: Option<i32> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => name = Some(map.next_value()?),
                        "class_name" => class_name = Some(map.next_value()?),
                        "subclass_name" => subclass_name = map.next_value()?,
                        "source" => source = Some(map.next_value()?),
                        "level" => level = Some(map.next_value()?),
                        _ => { let _: serde::de::IgnoredAny = map.next_value()?; }
                    }
                }

                Ok(FeatureReference {
                    name: name.ok_or_else(|| de::Error::missing_field("name"))?,
                    class_name: class_name.unwrap_or_else(|| "Unknown".to_string()),
                    subclass_name,
                    source: source.unwrap_or_else(|| "PHB".to_string()),
                    level: level.unwrap_or(1),
                })
            }
        }

        deserializer.deserialize_any(FeatureRefVisitor)
    }
}

impl std::fmt::Display for FeatureReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref subclass) = self.subclass_name {
            write!(f, "{} ({} {} Lv{})", self.name, self.class_name, subclass, self.level)
        } else {
            write!(f, "{} ({} Lv{})", self.name, self.class_name, self.level)
        }
    }
}

impl FeatureReference {
    pub fn new(
        name: impl Into<String>,
        class_name: impl Into<String>,
        source: impl Into<String>,
        level: i32,
    ) -> Self {
        Self {
            name: name.into(),
            class_name: class_name.into(),
            subclass_name: None,
            source: source.into(),
            level,
        }
    }

    pub fn with_subclass(
        name: impl Into<String>,
        class_name: impl Into<String>,
        subclass_name: impl Into<String>,
        source: impl Into<String>,
        level: i32,
    ) -> Self {
        Self {
            name: name.into(),
            class_name: class_name.into(),
            subclass_name: Some(subclass_name.into()),
            source: source.into(),
            level,
        }
    }
}

/// Spell data for spellcasting characters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SpellData {
    #[serde(default)]
    pub known_spells: Vec<SpellReference>,
    #[serde(default)]
    pub prepared_spells: Vec<SpellReference>,
    #[serde(default)]
    pub cantrips: Vec<SpellReference>,
    #[serde(default)]
    pub spell_slots: std::collections::HashMap<i32, SpellSlots>,
}

/// Currency tracking for D&D denominations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Currency {
    #[serde(default)]
    pub copper: i32,
    #[serde(default)]
    pub silver: i32,
    #[serde(default)]
    pub electrum: i32,
    #[serde(default)]
    pub gold: i32,
    #[serde(default)]
    pub platinum: i32,
}

/// Inventory item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryItem {
    pub name: String,
    #[serde(default)]
    pub source: Option<String>,
    pub quantity: i32,
    #[serde(default)]
    pub weight: f64,
    #[serde(default)]
    pub value: f64,
    pub notes: Option<String>,
}

/// Equipped items in specific slots
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EquippedItems {
    pub armor: Option<String>,
    pub shield: Option<String>,
    pub main_hand: Option<String>,
    pub off_hand: Option<String>,
}

/// Character personality traits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Personality {
    pub traits: Option<String>,
    pub ideals: Option<String>,
    pub bonds: Option<String>,
    pub flaws: Option<String>,
}

/// Character appearance details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Appearance {
    #[serde(default)]
    pub age: Option<String>,
    #[serde(default)]
    pub height: Option<String>,
    #[serde(default)]
    pub weight: Option<String>,
    #[serde(default)]
    pub eyes: Option<String>,
    #[serde(default)]
    pub hair: Option<String>,
    #[serde(default)]
    pub skin: Option<String>,
    #[serde(default)]
    pub physical_description: Option<String>,
    #[serde(default)]
    pub distinctive_features: Option<String>,
}

/// Roleplay notes for character portrayal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RoleplayNotes {
    #[serde(default)]
    pub voice_and_mannerisms: Option<String>,
    #[serde(default)]
    pub key_relationships: Option<String>,
    #[serde(default)]
    pub character_goals: Option<String>,
    #[serde(default)]
    pub play_reminders: Option<String>,
    #[serde(default)]
    pub allies_and_organizations: Option<String>,
    #[serde(default)]
    pub additional_treasure_notes: Option<String>,
}

/// Legendary action for boss NPCs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegendaryAction {
    /// Name of the legendary action
    pub name: String,
    /// Cost in legendary action points (usually 1-3)
    #[serde(default = "default_legendary_cost")]
    pub cost: i32,
    /// Description of what the action does
    pub description: String,
}

fn default_legendary_cost() -> i32 {
    1
}

/// Individual class level for multiclassing support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClassLevel {
    pub class_name: String,
    pub level: i32,
    pub subclass: Option<String>,
    pub hit_dice_type: String,
    #[serde(default)]
    pub hit_dice_remaining: i32,
}

/// Complete character data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterData {
    // Metadata
    pub character_name: String,
    #[serde(default)]
    pub player_name: Option<String>,
    pub player_id: Option<i32>,
    pub level: i32,
    #[serde(default)]
    pub experience_points: i32,
    pub version: i32,
    pub snapshot_reason: Option<String>,
    pub created_at: String,

    // Core Identity
    pub race: String,
    pub subrace: Option<String>,
    pub classes: Vec<ClassLevel>,
    pub background: String,
    pub alignment: Option<String>,

    // Abilities
    pub abilities: AbilityScores,

    // HP and Resources
    pub max_hp: i32,
    pub current_hp: i32,

    // Movement
    #[serde(default = "default_speed")]
    pub speed: i32,

    // Proficiencies
    pub proficiencies: Proficiencies,

    // Class Features
    #[serde(default)]
    pub class_features: Vec<FeatureReference>,

    // Feats
    #[serde(default)]
    pub feats: Vec<String>,

    // Spells
    #[serde(default)]
    pub spells: SpellData,

    // Inventory
    #[serde(default)]
    pub inventory: Vec<InventoryItem>,

    // Currency
    #[serde(default)]
    pub currency: Currency,

    // Equipment
    #[serde(default)]
    pub equipped: EquippedItems,

    // Personality
    #[serde(default)]
    pub personality: Personality,

    // Appearance
    #[serde(default)]
    pub appearance: Appearance,

    // History/Backstory
    #[serde(default)]
    pub backstory: Option<String>,
    #[serde(default)]
    pub background_feature: Option<String>,

    // Roleplay Notes
    #[serde(default)]
    pub roleplay_notes: RoleplayNotes,

    // NPC-specific fields
    #[serde(default)]
    pub npc_role: Option<String>,
    #[serde(default)]
    pub npc_location: Option<String>,
    #[serde(default)]
    pub npc_faction: Option<String>,
    #[serde(default)]
    pub npc_notes: Option<String>,

    // Boss NPC abilities
    #[serde(default)]
    pub legendary_actions: Vec<LegendaryAction>,
    /// Number of legendary actions available per round (default 3)
    #[serde(default)]
    pub legendary_action_count: Option<i32>,
}

impl CharacterData {
    /// Calculate proficiency bonus based on level
    pub fn proficiency_bonus(&self) -> i32 {
        match self.level {
            1..=4 => 2,
            5..=8 => 3,
            9..=12 => 4,
            13..=16 => 5,
            17..=20 => 6,
            _ => 2, // fallback
        }
    }

    /// Check if character is proficient in a skill
    pub fn is_proficient_in_skill(&self, skill: &str) -> bool {
        self.proficiencies.skills.iter().any(|s| s == skill)
    }

    /// Check if character is proficient in a saving throw
    pub fn is_proficient_in_save(&self, save: &str) -> bool {
        self.proficiencies.saves.iter().any(|s| s == save)
    }

    /// Get primary class (first class taken)
    pub fn primary_class(&self) -> Option<&ClassLevel> {
        self.classes.first()
    }

    /// Get primary class name for display
    pub fn primary_class_name(&self) -> &str {
        self.classes
            .first()
            .map(|c| c.class_name.as_str())
            .unwrap_or("Unknown")
    }

    /// Get class display string (e.g., "Fighter 3 / Wizard 2")
    pub fn class_string(&self) -> String {
        self.classes
            .iter()
            .map(|c| format!("{} {}", c.class_name, c.level))
            .collect::<Vec<_>>()
            .join(" / ")
    }

    /// Get total hit dice remaining across all classes
    pub fn total_hit_dice_remaining(&self) -> i32 {
        self.classes.iter().map(|c| c.hit_dice_remaining).sum()
    }

    /// Find a class level by name
    pub fn get_class(&self, class_name: &str) -> Option<&ClassLevel> {
        self.classes.iter().find(|c| c.class_name == class_name)
    }

    /// Find a mutable class level by name
    pub fn get_class_mut(&mut self, class_name: &str) -> Option<&mut ClassLevel> {
        self.classes.iter_mut().find(|c| c.class_name == class_name)
    }

    /// Check if character has a specific class
    pub fn has_class(&self, class_name: &str) -> bool {
        self.classes.iter().any(|c| c.class_name == class_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_modifier_calculation() {
        assert_eq!(AbilityScores::modifier(10), 0);
        assert_eq!(AbilityScores::modifier(11), 0);
        assert_eq!(AbilityScores::modifier(12), 1);
        assert_eq!(AbilityScores::modifier(8), -1);
        assert_eq!(AbilityScores::modifier(20), 5);
        assert_eq!(AbilityScores::modifier(3), -4);
    }

    #[test]
    fn test_ability_scores_modifiers() {
        let abilities = AbilityScores {
            strength: 16,
            dexterity: 12,
            constitution: 14,
            intelligence: 10,
            wisdom: 13,
            charisma: 8,
        };

        assert_eq!(abilities.str_modifier(), 3);
        assert_eq!(abilities.dex_modifier(), 1);
        assert_eq!(abilities.con_modifier(), 2);
        assert_eq!(abilities.int_modifier(), 0);
        assert_eq!(abilities.wis_modifier(), 1);
        assert_eq!(abilities.cha_modifier(), -1);
    }

    #[test]
    fn test_proficiency_bonus_by_level() {
        let mut character = CharacterData {
            character_name: "Test".to_string(),
            player_name: None,
            player_id: Some(1),
            level: 1,
            experience_points: 0,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
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
            alignment: Some("Neutral".to_string()),
            abilities: AbilityScores {
                strength: 15,
                dexterity: 14,
                constitution: 13,
                intelligence: 12,
                wisdom: 10,
                charisma: 8,
            },
            max_hp: 12,
            current_hp: 12,
            speed: 30,
            proficiencies: Proficiencies::default(),
            class_features: Vec::new(),
            feats: Vec::new(),
            spells: SpellData::default(),
            inventory: Vec::new(),
            currency: Currency::default(),
            equipped: EquippedItems::default(),
            personality: Personality::default(),
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
        };

        character.level = 1;
        assert_eq!(character.proficiency_bonus(), 2);

        character.level = 5;
        assert_eq!(character.proficiency_bonus(), 3);

        character.level = 9;
        assert_eq!(character.proficiency_bonus(), 4);

        character.level = 13;
        assert_eq!(character.proficiency_bonus(), 5);

        character.level = 17;
        assert_eq!(character.proficiency_bonus(), 6);
    }

    #[test]
    fn test_spell_slot_management() {
        let mut slots = SpellSlots::new(4);
        assert_eq!(slots.current, 4);
        assert_eq!(slots.max, 4);

        assert!(slots.expend(2));
        assert_eq!(slots.current, 2);

        assert!(!slots.expend(3)); // not enough slots
        assert_eq!(slots.current, 2);

        slots.recover(1);
        assert_eq!(slots.current, 3);

        slots.recover(5); // should cap at max
        assert_eq!(slots.current, 4);

        slots.expend(4);
        slots.recover_all();
        assert_eq!(slots.current, 4);
    }

    #[test]
    fn test_yaml_serialization() {
        let character = CharacterData {
            character_name: "Thorin".to_string(),
            player_name: Some("Dave".to_string()),
            player_id: Some(1),
            level: 3,
            experience_points: 900,
            version: 1,
            snapshot_reason: Some("Initial creation".to_string()),
            created_at: "2025-01-15T10:30:00Z".to_string(),
            race: "Dwarf".to_string(),
            subrace: Some("Mountain".to_string()),
            classes: vec![ClassLevel {
                class_name: "Fighter".to_string(),
                level: 3,
                subclass: Some("Champion".to_string()),
                hit_dice_type: "d10".to_string(),
                hit_dice_remaining: 3,
            }],
            background: "Soldier".to_string(),
            alignment: Some("Lawful Good".to_string()),
            abilities: AbilityScores {
                strength: 16,
                dexterity: 12,
                constitution: 16,
                intelligence: 10,
                wisdom: 13,
                charisma: 8,
            },
            max_hp: 28,
            current_hp: 28,
            speed: 25, // Dwarf speed
            proficiencies: Proficiencies {
                skills: vec!["Athletics".to_string(), "Intimidation".to_string()],
                saves: vec!["Strength".to_string(), "Constitution".to_string()],
                armor: vec!["All armor".to_string(), "Shields".to_string()],
                weapons: vec!["Simple weapons".to_string(), "Martial weapons".to_string()],
                tools: vec!["Smith's tools".to_string()],
                languages: vec!["Common".to_string(), "Dwarvish".to_string()],
            },
            class_features: vec![
                FeatureReference {
                    name: "Fighting Style (Defense)".to_string(),
                    class_name: "Fighter".to_string(),
                    subclass_name: None,
                    source: "PHB".to_string(),
                    level: 1,
                },
                FeatureReference {
                    name: "Second Wind".to_string(),
                    class_name: "Fighter".to_string(),
                    subclass_name: None,
                    source: "PHB".to_string(),
                    level: 1,
                },
            ],
            feats: Vec::new(),
            spells: SpellData::default(),
            inventory: vec![InventoryItem {
                name: "Rations".to_string(),
                source: None,
                quantity: 10,
                weight: 20.0,
                value: 5.0,
                notes: None,
            }],
            currency: Currency::default(),
            equipped: EquippedItems {
                armor: Some("Chain Mail".to_string()),
                shield: Some("Shield".to_string()),
                main_hand: Some("Warhammer".to_string()),
                off_hand: None,
            },
            personality: Personality {
                traits: Some("I'm always polite and respectful.".to_string()),
                ideals: Some("Responsibility.".to_string()),
                bonds: Some(
                    "I would still lay down my life for the people I served with.".to_string(),
                ),
                flaws: Some("I obey authority without question.".to_string()),
            },
            appearance: Appearance {
                age: Some("195".to_string()),
                height: Some("4'5\"".to_string()),
                weight: Some("180 lbs".to_string()),
                eyes: Some("Brown".to_string()),
                hair: Some("Black, braided beard".to_string()),
                skin: Some("Weathered tan".to_string()),
                physical_description: None,
                distinctive_features: Some("Battle scar across left cheek".to_string()),
            },
            backstory: Some("A veteran of the clan wars.".to_string()),
            background_feature: Some("Military Rank".to_string()),
            roleplay_notes: RoleplayNotes::default(),
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: Vec::new(),
            legendary_action_count: None,
        };

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&character).expect("Failed to serialize");
        assert!(yaml.contains("character_name: Thorin"));
        assert!(yaml.contains("race: Dwarf"));
        assert!(yaml.contains("strength: 16"));
        assert!(yaml.contains("class_name: Fighter"));

        // Test round-trip
        let deserialized: CharacterData =
            serde_yaml::from_str(&yaml).expect("Failed to deserialize");
        assert_eq!(character, deserialized);
    }

    #[test]
    fn test_spell_reference_backward_compatibility() {
        // Test deserializing old format (string)
        let old_format_yaml = r#"
known_spells:
  - "Fireball"
  - "Magic Missile"
cantrips:
  - "Fire Bolt"
prepared_spells: []
spell_slots: {}
"#;
        let spell_data: SpellData = serde_yaml::from_str(old_format_yaml)
            .expect("Failed to deserialize old format");

        assert_eq!(spell_data.known_spells.len(), 2);
        assert_eq!(spell_data.known_spells[0].name, "Fireball");
        assert_eq!(spell_data.known_spells[0].source, "PHB"); // Default source
        assert_eq!(spell_data.known_spells[1].name, "Magic Missile");
        assert_eq!(spell_data.cantrips[0].name, "Fire Bolt");

        // Test deserializing new format (object)
        let new_format_yaml = r#"
known_spells:
  - name: "Fireball"
    source: "PHB"
  - name: "Holy Weapon"
    source: "XGE"
cantrips:
  - name: "Fire Bolt"
    source: "PHB"
prepared_spells: []
spell_slots: {}
"#;
        let spell_data: SpellData = serde_yaml::from_str(new_format_yaml)
            .expect("Failed to deserialize new format");

        assert_eq!(spell_data.known_spells.len(), 2);
        assert_eq!(spell_data.known_spells[0].name, "Fireball");
        assert_eq!(spell_data.known_spells[0].source, "PHB");
        assert_eq!(spell_data.known_spells[1].name, "Holy Weapon");
        assert_eq!(spell_data.known_spells[1].source, "XGE");
    }
}
