//! Proficiency extraction from class, background and race catalog data (shared by character creation and multiclass level-up).

use diesel::SqliteConnection;
use uuid::Uuid;

use crate::dal::campaign as dal;
use crate::models::campaign::{NewCharacterProficiency, ProficiencyType};
use crate::services::ServiceResult;

// =============================================================================
// Proficiency Extraction Helpers
// =============================================================================

/// A proficiency to be inserted during character creation.
pub(super) struct ProficiencyEntry {
    prof_type: ProficiencyType,
    name: String,
}

/// Extract deterministic (non-choice) proficiencies from a keyed JSON object.
///
/// Handles the common 5etools format: `{ "perception": true, "stealth": true }`
/// Ignores choice keys like "choose", "any", "anyStandard", etc.
fn extract_keyed_proficiencies(
    items: &[serde_json::Value],
    prof_type: ProficiencyType,
) -> Vec<ProficiencyEntry> {
    let mut result = Vec::new();
    for item in items {
        if let Some(obj) = item.as_object() {
            for (key, val) in obj {
                // Skip choice/meta keys
                if key == "choose" || key == "any" || key.starts_with("any") {
                    continue;
                }
                // Boolean true means granted proficiency
                if val.as_bool() == Some(true) {
                    // Clean up key: remove source suffix (e.g., "longsword|phb" -> "longsword")
                    let name = key.split('|').next().unwrap_or(key);
                    result.push(ProficiencyEntry {
                        prof_type: prof_type.clone(),
                        name: capitalize_proficiency(name),
                    });
                }
            }
        } else if let Some(s) = item.as_str() {
            // Simple string proficiency (e.g., armor types: "light", "medium", "heavy")
            result.push(ProficiencyEntry {
                prof_type: prof_type.clone(),
                name: capitalize_proficiency(s),
            });
        }
    }
    result
}

