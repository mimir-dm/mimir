//! Shared types for 5etools catalog data
//!
//! This module provides typed alternatives to `serde_json::Value` for common
//! patterns in 5etools data structures.

use serde::{Deserialize, Serialize};

// =============================================================================
// Entry System
// =============================================================================

/// A content entry - can be plain text or a structured object.
///
/// 5etools uses entries extensively for rich content that can contain:
/// - Plain text strings
/// - Nested entry containers with names
/// - Lists with items
/// - Tables with rows
/// - Special formatting blocks (readaloud, inset, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entry {
    /// Plain text content
    Text(String),
    /// Structured entry object with a type field
    Object(EntryObject),
}

/// Structured entry object, discriminated by the "type" field.
///
/// Uses `#[serde(tag = "type")]` for internal tagging based on the type field.
/// Unknown types are captured by the `Unknown` variant to handle upstream changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EntryObject {
    /// Nested entries container (type: "entries")
    Entries {
        name: Option<String>,
        entries: Vec<Entry>,
    },
    /// A list of items (type: "list")
    List {
        items: Vec<Entry>,
        #[serde(default)]
        style: Option<String>,
    },
    /// A table (type: "table")
    Table {
        caption: Option<String>,
        #[serde(rename = "colLabels")]
        col_labels: Option<Vec<String>>,
        #[serde(rename = "colStyles")]
        col_styles: Option<Vec<String>>,
        rows: Vec<Vec<TableCell>>,
    },
    /// Read-aloud text block (type: "insetReadaloud")
    #[serde(rename = "insetReadaloud")]
    InsetReadaloud { entries: Vec<Entry> },
    /// Named item in a list (type: "item")
    Item {
        name: String,
        #[serde(default)]
        entry: Option<Box<Entry>>,
        #[serde(default)]
        entries: Option<Vec<Entry>>,
    },
    /// Section header (type: "section")
    Section {
        name: Option<String>,
        entries: Vec<Entry>,
    },
    /// Inset box (type: "inset")
    Inset {
        name: Option<String>,
        entries: Vec<Entry>,
    },
    /// Quote block (type: "quote")
    Quote {
        entries: Vec<Entry>,
        by: Option<String>,
        from: Option<String>,
    },
    /// Ability block for DC/attack info (type: "abilityDc", "abilityAttackMod")
    #[serde(rename = "abilityDc")]
    AbilityDc {
        name: String,
        attributes: Vec<String>,
    },
    #[serde(rename = "abilityAttackMod")]
    AbilityAttackMod {
        name: String,
        attributes: Vec<String>,
    },
    /// Options block (type: "options")
    Options {
        entries: Vec<Entry>,
        count: Option<i32>,
    },
    /// Inline entries (type: "inline")
    Inline { entries: Vec<Entry> },
    /// Inline block (type: "inlineBlock")
    InlineBlock { entries: Vec<Entry> },
    /// Bonus/feature block (type: "bonus", "optfeature")
    Bonus {
        name: String,
        entries: Vec<Entry>,
    },
    #[serde(rename = "optfeature")]
    OptFeature {
        name: String,
        entries: Vec<Entry>,
        #[serde(default)]
        prerequisite: Option<String>,
    },
    /// Catch-all for unknown entry types
    #[serde(other)]
    Unknown,
}

/// A cell in a table - can be text, number, or formatted object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TableCell {
    /// Plain text cell
    Text(String),
    /// Numeric cell
    Number(i64),
    /// Formatted cell with roll or other metadata
    Object(TableCellObject),
}

/// Formatted table cell with optional roll specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCellObject {
    #[serde(default)]
    pub roll: Option<RollSpec>,
    #[serde(default)]
    pub entry: Option<Box<Entry>>,
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}

/// Roll specification for dice expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollSpec {
    #[serde(default)]
    pub exact: Option<i32>,
    #[serde(default)]
    pub min: Option<i32>,
    #[serde(default)]
    pub max: Option<i32>,
    #[serde(default)]
    pub pad: Option<bool>,
}

// =============================================================================
// Image Types
// =============================================================================

/// An image reference in fluff/content data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "type")]
    pub image_type: Option<String>,
    pub href: Option<ImageHref>,
    pub title: Option<String>,
    pub credit: Option<String>,
    #[serde(rename = "maxWidth")]
    pub max_width: Option<i32>,
    #[serde(rename = "maxHeight")]
    pub max_height: Option<i32>,
}

