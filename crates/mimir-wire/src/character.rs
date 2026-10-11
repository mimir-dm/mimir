//! Characters (MIMIR-T-0717): the sheet, its changes, inventory, spells,
//! level-up, and the catalog choices the screens offer. The DM uses these
//! for any character; a player for their own (the server enforces).

use serde::{Deserialize, Serialize};

use crate::campaign::ClassLevel;
use crate::patch::double;

/// Ability scores, STR to CHA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Abilities {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

/// Coins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Currency {
    pub cp: i32,
    pub sp: i32,
    pub ep: i32,
    pub gp: i32,
    pub pp: i32,
}

/// One class of a character, with its subclass and sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetClass {
    pub class_name: String,
    pub class_source: String,
    pub level: i32,
    pub subclass_name: Option<String>,
    pub subclass_source: Option<String>,
    pub starting: bool,
}

impl From<&SheetClass> for ClassLevel {
    fn from(c: &SheetClass) -> Self {
        ClassLevel {
            class_name: c.class_name.clone(),
            subclass_name: c.subclass_name.clone(),
            level: c.level,
        }
    }
}

/// A proficiency: "skill", "save", "tool", "weapon", "armor" or "language".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proficiency {
    pub kind: String,
    pub name: String,
    pub expertise: bool,
}

/// An item a character carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub item_name: String,
    pub item_source: String,
    pub quantity: i32,
    pub equipped: bool,
    pub attuned: bool,
    pub notes: Option<String>,
}

/// A spell a character knows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnownSpell {
    pub id: String,
    pub spell_name: String,
    pub spell_source: String,
    pub source_class: String,
    pub prepared: bool,
}

/// A feat a character has.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterFeat {
    pub name: String,
    pub source: String,
    /// "asi", "race", "class" or "bonus".
    pub source_type: String,
}

/// A chosen class feature: "fighting_style", "metamagic", "maneuver",
/// "invocation" or "pact_boon".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChosenFeature {
    pub kind: String,
    pub name: String,
    pub source: String,
    pub source_class: String,
}

/// `GET /characters/{id}`: all of a character.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterSheet {
    pub id: String,
    pub campaign_id: Option<String>,
    pub name: String,
    pub is_npc: bool,
    pub player_name: Option<String>,
    pub race_name: Option<String>,
    pub race_source: Option<String>,
    pub background_name: Option<String>,
    pub background_source: Option<String>,
    pub abilities: Abilities,
    pub currency: Currency,
    pub traits: Option<String>,
    pub ideals: Option<String>,
    pub bonds: Option<String>,
    pub flaws: Option<String>,
    pub role: Option<String>,
    pub location: Option<String>,
    pub faction: Option<String>,
    pub classes: Vec<SheetClass>,
    pub proficiencies: Vec<Proficiency>,
    pub inventory: Vec<InventoryItem>,
    pub spells: Vec<KnownSpell>,
    pub feats: Vec<CharacterFeat>,
    pub features: Vec<ChosenFeature>,
    /// Walking speed in feet, from the race (30 when unknown).
    pub speed: i32,
    pub updated_at: String,
}

/// `PATCH /characters/{id}`: absent keeps; `null` clears a text.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CharacterPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub player_name: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abilities: Option<Abilities>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<Currency>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub traits: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub ideals: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub bonds: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub flaws: Option<Option<String>>,
}

/// `POST /characters/{id}/inventory`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewInventoryItem {
    pub item_name: String,
    pub item_source: String,
    #[serde(default)]
    pub quantity: Option<i32>,
    #[serde(default)]
    pub equipped: bool,
    #[serde(default)]
    pub attuned: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

/// `PATCH /inventory/{id}`. At most 3 items attuned (the server checks).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InventoryPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipped: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attuned: Option<bool>,
}

/// `POST /characters/{id}/spells`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewKnownSpell {
    pub spell_name: String,
    pub spell_source: String,
    pub source_class: String,
    #[serde(default)]
    pub prepared: bool,
}

