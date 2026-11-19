//! What level-up needs to know about a class (MIMIR-T-0717): hit die,
//! subclass level, ASI levels, multiclass requirements, spellcasting and
//! the progressions of cantrips, spells known and optional features. Read
//! from the class's catalog data (5etools form). Moved from the desktop
//! command layer, whose callers sent the wrong argument names.

use diesel::SqliteConnection;
use serde::Serialize;
use serde_json::Value;

use super::{CatalogEntityService, ClassService};
use crate::services::{ServiceError, ServiceResult};

/// Level-up facts of a class.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassLevelInfo {
    pub name: String,
    pub source: String,
    pub hit_die: i32,
    /// The class level that gives the subclass.
    pub subclass_level: i32,
    pub asi_levels: Vec<i32>,
    /// The multiclass requirements as in the data (`{"str": 13}`, or an
    /// `{"or": [...]}` object); null when none.
    pub multiclass_requirements: Value,
    /// "full", "half", "third" or "pact"; `None` for no spellcasting.
    pub caster: Option<String>,
    /// "int", "wis" or "cha".
    pub spellcasting_ability: Option<String>,
    /// Cantrips known at each class level (index 0 = level 1); empty when
    /// the class has none.
    pub cantrips_known: Vec<i32>,
    /// Spells known at each class level, for known casters; empty for
    /// prepared casters.
    pub spells_known: Vec<i32>,
    /// Spells added to the book at each level (the Wizard); empty if none.
    pub spells_added: Vec<i32>,
    /// Optional feature progressions (invocations, metamagic, maneuvers…)
    /// as in the data; null when none.
    pub optional_features: Value,
}

fn int_array(v: Option<&Value>) -> Vec<i32> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_i64())
                .map(|x| x as i32)
                .collect()
        })
        .unwrap_or_default()
}

/// The feature reference string of a class feature entry.
fn feature_ref(feature: &Value) -> Option<&str> {
    match feature {
        Value::String(s) => Some(s),
        Value::Object(o) => o.get("classFeature").and_then(Value::as_str),
        _ => None,
    }
}

/// The level of a "Name|Class|Source|Level" reference.
fn ref_level(r: &str) -> Option<i32> {
    r.split('|').nth(3).and_then(|l| l.parse().ok())
}

/// Read the level-up facts from a class's catalog data.
pub fn class_level_info(name: &str, source: &str, data: &Value) -> ClassLevelInfo {
    let features: &[Value] = data
        .get("classFeatures")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let subclass_level = features
        .iter()
        .find(|f| f.get("gainSubclassFeature").and_then(Value::as_bool) == Some(true))
        .and_then(feature_ref)
        .and_then(ref_level)
        .unwrap_or(3);
    let asi_levels = match name.to_lowercase().as_str() {
        "fighter" => vec![4, 6, 8, 12, 14, 16, 19],
        "rogue" => vec![4, 8, 10, 12, 16, 19],
        _ => {
            let found: Vec<i32> = features
                .iter()
                .filter_map(feature_ref)
                .filter(|r| r.to_lowercase().starts_with("ability score improvement"))
                .filter_map(ref_level)
                .collect();
            if found.is_empty() {
                vec![4, 8, 12, 16, 19]
            } else {
                found
            }
        }
    };
    let caster = data
        .get("casterProgression")
        .and_then(Value::as_str)
        .and_then(|p| match p {
            "full" => Some("full"),
            "1/2" | "half" | "artificer" => Some("half"),
            "1/3" | "third" => Some("third"),
            "pact" => Some("pact"),
            _ => None,
        })
        .map(String::from);
    ClassLevelInfo {
        name: name.to_string(),
        source: source.to_string(),
        hit_die: data
            .get("hd")
            .and_then(|h| h.get("faces"))
            .and_then(Value::as_i64)
            .map(|f| f as i32)
            .unwrap_or(8),
        subclass_level,
        asi_levels,
        multiclass_requirements: data
            .get("multiclassing")
            .and_then(|m| m.get("requirements"))
            .cloned()
            .unwrap_or(Value::Null),
        caster,
        spellcasting_ability: data
            .get("spellcastingAbility")
            .and_then(Value::as_str)
            .map(String::from),
        cantrips_known: int_array(data.get("cantripProgression")),
        spells_known: int_array(data.get("spellsKnownProgression")),
        spells_added: int_array(data.get("spellsKnownProgressionFixed")),
        optional_features: data
            .get("optionalfeatureProgression")
            .cloned()
            .unwrap_or(Value::Null),
    }
}

/// The level-up facts of a catalog class; `None` when it is not in the
/// catalog.
pub fn find_class_level_info(
    conn: &mut SqliteConnection,
    name: &str,
    source: &str,
) -> ServiceResult<Option<ClassLevelInfo>> {
    let Some(class) = ClassService::new(conn).get_by_name_and_source(name, source)? else {
        return Ok(None);
    };
    let data: Value = serde_json::from_str(&class.data)
        .map_err(|e| ServiceError::validation(format!("bad class data: {e}")))?;
    Ok(Some(class_level_info(&class.name, &class.source, &data)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_a_warlock() {
        let data = json!({
            "hd": {"number": 1, "faces": 8},
            "classFeatures": [
                "Otherworldly Patron|Warlock||1",
                {"classFeature": "Otherworldly Patron|Warlock||1", "gainSubclassFeature": true},
                "Ability Score Improvement|Warlock||4",
                "Ability Score Improvement|Warlock||8"
            ],
            "casterProgression": "pact",
            "spellcastingAbility": "cha",
            "cantripProgression": [2, 2, 2, 3],
            "spellsKnownProgression": [2, 3, 4, 5],
            "multiclassing": {"requirements": {"cha": 13}},
            "optionalfeatureProgression": [{"name": "Eldritch Invocations", "featureType": ["EI"], "progression": {"2": 2}}]
        });
        let info = class_level_info("Warlock", "PHB", &data);
        assert_eq!(info.hit_die, 8);
        assert_eq!(info.subclass_level, 1);
        assert_eq!(info.asi_levels, vec![4, 8]);
        assert_eq!(info.caster.as_deref(), Some("pact"));
        assert_eq!(info.spellcasting_ability.as_deref(), Some("cha"));
        assert_eq!(info.cantrips_known, vec![2, 2, 2, 3]);
        assert_eq!(info.spells_known, vec![2, 3, 4, 5]);
        assert_eq!(info.multiclass_requirements, json!({"cha": 13}));
        assert!(info.optional_features.is_array());
    }

    #[test]
    fn defaults_and_fighter_asis() {
        let info = class_level_info("Fighter", "PHB", &json!({"hd": {"faces": 10}}));
        assert_eq!(info.hit_die, 10);
        assert_eq!(info.subclass_level, 3);
        assert_eq!(info.asi_levels, vec![4, 6, 8, 12, 14, 16, 19]);
        assert_eq!(info.caster, None);
        assert!(info.cantrips_known.is_empty());
        let none = class_level_info("Oddity", "X", &json!({}));
        assert_eq!((none.hit_die, none.asi_levels.len()), (8, 5));
    }

    #[test]
    fn half_casters() {
        let info = class_level_info("Paladin", "PHB", &json!({"casterProgression": "1/2"}));
        assert_eq!(info.caster.as_deref(), Some("half"));
    }
}