/// Image href - path information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageHref {
    #[serde(rename = "type")]
    pub href_type: Option<String>,
    pub path: Option<String>,
    pub url: Option<String>,
}

// =============================================================================
// Proficiency/Choice Types
// =============================================================================

/// A proficiency item that can be a simple name or a choice.
///
/// 5etools uses this pattern for skill, language, tool, weapon, and armor proficiencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProficiencyItem {
    /// Simple proficiency by name
    Simple(String),
    /// Boolean flag (some proficiency lists use true/false)
    Flag(bool),
    /// A choice from a list
    Choice(ProficiencyChoice),
    /// A keyed object (e.g., { "perception": true })
    Keyed(std::collections::HashMap<String, serde_json::Value>),
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

// =============================================================================
// Monster/Creature Types
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
///
/// 5etools uses various formats:
/// - Plain number: `[12]`
/// - Object: `[{"ac": 15, "from": ["natural armor"]}]`
/// - Mixed: `[15, {"ac": 17, "from": ["shield"], "condition": "with shield"}]`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmorClassEntry {
    /// Just a number (e.g., `12` in `"ac": [12]`)
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

    /// Get the "from" sources if this is an object entry
    pub fn from(&self) -> Option<&Vec<String>> {
        match self {
            ArmorClassEntry::Number(_) => None,
            ArmorClassEntry::Object(obj) => obj.from.as_ref(),
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
    /// Just a number (for simple creatures)
    Number(i32),
}

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

/// Speed value - number or object with condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpeedValue {
    /// Simple speed in feet
    Number(i32),
    /// Speed with condition
    WithCondition {
        number: i32,
        condition: String,
    },
    /// Boolean (for "true" meaning can use this movement)
    Flag(bool),
}

/// Creature type - simple string or object with tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatureTypeValue {
    /// Simple type name
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

/// Creature type tag - string or object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreatureTag {
    /// Simple tag name
    Simple(String),
    /// Tag with prefix
    WithPrefix { tag: String, prefix: String },
}

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
// Ability Score Types
// =============================================================================

/// Ability score bonus/modifier - can be object with scores or choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AbilityBonus {
    /// Fixed ability bonuses
    Fixed(std::collections::HashMap<String, i32>),
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
// Resistance/Immunity Types
// =============================================================================

/// Damage resistance/immunity/vulnerability - string or choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamageModifier {
    /// Simple damage type
    Simple(String),
    /// Choice of damage types
    Choice(DamageModifierChoice),
    /// Conditional resistance
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

/// Choice of damage modifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageModifierChoice {
    #[serde(default)]
    pub choose: Option<ChooseSpec>,
    #[serde(default)]
    pub from: Option<Vec<String>>,
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

// =============================================================================
// Prerequisite Types
// =============================================================================

/// Prerequisite for a feat, optional feature, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Prerequisite {
    /// Simple text prerequisite
    Text(String),
    /// Structured prerequisite object
    Object(PrerequisiteObject),
}

/// Structured prerequisite with specific requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteObject {
    #[serde(default)]
    pub level: Option<PrerequisiteLevel>,
    #[serde(default)]
    pub race: Option<Vec<PrerequisiteRace>>,
    #[serde(default)]
    pub ability: Option<Vec<std::collections::HashMap<String, i32>>>,
    #[serde(default)]
    pub spellcasting: Option<bool>,
    #[serde(default)]
    pub pact: Option<String>,
    #[serde(default)]
    pub patron: Option<String>,
    #[serde(default)]
    pub spell: Option<Vec<String>>,
    #[serde(default)]
    pub feat: Option<Vec<String>>,
    #[serde(default)]
    pub feature: Option<Vec<String>>,
    #[serde(default)]
    pub item: Option<Vec<String>>,
    #[serde(default, rename = "otherSummary")]
    pub other_summary: Option<PrerequisiteOther>,
    #[serde(default)]
    pub other: Option<String>,
    #[serde(default)]
    pub proficiency: Option<Vec<std::collections::HashMap<String, String>>>,
    #[serde(default)]
    pub psionics: Option<bool>,
}

/// Level prerequisite.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrerequisiteLevel {
    /// Simple level number
    Number(i32),
    /// Level with class
    WithClass {
        level: i32,
        #[serde(default)]
        class: Option<PrerequisiteClass>,
    },
}

