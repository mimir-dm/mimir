//! Helper Functions for Catalog Commands
//!
//! Shared utilities for parsing and processing catalog data.

use serde_json::Value;

/// Find the level at which a class gains its subclass.
pub fn find_subclass_level(data: &Value) -> i32 {
    if let Some(features) = data.get("classFeatures").and_then(|f| f.as_array()) {
        for feature in features {
            // Check if this feature grants subclass
            let grants_subclass = match feature {
                Value::Object(obj) => obj.get("gainSubclassFeature")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                Value::String(_) => false, // Simple string refs don't grant subclass
                _ => false,
            };

            if grants_subclass {
                // Parse the level from the classFeature string or object
                if let Value::Object(obj) = feature {
                    if let Some(cf) = obj.get("classFeature").and_then(|v| v.as_str()) {
                        // Format: "FeatureName|ClassName|ClassSource|Level"
                        let parts: Vec<&str> = cf.split('|').collect();
                        if parts.len() >= 4 {
                            if let Ok(level) = parts[3].parse::<i32>() {
                                return level;
                            }
                        }
                    }
                }
            }
        }
    }

    // Default subclass levels by class name patterns
    3 // Most classes get subclass at 3
}

/// Extract multiclass prerequisites from class data.
pub fn extract_multiclass_prereqs(data: &Value) -> Value {
    if let Some(mc) = data.get("multiclassing") {
        if let Some(reqs) = mc.get("requirements") {
            return reqs.clone();
        }
    }
    Value::Null
}

/// Determine ASI levels for a class.
pub fn determine_asi_levels(class_name: &str, data: &Value) -> Vec<i32> {
    // Standard ASI levels
    let standard = vec![4, 8, 12, 16, 19];

    // Fighter and Rogue get extra ASIs
    let fighter_levels = vec![4, 6, 8, 12, 14, 16, 19];
    let rogue_levels = vec![4, 8, 10, 12, 16, 19];

    match class_name.to_lowercase().as_str() {
        "fighter" => fighter_levels,
        "rogue" => rogue_levels,
        _ => {
            // Try to find ASI levels from class features
            if let Some(features) = data.get("classFeatures").and_then(|f| f.as_array()) {
                let mut asi_levels = Vec::new();
                for feature in features {
                    let feature_str = match feature {
                        Value::String(s) => s.clone(),
                        Value::Object(obj) => obj.get("classFeature")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        _ => continue,
                    };

                    if feature_str.to_lowercase().contains("ability score improvement") {
                        // Parse level from format: "Feature|Class|Source|Level"
                        let parts: Vec<&str> = feature_str.split('|').collect();
                        if parts.len() >= 4 {
                            if let Ok(level) = parts[3].parse::<i32>() {
                                asi_levels.push(level);
                            }
                        }
                    }
                }
                if !asi_levels.is_empty() {
                    return asi_levels;
                }
            }
            standard
        }
    }
}

/// Extract spell slot progression from class table groups.
pub fn extract_spell_slots_from_table(table_groups: &[Value]) -> Vec<Value> {
    for group in table_groups {
        if let Some(labels) = group.get("colLabels").and_then(|l| l.as_array()) {
            // Look for spell slot columns (1st, 2nd, 3rd, etc.)
            let has_spell_slots = labels.iter().any(|l| {
                l.as_str().map(|s| s.contains("st") || s.contains("nd") || s.contains("rd") || s.contains("th")).unwrap_or(false)
            });

            if has_spell_slots {
                if let Some(rows) = group.get("rows").and_then(|r| r.as_array()) {
                    // Return the rows which contain spell slot data per level
                    return rows.clone();
                }
            }
        }
    }
    Vec::new()
}

