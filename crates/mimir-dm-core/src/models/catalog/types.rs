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