/// Class reference in prerequisite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteClass {
    pub name: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub visible: Option<bool>,
}

/// Race reference in prerequisite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteRace {
    pub name: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub subrace: Option<String>,
    #[serde(default, rename = "displayEntry")]
    pub display_entry: Option<String>,
}

/// Other prerequisite summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteOther {
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub entries: Option<Vec<String>>,
}

// =============================================================================
// Additional Spells Types
// =============================================================================

/// Additional spells granted by a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalSpells {
    #[serde(default)]
    pub innate: Option<std::collections::HashMap<String, AdditionalSpellList>>,
    #[serde(default)]
    pub known: Option<std::collections::HashMap<String, AdditionalSpellList>>,
    #[serde(default)]
    pub prepared: Option<std::collections::HashMap<String, AdditionalSpellList>>,
    #[serde(default)]
    pub expanded: Option<std::collections::HashMap<String, AdditionalSpellList>>,
    #[serde(default)]
    pub ability: Option<SpellcastingAbility>,
    #[serde(default)]
    pub name: Option<String>,
}

/// List of spells at a level or always available.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AdditionalSpellList {
    /// List of spells
    List(Vec<AdditionalSpellEntry>),
    /// Daily use limits
    Daily(std::collections::HashMap<String, Vec<AdditionalSpellEntry>>),
}

/// A spell entry in additional spells.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AdditionalSpellEntry {
    /// Simple spell name
    Name(String),
    /// Spell with metadata
    Object {
        #[serde(flatten)]
        spell: std::collections::HashMap<String, serde_json::Value>,
    },
}

/// Spellcasting ability specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpellcastingAbility {
    /// Single ability
    Single(String),
    /// Choice of abilities
    Choose { choose: Vec<String> },
}

// =============================================================================
// Legendary Group Types
// =============================================================================

/// Reference to a legendary group definition.
///
/// 5etools uses this to link creatures to their legendary group data,
/// which contains lair actions and regional effects shared by related creatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendaryGroup {
    /// Name of the legendary group (e.g., "Aboleth", "Black Dragon")
    pub name: String,
    /// Source book containing the legendary group definition
    pub source: String,
}

// =============================================================================
// SRD Types
// =============================================================================

/// SRD (System Reference Document) status indicator.
///
/// Most entries use a boolean `true` to indicate SRD inclusion.
/// Some entries (especially items) use a string to specify an alternate
/// SRD name when it differs from the official name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SrdValue {
    /// Boolean flag - true means included in SRD
    Flag(bool),
    /// Alternative SRD name (e.g., "Apparatus of the Crab" for "Apparatus of Kwalish")
    Name(String),
}

// =============================================================================
// Race Types
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
    /// Get the numeric speed value
    pub fn as_number(&self) -> i32 {
        match self {
            SpeedValue::Number(n) => *n,
            SpeedValue::WithCondition { number, .. } => *number,
            SpeedValue::Flag(_) => 0, // true means "equal to walk", caller should handle
        }
    }

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

/// An entry in starting equipment for backgrounds.
///
/// Starting equipment can be:
/// - An item reference: `"common clothes|phb"`
/// - An equipment object with item and metadata
/// - A choice group with alternatives (a/b keys)
///
/// The `_` key represents the default/required items, while `a`, `b`, etc.
/// represent alternative choices.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StartingEquipmentEntry {
    /// A choice group with default items and alternatives
    ChoiceGroup(StartingEquipmentChoiceGroup),
}

/// A choice group for starting equipment.
///
/// Uses `_` for default items and letter keys (a, b, etc.) for alternatives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartingEquipmentChoiceGroup {
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

// =============================================================================
// Spell Component Types
// =============================================================================

/// Material component consumption indicator.
///
/// Indicates whether a spell's material components are consumed:
/// - `true`: Components are consumed when cast
/// - `false`: Components are not consumed
/// - `"optional"`: Components are optionally consumed (some uses consume, others don't)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConsumeValue {
    /// Boolean consumption flag
    Flag(bool),
    /// String value (typically "optional")
    Text(String),
}

impl ConsumeValue {
    /// Check if this always consumes the component
    pub fn is_consumed(&self) -> bool {
        matches!(self, ConsumeValue::Flag(true))
    }

    /// Check if consumption is optional
    pub fn is_optional(&self) -> bool {
        matches!(self, ConsumeValue::Text(s) if s == "optional")
    }
}
