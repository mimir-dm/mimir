//! Combat (MIMIR-T-0715): one session per module, its entries in turn
//! order, HP, conditions and concentration.

use serde::{Deserialize, Serialize};

use crate::patch::double;

/// A combat session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatSession {
    pub id: String,
    pub module_id: String,
    /// From 1.
    pub round: i32,
    /// Index in `entries` (turn order) of whose turn it is.
    pub turn_index: i32,
    /// "active" or "ended".
    pub status: String,
}

/// A condition on an entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub name: String,
    /// The round at whose start it ends; `None`: until removed.
    pub expires_round: Option<i32>,
}

/// One change of HP, for the log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HpChange {
    /// "damage", "heal" or "temp".
    pub kind: String,
    pub amount: i32,
    pub round: i32,
}

/// A combatant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatEntry {
    pub id: String,
    /// "module_monster", "module_npc", "character" or "custom".
    pub source_kind: String,
    pub source_id: Option<String>,
    /// The map token it stands for.
    pub token_id: Option<String>,
    pub name: String,
    pub initiative: Option<i32>,
    pub dex_modifier: Option<i32>,
    pub max_hp: Option<i32>,
    pub current_hp: Option<i32>,
    pub temp_hp: i32,
    pub concentrating: bool,
    pub conditions: Vec<Condition>,
    /// The last changes, newest first.
    pub hp_log: Vec<HpChange>,
}

/// A combat and its entries in turn order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Combat {
    pub session: CombatSession,
    pub entries: Vec<CombatEntry>,
    pub current_entry_id: Option<String>,
}

/// `POST /combat/{id}/entries`: what to add.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NewEntry {
    /// A monster group of the module; `count` copies (default: the group's
    /// quantity).
    Monster {
        module_monster_id: String,
        #[serde(default)]
        count: Option<i32>,
    },
    Npc {
        npc_id: String,
    },
    Character {
        character_id: String,
    },
    Custom {
        name: String,
        #[serde(default)]
        max_hp: Option<i32>,
        #[serde(default)]
        initiative: Option<i32>,
    },
}

/// `PATCH /combat-entries/{id}`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EntryPatch {
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub initiative: Option<Option<i32>>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_hp: Option<Option<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_hp: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concentrating: Option<bool>,
    #[serde(
        default,
        deserialize_with = "double",
        skip_serializing_if = "Option::is_none"
    )]
    pub token_id: Option<Option<String>>,
}

/// `POST /combat-entries/{id}/damage` and `/heal`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HpAmount {
    pub amount: i32,
}

/// The answer to damage: the entry, and the concentration save DC when the
/// entry concentrates (max(10, half the damage)).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageResult {
    pub entry: CombatEntry,
    pub concentration_dc: Option<i32>,
}

/// `POST /combat-entries/{id}/conditions`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewCondition {
    pub name: String,
    /// Rounds it lasts; `None`: until removed.
    #[serde(default)]
    pub duration_rounds: Option<i32>,
}

/// `POST /combat/{id}/link-tokens`: link entries to the tokens of a map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkTokens {
    pub map_id: String,
}

/// The turn order as players see it: names only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInitiative {
    pub round: i32,
    pub entries: Vec<PlayerTurn>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerTurn {
    pub name: String,
    /// It is this one's turn.
    pub current: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_entries_are_tagged_by_kind() {
        let e: NewEntry =
            serde_json::from_str(r#"{"kind":"monster","module_monster_id":"m1"}"#).unwrap();
        assert_eq!(
            e,
            NewEntry::Monster {
                module_monster_id: "m1".into(),
                count: None
            }
        );
        let c: NewEntry = serde_json::from_str(r#"{"kind":"custom","name":"Ghost"}"#).unwrap();
        assert_eq!(
            c,
            NewEntry::Custom {
                name: "Ghost".into(),
                max_hp: None,
                initiative: None
            }
        );
    }

    #[test]
    fn an_entry_patch_can_clear_the_initiative() {
        let p: EntryPatch = serde_json::from_str(r#"{"initiative":null}"#).unwrap();
        assert_eq!(p.initiative, Some(None));
        assert_eq!(p.max_hp, None);
    }
}