/// `PATCH /characters/{id}/spells/{spell_id}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpellPatch {
    pub prepared: bool,
}

// ---- Level-up (the server's LevelUpRequest, same JSON) ----------------------

/// A catalog reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ref {
    pub name: String,
    pub source: String,
}

/// How the new hit points are found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum HpMethod {
    Average,
    Roll(i32),
    Manual(i32),
}

/// The ASI or a feat of an ASI level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AsiOrFeat {
    AbilityScoreImprovement {
        ability1: String,
        increase1: i32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ability2: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        increase2: Option<i32>,
    },
    Feat {
        name: String,
        source: String,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SpellChanges {
    #[serde(default)]
    pub new_spells: Vec<Ref>,
    #[serde(default)]
    pub new_cantrips: Vec<Ref>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap_out: Option<Ref>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap_in: Option<Ref>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FeatureChoices {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fighting_style: Option<Ref>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metamagic: Option<Vec<Ref>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maneuvers: Option<ManeuverChoices>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocations: Option<InvocationChoices>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pact_boon: Option<Ref>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expertise_skills: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ManeuverChoices {
    #[serde(default)]
    pub new_maneuvers: Vec<Ref>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InvocationChoices {
    #[serde(default)]
    pub new_invocations: Vec<Ref>,
}

/// `POST /characters/{id}/level-up`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelUpRequest {
    pub class_name: String,
    pub class_source: String,
    pub hit_points_method: HpMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subclass: Option<Ref>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asi_or_feat: Option<AsiOrFeat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spell_changes: Option<SpellChanges>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_choices: Option<FeatureChoices>,
}

/// The answer to a level-up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelUpResult {
    pub sheet: CharacterSheet,
    pub hp_gained: i32,
    pub new_total_level: i32,
    pub is_multiclass: bool,
}

// ---- Catalog choices ------------------------------------------------------------

/// `GET /catalog/classes/{name}/{source}/level-info`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassLevelInfo {
    pub name: String,
    pub source: String,
    pub hit_die: i32,
    pub subclass_level: i32,
    pub asi_levels: Vec<i32>,
    /// As in the data: `{"str": 13}` or `{"or": [...]}`; null for none.
    pub multiclass_requirements: serde_json::Value,
    /// "full", "half", "third" or "pact".
    pub caster: Option<String>,
    pub spellcasting_ability: Option<String>,
    pub cantrips_known: Vec<i32>,
    pub spells_known: Vec<i32>,
    pub spells_added: Vec<i32>,
    pub optional_features: serde_json::Value,
}

/// A catalog entry offered as a choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub name: String,
    pub source: String,
    /// A short line: a spell's level and school, a feat's prerequisite, an
    /// item's type and rarity.
    #[serde(default)]
    pub detail: Option<String>,
    /// A spell's level.
    #[serde(default)]
    pub level: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_up_json_matches_the_server_form() {
        let r = LevelUpRequest {
            class_name: "Wizard".into(),
            class_source: "PHB".into(),
            hit_points_method: HpMethod::Roll(5),
            subclass: None,
            asi_or_feat: Some(AsiOrFeat::AbilityScoreImprovement {
                ability1: "intelligence".into(),
                increase1: 2,
                ability2: None,
                increase2: None,
            }),
            spell_changes: Some(SpellChanges::default()),
            feature_choices: None,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(
            v["hit_points_method"],
            serde_json::json!({"type": "Roll", "value": 5})
        );
        assert_eq!(v["asi_or_feat"]["type"], "AbilityScoreImprovement");
        assert_eq!(v["spell_changes"]["new_spells"], serde_json::json!([]));
        assert_eq!(
            serde_json::to_value(HpMethod::Average).unwrap(),
            serde_json::json!({"type": "Average"})
        );
        let back: LevelUpRequest = serde_json::from_value(v).unwrap();
        assert_eq!(back, r);
    }
}
