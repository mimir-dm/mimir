//! Shared types for 5etools catalog data
//!
//! Common enums, polymorphic wrappers, and utility types used across
//! multiple entity types (monsters, spells, items, etc.).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Armor Class Types
// =============================================================================

/// Armor Class value - can be a number or array of AC entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmorClassValue {
    /// Simple numeric AC
    Number(i32),
    /// Array of AC entries with conditions
    Array(Vec<ArmorClassEntry>),
}

/// An AC entry - can be a plain number or an object with details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmorClassEntry {
    /// Just a number
    Number(i32),
    /// Full AC entry with optional details
    Object(ArmorClassEntryObject),
}

impl ArmorClassEntry {
    /// Get the AC value from this entry
    pub fn ac(&self) -> Option<i32> {
        match self {
            ArmorClassEntry::Number(n) => Some(*n),
            ArmorClassEntry::Object(obj) => obj.ac,
        }
    }
}

/// An AC entry object with optional source and condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorClassEntryObject {
    #[serde(default)]
    pub ac: Option<i32>,
    #[serde(default)]
    pub from: Option<Vec<String>>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub braces: Option<bool>,
}

// =============================================================================
// Hit Points Types
// =============================================================================

/// Hit points - standard formula or special text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HitPointsValue {
    /// Standard HP with average and optional formula
    Standard {
        average: i32,
        #[serde(default)]
        formula: Option<String>,
    },
    /// Special HP description
    Special { special: String },
    /// Just a number
    Number(i32),
}

impl HitPointsValue {
    /// Get the average HP value
    pub fn average(&self) -> Option<i32> {
        match self {
            HitPointsValue::Standard { average, .. } => Some(*average),
            HitPointsValue::Number(n) => Some(*n),
            HitPointsValue::Special { .. } => None,
        }
    }
}

// =============================================================================
// Challenge Rating Types
// =============================================================================

/// Challenge Rating - simple string or complex object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChallengeRatingValue {
    /// Simple CR string (e.g., "1/4", "5")
    Simple(String),
    /// Complex CR with lair/coven variants
    Complex {
        cr: String,
        #[serde(default)]
        lair: Option<String>,
        #[serde(default)]
        coven: Option<String>,
    },
}

impl ChallengeRatingValue {
    /// Get the base CR string
    pub fn cr(&self) -> &str {
        match self {
            ChallengeRatingValue::Simple(s) => s,
            ChallengeRatingValue::Complex { cr, .. } => cr,
        }
    }
}

// =============================================================================
// Speed Types
// =============================================================================

/// Speed value - number or object with condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpeedValue {
    /// Simple speed in feet
    Number(i32),
    /// Speed with condition
    WithCondition { number: i32, condition: String },
    /// Boolean (true = equals walk speed)
    Flag(bool),
}

impl SpeedValue {
    /// Get the numeric speed value
    pub fn as_number(&self) -> i32 {
        match self {
            SpeedValue::Number(n) => *n,
            SpeedValue::WithCondition { number, .. } => *number,
            SpeedValue::Flag(_) => 0,
        }
    }
}

/// Creature speeds object with multiple movement types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedObject {
    #[serde(default)]
    pub walk: Option<SpeedValue>,
    #[serde(default)]
    pub fly: Option<SpeedValue>,
    #[serde(default)]
    pub swim: Option<SpeedValue>,
    #[serde(default)]
    pub climb: Option<SpeedValue>,
    #[serde(default)]
    pub burrow: Option<SpeedValue>,
    #[serde(default)]
    pub can_hover: Option<bool>,
}

// =============================================================================
// Creature Type
// =============================================================================

/// Creature type - simple string or object with tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatureTypeValue {
    /// Simple type name (e.g., "humanoid")
    Simple(String),
    /// Type with tags
    Complex {
        #[serde(rename = "type")]
        base_type: String,
        #[serde(default)]
        tags: Option<Vec<CreatureTag>>,
        #[serde(default, rename = "swarmSize")]
        swarm_size: Option<String>,
    },
}

