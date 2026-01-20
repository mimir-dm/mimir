//! Spell catalog extraction types
//!
//! Types for deserializing 5etools spell JSON data.

use serde::{Deserialize, Serialize};

/// A D&D 5e spell from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spell {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,
    pub level: u8,
    pub school: SpellSchool,
    pub time: Vec<CastingTime>,
    pub range: SpellRange,
    pub components: SpellComponents,
    pub duration: Vec<SpellDuration>,

    // Entries stored as JSON blob
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,
    #[serde(default)]
    pub entries_higher_level: Option<Vec<serde_json::Value>>,

    // Class/subclass access
    #[serde(default)]
    pub classes: Option<SpellClasses>,

    // Damage and effects
    #[serde(default)]
    pub damage_inflict: Option<Vec<String>>,
    #[serde(default)]
    pub saving_throw: Option<Vec<String>>,
    #[serde(default)]
    pub condition_inflict: Option<Vec<String>>,
    #[serde(default)]
    pub spell_attack: Option<Vec<String>>,

    // Scaling
    #[serde(default)]
    pub scaling_level_dice: Option<serde_json::Value>,

    // Metadata
    #[serde(default)]
    pub meta: Option<SpellMeta>,
    #[serde(default)]
    pub srd: Option<bool>,
    #[serde(default)]
    pub basic_rules: Option<bool>,
}

/// Spell school codes (single letter in 5etools).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpellSchool {
    #[serde(rename = "A")]
    Abjuration,
    #[serde(rename = "C")]
    Conjuration,
    #[serde(rename = "D")]
    Divination,
    #[serde(rename = "E")]
    Enchantment,
    #[serde(rename = "V")]
    Evocation,
    #[serde(rename = "I")]
    Illusion,
    #[serde(rename = "N")]
    Necromancy,
    #[serde(rename = "T")]
    Transmutation,
    #[serde(rename = "P")]
    Psionic,
}

impl SpellSchool {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Abjuration => "Abjuration",
            Self::Conjuration => "Conjuration",
            Self::Divination => "Divination",
            Self::Enchantment => "Enchantment",
            Self::Evocation => "Evocation",
            Self::Illusion => "Illusion",
            Self::Necromancy => "Necromancy",
            Self::Transmutation => "Transmutation",
            Self::Psionic => "Psionic",
        }
    }
}

/// Casting time specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastingTime {
    pub number: u32,
    pub unit: String,
    #[serde(default)]
    pub condition: Option<String>,
}

/// Spell range - point-based or special.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpellRange {
    Point {
        #[serde(rename = "type")]
        range_type: String,
        distance: SpellDistance,
    },
    Special {
        #[serde(rename = "type")]
        range_type: String,
    },
}

/// Distance specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellDistance {
    #[serde(rename = "type")]
    pub distance_type: String,
    #[serde(default)]
    pub amount: Option<u32>,
}

/// Spell components (V, S, M, R).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellComponents {
    #[serde(default)]
    pub v: Option<bool>,
    #[serde(default)]
    pub s: Option<bool>,
    #[serde(default)]
    pub m: Option<MaterialComponent>,
    #[serde(default)]
    pub r: Option<bool>, // Royalty component (from Acquisitions Incorporated)
}

/// Material component - can be text, object with cost, or bool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaterialComponent {
    Text(String),
    Object {
        text: String,
        #[serde(default)]
        cost: Option<u32>,
        #[serde(default)]
        consume: Option<ConsumeValue>,
    },
    Bool(bool),
}

/// Material component consumption indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConsumeValue {
    Flag(bool),
    Text(String),
}

impl ConsumeValue {
    pub fn is_consumed(&self) -> bool {
        matches!(self, ConsumeValue::Flag(true))
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, ConsumeValue::Text(s) if s == "optional")
    }
}

/// Spell duration specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellDuration {
    #[serde(rename = "type")]
    pub duration_type: String,
    #[serde(default)]
    pub duration: Option<DurationValue>,
    #[serde(default)]
    pub concentration: Option<bool>,
    #[serde(default)]
    pub ends: Option<Vec<String>>,
}

/// Duration value (amount and type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(default)]
    pub amount: Option<u32>,
    #[serde(default)]
    pub up_to: Option<bool>,
}

/// Classes that can cast the spell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellClasses {
    #[serde(default, rename = "fromClassList")]
    pub from_class_list: Option<Vec<ClassReference>>,
    #[serde(default, rename = "fromSubclass")]
    pub from_subclass: Option<Vec<SubclassSpellRef>>,
}

/// Class reference for spell lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassReference {
    pub name: String,
    pub source: String,
}

/// Subclass reference for spell lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassSpellRef {
    pub class: ClassReference,
    pub subclass: SubclassReference,
}

/// Subclass name and source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassReference {
    pub name: String,
    pub source: String,
    #[serde(default, rename = "subSubclass")]
    pub sub_subclass: Option<String>,
}

/// Spell metadata (ritual, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellMeta {
    #[serde(default)]
    pub ritual: bool,
}