/// Generate standard spell slot progression based on caster type.
pub fn generate_spell_slot_progression(caster_type: &str) -> Vec<Vec<i32>> {
    match caster_type {
        "full" => vec![
            // Level 1-20 spell slots [1st, 2nd, 3rd, 4th, 5th, 6th, 7th, 8th, 9th]
            vec![2, 0, 0, 0, 0, 0, 0, 0, 0], // 1
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0], // 2
            vec![4, 2, 0, 0, 0, 0, 0, 0, 0], // 3
            vec![4, 3, 0, 0, 0, 0, 0, 0, 0], // 4
            vec![4, 3, 2, 0, 0, 0, 0, 0, 0], // 5
            vec![4, 3, 3, 0, 0, 0, 0, 0, 0], // 6
            vec![4, 3, 3, 1, 0, 0, 0, 0, 0], // 7
            vec![4, 3, 3, 2, 0, 0, 0, 0, 0], // 8
            vec![4, 3, 3, 3, 1, 0, 0, 0, 0], // 9
            vec![4, 3, 3, 3, 2, 0, 0, 0, 0], // 10
            vec![4, 3, 3, 3, 2, 1, 0, 0, 0], // 11
            vec![4, 3, 3, 3, 2, 1, 0, 0, 0], // 12
            vec![4, 3, 3, 3, 2, 1, 1, 0, 0], // 13
            vec![4, 3, 3, 3, 2, 1, 1, 0, 0], // 14
            vec![4, 3, 3, 3, 2, 1, 1, 1, 0], // 15
            vec![4, 3, 3, 3, 2, 1, 1, 1, 0], // 16
            vec![4, 3, 3, 3, 2, 1, 1, 1, 1], // 17
            vec![4, 3, 3, 3, 3, 1, 1, 1, 1], // 18
            vec![4, 3, 3, 3, 3, 2, 1, 1, 1], // 19
            vec![4, 3, 3, 3, 3, 2, 2, 1, 1], // 20
        ],
        "1/2" | "half" => vec![
            // Half casters start at level 2
            vec![0, 0, 0, 0, 0], // 1
            vec![2, 0, 0, 0, 0], // 2
            vec![3, 0, 0, 0, 0], // 3
            vec![3, 0, 0, 0, 0], // 4
            vec![4, 2, 0, 0, 0], // 5
            vec![4, 2, 0, 0, 0], // 6
            vec![4, 3, 0, 0, 0], // 7
            vec![4, 3, 0, 0, 0], // 8
            vec![4, 3, 2, 0, 0], // 9
            vec![4, 3, 2, 0, 0], // 10
            vec![4, 3, 3, 0, 0], // 11
            vec![4, 3, 3, 0, 0], // 12
            vec![4, 3, 3, 1, 0], // 13
            vec![4, 3, 3, 1, 0], // 14
            vec![4, 3, 3, 2, 0], // 15
            vec![4, 3, 3, 2, 0], // 16
            vec![4, 3, 3, 3, 1], // 17
            vec![4, 3, 3, 3, 1], // 18
            vec![4, 3, 3, 3, 2], // 19
            vec![4, 3, 3, 3, 2], // 20
        ],
        "1/3" | "third" => vec![
            // Third casters (Eldritch Knight, Arcane Trickster) start at level 3
            vec![0, 0, 0, 0], // 1
            vec![0, 0, 0, 0], // 2
            vec![2, 0, 0, 0], // 3
            vec![3, 0, 0, 0], // 4
            vec![3, 0, 0, 0], // 5
            vec![3, 0, 0, 0], // 6
            vec![4, 2, 0, 0], // 7
            vec![4, 2, 0, 0], // 8
            vec![4, 2, 0, 0], // 9
            vec![4, 3, 0, 0], // 10
            vec![4, 3, 0, 0], // 11
            vec![4, 3, 0, 0], // 12
            vec![4, 3, 2, 0], // 13
            vec![4, 3, 2, 0], // 14
            vec![4, 3, 2, 0], // 15
            vec![4, 3, 3, 0], // 16
            vec![4, 3, 3, 0], // 17
            vec![4, 3, 3, 0], // 18
            vec![4, 3, 3, 1], // 19
            vec![4, 3, 3, 1], // 20
        ],
        "pact" => vec![
            // Warlock pact magic - slots per short rest at highest available level
            // Format: [slots, slot_level]
            vec![1, 1], // 1
            vec![2, 1], // 2
            vec![2, 2], // 3
            vec![2, 2], // 4
            vec![2, 3], // 5
            vec![2, 3], // 6
            vec![2, 4], // 7
            vec![2, 4], // 8
            vec![2, 5], // 9
            vec![2, 5], // 10
            vec![3, 5], // 11
            vec![3, 5], // 12
            vec![3, 5], // 13
            vec![3, 5], // 14
            vec![3, 5], // 15
            vec![3, 5], // 16
            vec![4, 5], // 17
            vec![4, 5], // 18
            vec![4, 5], // 19
            vec![4, 5], // 20
        ],
        _ => Vec::new(),
    }
}