/// Capitalize a proficiency name for display.
fn capitalize_proficiency(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

/// Extract proficiencies from class catalog data.
pub(super) fn extract_class_proficiencies(
    class_data: &serde_json::Value,
    selected_skills: &[String],
) -> Vec<ProficiencyEntry> {
    let mut profs = Vec::new();

    // Saving throws from "proficiency" field: ["str", "con"]
    if let Some(saves) = class_data.get("proficiency").and_then(|v| v.as_array()) {
        for save in saves {
            if let Some(s) = save.as_str() {
                let full_name = match s {
                    "str" => "Strength",
                    "dex" => "Dexterity",
                    "con" => "Constitution",
                    "int" => "Intelligence",
                    "wis" => "Wisdom",
                    "cha" => "Charisma",
                    other => other,
                };
                profs.push(ProficiencyEntry {
                    prof_type: ProficiencyType::Save,
                    name: full_name.to_string(),
                });
            }
        }
    }

    // Starting proficiencies
    if let Some(sp) = class_data.get("startingProficiencies") {
        // Armor proficiencies
        if let Some(armor) = sp.get("armor").and_then(|v| v.as_array()) {
            profs.extend(extract_keyed_proficiencies(armor, ProficiencyType::Armor));
        }

        // Weapon proficiencies
        if let Some(weapons) = sp.get("weapons").and_then(|v| v.as_array()) {
            profs.extend(extract_keyed_proficiencies(
                weapons,
                ProficiencyType::Weapon,
            ));
        }

        // Tool proficiencies
        if let Some(tools) = sp.get("tools").and_then(|v| v.as_array()) {
            profs.extend(extract_keyed_proficiencies(tools, ProficiencyType::Tool));
        }
    }

    // Skill proficiencies (user-selected)
    for skill in selected_skills {
        profs.push(ProficiencyEntry {
            prof_type: ProficiencyType::Skill,
            name: skill.clone(),
        });
    }

    profs
}

/// Extract proficiencies from background catalog data.
pub(super) fn extract_background_proficiencies(
    bg_data: &serde_json::Value,
) -> Vec<ProficiencyEntry> {
    let mut profs = Vec::new();

    // Skill proficiencies (usually deterministic for backgrounds)
    if let Some(skills) = bg_data.get("skillProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(skills, ProficiencyType::Skill));
    }

    // Tool proficiencies
    if let Some(tools) = bg_data.get("toolProficiencies").and_then(|v| v.as_array()) {
        profs.extend(extract_keyed_proficiencies(tools, ProficiencyType::Tool));
    }

    // Language proficiencies
    if let Some(langs) = bg_data
        .get("languageProficiencies")
        .and_then(|v| v.as_array())
    {
        profs.extend(extract_keyed_proficiencies(
            langs,
            ProficiencyType::Language,
        ));
    }

    profs
}

/// Extract proficiencies from race catalog data.
pub(super) fn extract_race_proficiencies(race_data: &serde_json::Value) -> Vec<ProficiencyEntry> {
    let mut profs = Vec::new();

    // Skill proficiencies
    if let Some(skills) = race_data
        .get("skillProficiencies")
        .and_then(|v| v.as_array())
    {
        profs.extend(extract_keyed_proficiencies(skills, ProficiencyType::Skill));
    }

    // Weapon proficiencies
    if let Some(weapons) = race_data
        .get("weaponProficiencies")
        .and_then(|v| v.as_array())
    {
        profs.extend(extract_keyed_proficiencies(
            weapons,
            ProficiencyType::Weapon,
        ));
    }

    // Armor proficiencies
    if let Some(armor) = race_data
        .get("armorProficiencies")
        .and_then(|v| v.as_array())
    {
        profs.extend(extract_keyed_proficiencies(armor, ProficiencyType::Armor));
    }

    // Tool proficiencies
    if let Some(tools) = race_data
        .get("toolProficiencies")
        .and_then(|v| v.as_array())
    {
        profs.extend(extract_keyed_proficiencies(tools, ProficiencyType::Tool));
    }

    // Language proficiencies
    if let Some(langs) = race_data
        .get("languageProficiencies")
        .and_then(|v| v.as_array())
    {
        profs.extend(extract_keyed_proficiencies(
            langs,
            ProficiencyType::Language,
        ));
    }

    profs
}

/// Extract multiclass proficiencies from class catalog data.
pub(super) fn extract_multiclass_proficiencies(
    class_data: &serde_json::Value,
) -> Vec<ProficiencyEntry> {
    let mut profs = Vec::new();

    if let Some(mc) = class_data.get("multiclassing") {
        if let Some(gained) = mc.get("proficienciesGained") {
            if let Some(armor) = gained.get("armor").and_then(|v| v.as_array()) {
                profs.extend(extract_keyed_proficiencies(armor, ProficiencyType::Armor));
            }
            if let Some(weapons) = gained.get("weapons").and_then(|v| v.as_array()) {
                profs.extend(extract_keyed_proficiencies(
                    weapons,
                    ProficiencyType::Weapon,
                ));
            }
            if let Some(tools) = gained.get("tools").and_then(|v| v.as_array()) {
                profs.extend(extract_keyed_proficiencies(tools, ProficiencyType::Tool));
            }
        }
    }

    profs
}

/// Insert proficiency entries for a character, skipping duplicates.
pub(super) fn insert_proficiencies(
    conn: &mut SqliteConnection,
    character_id: &str,
    entries: &[ProficiencyEntry],
) -> ServiceResult<()> {
    for entry in entries {
        // Skip if character already has this proficiency
        if dal::character_has_proficiency(
            conn,
            character_id,
            &entry.prof_type.as_str(),
            &entry.name,
        )? {
            continue;
        }
        let prof_id = Uuid::new_v4().to_string();
        let new_prof = NewCharacterProficiency::new(
            &prof_id,
            character_id,
            entry.prof_type.clone(),
            &entry.name,
        );
        dal::insert_character_proficiency(conn, &new_prof)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Proficiency Extraction Tests ────────────────────────────────

    #[test]
    fn test_extract_keyed_proficiencies_boolean_keys() {
        let items = vec![serde_json::json!({"perception": true, "stealth": true})];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Skill);
        let names: Vec<&str> = profs.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Perception"));
        assert!(names.contains(&"Stealth"));
        assert_eq!(profs.len(), 2);
    }

    #[test]
    fn test_extract_keyed_proficiencies_ignores_choice_keys() {
        let items = vec![serde_json::json!({"choose": {"from": ["a", "b"]}, "any": 2})];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Skill);
        assert_eq!(profs.len(), 0);
    }

    #[test]
    fn test_extract_keyed_proficiencies_ignores_any_prefix_keys() {
        let items = vec![serde_json::json!({"anyStandard": 2, "anyMusicalInstrument": 1})];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Language);
        assert_eq!(profs.len(), 0);
    }

    #[test]
    fn test_extract_keyed_proficiencies_string_values() {
        let items = vec![
            serde_json::Value::String("light".to_string()),
            serde_json::Value::String("medium".to_string()),
            serde_json::Value::String("{@item shield|phb}".to_string()),
        ];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Armor);
        assert_eq!(profs.len(), 3);
        assert_eq!(profs[0].name, "Light");
        assert_eq!(profs[1].name, "Medium");
    }

    #[test]
    fn test_extract_keyed_proficiencies_strips_source_suffix() {
        let items = vec![serde_json::json!({"longsword|phb": true, "shortbow|phb": true})];
        let profs = extract_keyed_proficiencies(&items, ProficiencyType::Weapon);
        let names: Vec<&str> = profs.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Longsword"));
        assert!(names.contains(&"Shortbow"));
    }

    #[test]
    fn test_extract_class_proficiencies_saves() {
        let class_data = serde_json::json!({
            "proficiency": ["str", "con"],
            "startingProficiencies": {
                "armor": ["light", "medium", "heavy", "{@item shield|phb}"],
                "weapons": [{"simple weapons": true, "martial weapons": true}]
            }
        });
        let profs = extract_class_proficiencies(&class_data, &[]);
        let saves: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Save))
            .map(|p| p.name.as_str())
            .collect();
        assert!(saves.contains(&"Strength"));
        assert!(saves.contains(&"Constitution"));
    }

    #[test]
    fn test_extract_class_proficiencies_with_skills() {
        let class_data = serde_json::json!({
            "proficiency": ["dex", "int"],
            "startingProficiencies": {}
        });
        let selected = vec!["Stealth".to_string(), "Perception".to_string()];
        let profs = extract_class_proficiencies(&class_data, &selected);
        let skills: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Skill))
            .map(|p| p.name.as_str())
            .collect();
        assert!(skills.contains(&"Stealth"));
        assert!(skills.contains(&"Perception"));
    }

    #[test]
    fn test_extract_background_proficiencies() {
        let bg_data = serde_json::json!({
            "skillProficiencies": [{"insight": true, "religion": true}],
            "languageProficiencies": [{"common": true, "elvish": true}],
            "toolProficiencies": [{"disguise kit": true}]
        });
        let profs = extract_background_proficiencies(&bg_data);
        let skills: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Skill))
            .map(|p| p.name.as_str())
            .collect();
        let langs: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Language))
            .map(|p| p.name.as_str())
            .collect();
        let tools: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Tool))
            .map(|p| p.name.as_str())
            .collect();
        assert!(skills.contains(&"Insight"));
        assert!(skills.contains(&"Religion"));
        assert!(langs.contains(&"Common"));
        assert!(langs.contains(&"Elvish"));
        assert!(tools.contains(&"Disguise kit"));
    }

    #[test]
    fn test_extract_race_proficiencies() {
        let race_data = serde_json::json!({
            "skillProficiencies": [{"perception": true}],
            "languageProficiencies": [{"common": true, "elvish": true}],
            "weaponProficiencies": [{"longsword|phb": true, "shortbow|phb": true}]
        });
        let profs = extract_race_proficiencies(&race_data);
        let skills: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Skill))
            .map(|p| p.name.as_str())
            .collect();
        let weapons: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Weapon))
            .map(|p| p.name.as_str())
            .collect();
        assert!(skills.contains(&"Perception"));
        assert!(weapons.contains(&"Longsword"));
        assert!(weapons.contains(&"Shortbow"));
    }

    #[test]
    fn test_extract_multiclass_proficiencies() {
        let class_data = serde_json::json!({
            "multiclassing": {
                "proficienciesGained": {
                    "armor": ["light", "medium"],
                    "weapons": [{"simple weapons": true}]
                }
            }
        });
        let profs = extract_multiclass_proficiencies(&class_data);
        let armor: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Armor))
            .map(|p| p.name.as_str())
            .collect();
        let weapons: Vec<&str> = profs
            .iter()
            .filter(|p| matches!(p.prof_type, ProficiencyType::Weapon))
            .map(|p| p.name.as_str())
            .collect();
        assert!(armor.contains(&"Light"));
        assert!(armor.contains(&"Medium"));
        assert!(weapons.contains(&"Simple weapons"));
    }

    #[test]
    fn test_extract_multiclass_proficiencies_no_multiclass() {
        let class_data = serde_json::json!({});
        let profs = extract_multiclass_proficiencies(&class_data);
        assert_eq!(profs.len(), 0);
    }

    #[test]
    fn test_capitalize_proficiency() {
        assert_eq!(capitalize_proficiency("perception"), "Perception");
        assert_eq!(capitalize_proficiency("stealth"), "Stealth");
        assert_eq!(capitalize_proficiency(""), "");
        assert_eq!(capitalize_proficiency("a"), "A");
    }
}