impl CreatureTypeValue {
    /// Get the base creature type
    pub fn base_type(&self) -> &str {
        match self {
            CreatureTypeValue::Simple(s) => s,
            CreatureTypeValue::Complex { base_type, .. } => base_type,
        }
    }
}

/// Creature type tag - string or object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatureTag {
    /// Simple tag name
    Simple(String),
    /// Tag with prefix
    WithPrefix { tag: String, prefix: String },
}

// =============================================================================
// Alignment Types
// =============================================================================

/// Alignment value - array of strings or objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AlignmentValue {
    /// Array of alignment components
    Array(Vec<AlignmentComponent>),
    /// Single alignment
    Single(AlignmentComponent),
}

/// Alignment component - abbreviation or special.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AlignmentComponent {
    /// Alignment abbreviation (L, N, C, G, E, etc.)
    Abbr(String),
    /// Special alignment description
    Special { special: String },
    /// Choice of alignments
    Choice { alignment: Vec<String> },
}

// =============================================================================
// Damage/Resistance Types
// =============================================================================

/// Damage resistance/immunity/vulnerability - string or complex.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamageModifier {
    /// Simple damage type
    Simple(String),
    /// Conditional resistance with notes
    Conditional {
        #[serde(default)]
        resist: Option<Vec<String>>,
        #[serde(default)]
        immune: Option<Vec<String>>,
        #[serde(default)]
        vulnerable: Option<Vec<String>>,
        #[serde(default)]
        note: Option<String>,
        #[serde(default)]
        cond: Option<bool>,
        #[serde(default, rename = "preNote")]
        pre_note: Option<String>,
    },
}

// =============================================================================
// Source Reference Types
// =============================================================================

/// Other source reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherSource {
    pub source: String,
    #[serde(default)]
    pub page: Option<i32>,
}

/// SRD status indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SrdValue {
    /// Boolean flag
    Flag(bool),
    /// Alternative SRD name
    Name(String),
}

// =============================================================================
// Legendary Group Types
// =============================================================================

/// Reference to a legendary group definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendaryGroup {
    pub name: String,
    pub source: String,
}

// =============================================================================
// Proficiency/Choice Types
// =============================================================================

/// Specification for a choice selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChooseSpec {
    #[serde(default)]
    pub from: Option<Vec<String>>,
    #[serde(default, rename = "fromFilter")]
    pub from_filter: Option<String>,
    #[serde(default)]
    pub count: Option<i32>,
    #[serde(default)]
    pub amount: Option<i32>,
}

/// A proficiency item that can be a simple name or a choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProficiencyItem {
    /// Simple proficiency by name
    Simple(String),
    /// Boolean flag
    Flag(bool),
    /// A choice from a list
    Choice(ProficiencyChoice),
    /// A keyed object
    Keyed(HashMap<String, serde_json::Value>),
}

/// A choice of proficiencies from a list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProficiencyChoice {
    #[serde(default)]
    pub choose: Option<ChooseSpec>,
    #[serde(default)]
    pub any: Option<i32>,
    #[serde(default)]
    pub from: Option<Vec<String>>,
    #[serde(default)]
    pub count: Option<i32>,
}

// =============================================================================
// Ability Score Types
// =============================================================================

/// Ability score bonus - can be object with scores or choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AbilityBonus {
    /// Fixed ability bonuses
    Fixed(HashMap<String, i32>),
    /// Choice of abilities
    Choice(AbilityChoice),
}

/// Choice of ability score improvements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityChoice {
    #[serde(default)]
    pub choose: Option<ChooseSpec>,
    #[serde(default)]
    pub from: Option<Vec<String>>,
    #[serde(default)]
    pub count: Option<i32>,
    #[serde(default)]
    pub amount: Option<i32>,
}

// =============================================================================
// Copy/Reference Types
// =============================================================================

/// Copy directive for inheriting from another entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyDirective {
    pub name: String,
    pub source: String,
    #[serde(default, rename = "_mod")]
    pub modifications: Option<serde_json::Value>,
    #[serde(default, rename = "_trait")]
    pub traits: Option<serde_json::Value>,
    #[serde(default, rename = "_preserve")]
    pub preserve: Option<serde_json::Value>,
}