/// Extract which classes can use a fighting style based on its type code.
pub fn extract_fighting_style_classes(feature_type: Option<&str>) -> Vec<String> {
    match feature_type {
        Some(t) => {
            let mut classes = Vec::new();
            if t.contains("FS:F") || t == "FS" {
                classes.push("Fighter".to_string());
            }
            if t.contains("FS:P") || t == "FS" {
                classes.push("Paladin".to_string());
            }
            if t.contains("FS:R") || t == "FS" {
                classes.push("Ranger".to_string());
            }
            if t.contains("FS:B") {
                classes.push("Bard".to_string());
            }
            classes
        }
        None => Vec::new(),
    }
}

/// Extract invocation prerequisites (level, pact boon, spell requirements).
pub fn extract_invocation_prereqs(prereq: Option<&Value>) -> (Option<i32>, Option<String>, Option<String>) {
    let mut level_prereq = None;
    let mut pact_prereq = None;
    let mut spell_prereq = None;

    if let Some(Value::Array(prereqs)) = prereq {
        for p in prereqs {
            if let Value::Object(obj) = p {
                // Level requirement
                if let Some(lvl) = obj.get("level") {
                    if let Some(warlock_level) = lvl.get("warlock") {
                        level_prereq = warlock_level.as_i64().map(|l| l as i32);
                    } else if let Some(l) = lvl.as_i64() {
                        level_prereq = Some(l as i32);
                    }
                }

                // Pact boon requirement
                if let Some(pact) = obj.get("pact") {
                    pact_prereq = pact.as_str().map(|s| s.to_string());
                }

                // Spell requirement
                if let Some(spell) = obj.get("spell") {
                    if let Some(spells) = spell.as_array() {
                        spell_prereq = spells.first()
                            .and_then(|s| s.as_str())
                            .map(|s| s.replace("#c", "").to_string());
                    }
                }
            }
        }
    }

    (level_prereq, pact_prereq, spell_prereq)
}

/// Extract feat prerequisites into a simple list.
pub fn extract_feat_prereqs(prereq: Option<&Value>) -> Vec<String> {
    let mut prereqs = Vec::new();

    if let Some(Value::Array(arr)) = prereq {
        for p in arr {
            if let Value::Object(obj) = p {
                // Ability score requirements
                if let Some(ability) = obj.get("ability") {
                    if let Value::Array(abilities) = ability {
                        for a in abilities {
                            if let Value::Object(ab) = a {
                                for (stat, val) in ab {
                                    if let Some(v) = val.as_i64() {
                                        prereqs.push(format!("{} {}", stat.to_uppercase(), v));
                                    }
                                }
                            }
                        }
                    }
                }

                // Race requirements
                if let Some(race) = obj.get("race") {
                    if let Value::Array(races) = race {
                        for r in races {
                            if let Some(name) = r.get("name").and_then(|n| n.as_str()) {
                                prereqs.push(format!("Race: {}", name));
                            }
                        }
                    }
                }

                // Spellcasting requirement
                if obj.get("spellcasting").is_some() || obj.get("spellcastingFeature").is_some() {
                    prereqs.push("Spellcasting".to_string());
                }

                // Proficiency requirements
                if let Some(prof) = obj.get("proficiency") {
                    if let Value::Array(profs) = prof {
                        for pr in profs {
                            if let Value::Object(po) = pr {
                                for (key, _) in po {
                                    prereqs.push(format!("Proficiency: {}", key));
                                }
                            }
                        }
                    }
                }

                // Level requirement
                if let Some(level) = obj.get("level") {
                    if let Some(l) = level.as_i64() {
                        prereqs.push(format!("Level {}", l));
                    }
                }
            }
        }
    }

    prereqs
}