/// Container for spell data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellData {
    #[serde(default)]
    pub spell: Vec<Spell>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_spell() {
        let json = json!({
            "name": "Fire Bolt",
            "source": "PHB",
            "page": 242,
            "level": 0,
            "school": "V",
            "time": [{"number": 1, "unit": "action"}],
            "range": {"type": "point", "distance": {"type": "feet", "amount": 120}},
            "components": {"v": true, "s": true},
            "duration": [{"type": "instant"}],
            "entries": ["You hurl a mote of fire at a creature or object within range."],
            "damageInflict": ["fire"],
            "spellAttack": ["R"]
        });

        let spell: Spell = serde_json::from_value(json).unwrap();
        assert_eq!(spell.name, "Fire Bolt");
        assert_eq!(spell.level, 0);
        assert_eq!(spell.school, SpellSchool::Evocation);
    }

    #[test]
    fn test_concentration_spell() {
        let json = json!({
            "name": "Bless",
            "source": "PHB",
            "level": 1,
            "school": "E",
            "time": [{"number": 1, "unit": "action"}],
            "range": {"type": "point", "distance": {"type": "feet", "amount": 30}},
            "components": {"v": true, "s": true, "m": "a sprinkling of holy water"},
            "duration": [{"type": "timed", "duration": {"type": "minute", "amount": 1}, "concentration": true}],
            "entries": ["You bless up to three creatures of your choice within range."]
        });

        let spell: Spell = serde_json::from_value(json).unwrap();
        assert_eq!(spell.name, "Bless");
        assert!(spell.duration[0].concentration.unwrap_or(false));
    }

    #[test]
    fn test_ritual_spell() {
        let json = json!({
            "name": "Detect Magic",
            "source": "PHB",
            "level": 1,
            "school": "D",
            "time": [{"number": 1, "unit": "action"}],
            "range": {"type": "point", "distance": {"type": "self"}},
            "components": {"v": true, "s": true},
            "duration": [{"type": "timed", "duration": {"type": "minute", "amount": 10}, "concentration": true}],
            "meta": {"ritual": true},
            "entries": ["For the duration, you sense the presence of magic within 30 feet of you."]
        });

        let spell: Spell = serde_json::from_value(json).unwrap();
        assert!(spell.meta.as_ref().map(|m| m.ritual).unwrap_or(false));
    }

    #[test]
    fn test_material_component_with_cost() {
        let json = json!({
            "name": "Revivify",
            "source": "PHB",
            "level": 3,
            "school": "N",
            "time": [{"number": 1, "unit": "action"}],
            "range": {"type": "point", "distance": {"type": "touch"}},
            "components": {
                "v": true,
                "s": true,
                "m": {
                    "text": "diamonds worth 300 gp, which the spell consumes",
                    "cost": 30000,
                    "consume": true
                }
            },
            "duration": [{"type": "instant"}],
            "entries": ["You touch a creature that has died within the last minute."]
        });

        let spell: Spell = serde_json::from_value(json).unwrap();
        if let Some(MaterialComponent::Object { cost, consume, .. }) = &spell.components.m {
            assert_eq!(*cost, Some(30000));
            assert!(consume.as_ref().map(|c| c.is_consumed()).unwrap_or(false));
        } else {
            panic!("Expected MaterialComponent::Object");
        }
    }

    #[test]
    fn test_spell_with_classes() {
        let json = json!({
            "name": "Fireball",
            "source": "PHB",
            "level": 3,
            "school": "V",
            "time": [{"number": 1, "unit": "action"}],
            "range": {"type": "point", "distance": {"type": "feet", "amount": 150}},
            "components": {"v": true, "s": true, "m": "a tiny ball of bat guano and sulfur"},
            "duration": [{"type": "instant"}],
            "entries": ["A bright streak flashes from your pointing finger..."],
            "classes": {
                "fromClassList": [
                    {"name": "Sorcerer", "source": "PHB"},
                    {"name": "Wizard", "source": "PHB"}
                ]
            }
        });

        let spell: Spell = serde_json::from_value(json).unwrap();
        let classes = spell.classes.as_ref().unwrap();
        let class_list = classes.from_class_list.as_ref().unwrap();
        assert_eq!(class_list.len(), 2);
        assert_eq!(class_list[0].name, "Sorcerer");
    }

    #[test]
    fn test_spell_data_container() {
        let json = json!({
            "spell": [
                {"name": "Light", "source": "PHB", "level": 0, "school": "V",
                 "time": [{"number": 1, "unit": "action"}],
                 "range": {"type": "point", "distance": {"type": "touch"}},
                 "components": {"v": true, "m": "a firefly"},
                 "duration": [{"type": "timed", "duration": {"type": "hour", "amount": 1}}],
                 "entries": []},
                {"name": "Mage Hand", "source": "PHB", "level": 0, "school": "C",
                 "time": [{"number": 1, "unit": "action"}],
                 "range": {"type": "point", "distance": {"type": "feet", "amount": 30}},
                 "components": {"v": true, "s": true},
                 "duration": [{"type": "timed", "duration": {"type": "minute", "amount": 1}}],
                 "entries": []}
            ]
        });

        let data: SpellData = serde_json::from_value(json).unwrap();
        assert_eq!(data.spell.len(), 2);
    }
}