// =============================================================================
// Race-specific Types
// =============================================================================

/// Race speed - can be a simple number or a structured object with multiple movement types.
///
/// Examples:
/// - Simple: `30` (walk 30 ft)
/// - Object: `{"walk": 30, "fly": 30}` or `{"walk": 30, "swim": true}` (swim equals walk)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RaceSpeed {
    /// Simple walking speed in feet
    Number(i32),
    /// Structured speed with multiple movement types
    Object(RaceSpeedObject),
}

impl RaceSpeed {
    /// Get the walking speed (or base speed if only a number)
    pub fn walk_speed(&self) -> i32 {
        match self {
            RaceSpeed::Number(n) => *n,
            RaceSpeed::Object(obj) => obj.walk.as_ref().map(|v| v.as_number()).unwrap_or(30),
        }
    }
}

/// Structured speed object with optional movement types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceSpeedObject {
    #[serde(default)]
    pub walk: Option<SpeedValue>,
    #[serde(default)]
    pub fly: Option<SpeedValue>,
    #[serde(default)]
    pub swim: Option<SpeedValue>,
    #[serde(default)]
    pub climb: Option<SpeedValue>,
    #[serde(default)]
    pub burrow: Option<SpeedValue>,
    /// Whether flight requires no armor
    #[serde(default)]
    pub fly_can_hover: Option<bool>,
}

impl SpeedValue {
    /// Check if this is a "true" flag meaning "equal to walk"
    pub fn is_equal_to_walk(&self) -> bool {
        matches!(self, SpeedValue::Flag(true))
    }
}

/// Lineage indicator - boolean or source string for legacy races.
///
/// In newer 5e content, `lineage: true` indicates a race follows the
/// Customizing Your Origin rules. A string value (e.g., "VRGR") indicates
/// the source where the lineage rules are defined.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Lineage {
    /// Boolean flag (true = uses lineage rules)
    Flag(bool),
    /// Source string where lineage rules are defined
    Source(String),
}

/// Height and weight generation parameters for a race.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeightAndWeight {
    /// Base height in inches
    #[serde(default)]
    pub base_height: Option<i32>,
    /// Height modifier dice expression (e.g., "2d8")
    #[serde(default)]
    pub height_mod: Option<String>,
    /// Base weight in pounds
    #[serde(default)]
    pub base_weight: Option<i32>,
    /// Weight modifier dice expression (e.g., "2d6")
    #[serde(default)]
    pub weight_mod: Option<String>,
}

// =============================================================================
// Starting Equipment Types
// =============================================================================

/// A choice group for starting equipment.
///
/// Uses `_` for default items and letter keys (a, b, etc.) for alternatives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartingEquipmentEntry {
    /// Default/required items (underscore key in JSON)
    #[serde(rename = "_", default)]
    pub default_items: Vec<StartingEquipmentItem>,
    /// Alternative choice A
    #[serde(default)]
    pub a: Option<Vec<StartingEquipmentItem>>,
    /// Alternative choice B
    #[serde(default)]
    pub b: Option<Vec<StartingEquipmentItem>>,
    /// Alternative choice C
    #[serde(default)]
    pub c: Option<Vec<StartingEquipmentItem>>,
}

/// A single item in starting equipment.
///
/// Can be a simple item reference string or a structured object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StartingEquipmentItem {
    /// Simple item reference: "common clothes|phb"
    ItemRef(String),
    /// Structured item with metadata
    Object(StartingEquipmentItemObject),
}

/// A structured starting equipment item with optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartingEquipmentItemObject {
    /// Item reference (e.g., "holy symbol|phb")
    #[serde(default)]
    pub item: Option<String>,
    /// Special item description for non-catalog items
    #[serde(default)]
    pub special: Option<String>,
    /// Display name override
    #[serde(default)]
    pub display_name: Option<String>,
    /// Quantity of this item
    #[serde(default)]
    pub quantity: Option<i32>,
    /// Value contained in this item (e.g., for pouches)
    #[serde(default)]
    pub contains_value: Option<i32>,
}
