use super::types::{Entry, Image, ProficiencyItem};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Background {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,
    #[serde(rename = "skillProficiencies", default)]
    pub skill_proficiencies: Vec<ProficiencyItem>,
    #[serde(rename = "languageProficiencies", default)]
    pub language_proficiencies: Vec<ProficiencyItem>,
    #[serde(rename = "toolProficiencies", default)]
    pub tool_proficiencies: Vec<ProficiencyItem>,
    #[serde(rename = "startingEquipment", default)]
    pub starting_equipment: Vec<serde_json::Value>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,
    #[serde(rename = "basicRules")]
    pub basic_rules: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackgroundData {
    #[serde(default)]
    pub background: Option<Vec<Background>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackgroundFluffData {
    #[serde(rename = "backgroundFluff", default)]
    pub background_fluff: Option<Vec<BackgroundFluff>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSummary {
    pub name: String,
    pub source: String,
    pub skills: String,
    pub languages: String,
    pub tools: String,
    pub feature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundWithDetails {
    pub background: Background,
    pub fluff: Option<BackgroundFluff>,
}

impl From<&Background> for BackgroundSummary {
    fn from(bg: &Background) -> Self {
        // Helper to extract proficiency names from ProficiencyItem
        fn extract_proficiency_names(items: &[ProficiencyItem], skip_keys: &[&str]) -> String {
            if items.is_empty() {
                return "None".to_string();
            }

            items
                .iter()
                .filter_map(|item| match item {
                    ProficiencyItem::Simple(name) => Some(titlecase(name)),
                    ProficiencyItem::Flag(_) => None,
                    ProficiencyItem::Choice(choice) => {
                        if let Some(any) = choice.any {
                            Some(format!("Any {}", any))
                        } else if let Some(choose) = &choice.choose {
                            let count = choose.count.unwrap_or(1);
                            if let Some(from) = &choose.from {
                                Some(format!(
                                    "Choose {} from {}",
                                    count,
                                    from.iter().map(|s| titlecase(s)).collect::<Vec<_>>().join(", ")
                                ))
                            } else {
                                Some(format!("Choose {}", count))
                            }
                        } else {
                            None
                        }
                    }
                    ProficiencyItem::Keyed(map) => {
                        let names: Vec<String> = map
                            .keys()
                            .filter(|k| !skip_keys.contains(&k.as_str()))
                            .map(|k| titlecase(k))
                            .collect();
                        if !names.is_empty() {
                            Some(names.join(", "))
                        } else {
                            // Check for 'any' or 'anyStandard' keys
                            if let Some(any_val) = map.get("any").or(map.get("anyStandard")) {
                                any_val.as_i64().map(|n| format!("Any {}", n))
                            } else {
                                None
                            }
                        }
                    }
                })
                .collect::<Vec<String>>()
                .join(", ")
        }

        let skills = extract_proficiency_names(&bg.skill_proficiencies, &["any", "choose"]);
        let languages = extract_proficiency_names(&bg.language_proficiencies, &["anyStandard", "choose", "any"]);
        let tools = extract_proficiency_names(&bg.tool_proficiencies, &["any", "choose"]);

        // Extract feature name from entries
        use super::types::EntryObject;
        let feature = bg
            .entries
            .iter()
            .filter_map(|e| match e {
                Entry::Object(EntryObject::Entries { name, .. }) => name.as_ref().and_then(|n| {
                    if n.starts_with("Feature:") {
                        Some(n.replace("Feature: ", ""))
                    } else {
                        None
                    }
                }),
                Entry::Object(EntryObject::Item { name, .. }) => {
                    if name.starts_with("Feature:") {
                        Some(name.replace("Feature: ", ""))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .next()
            .unwrap_or_else(|| "Special Feature".to_string());

        Self {
            name: bg.name.clone(),
            source: bg.source.clone(),
            skills,
            languages,
            tools,
            feature,
        }
    }
}

fn titlecase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

// Database models for catalog_backgrounds table
use crate::schema::catalog_backgrounds;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = catalog_backgrounds)]
pub struct CatalogBackground {
    pub id: i32,
    pub name: String,
    pub skills: String,
    pub languages: String,
    pub tools: String,
    pub feature: String,
    pub source: String,
    pub full_background_json: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = catalog_backgrounds)]
pub struct NewCatalogBackground {
    pub name: String,
    pub skills: String,
    pub languages: String,
    pub tools: String,
    pub feature: String,
    pub source: String,
    pub full_background_json: String,
}

impl From<&Background> for NewCatalogBackground {
    fn from(background: &Background) -> Self {
        let summary = BackgroundSummary::from(background);
        let full_json = serde_json::to_string(background).unwrap_or_default();

        Self {
            name: summary.name,
            skills: summary.skills,
            languages: summary.languages,
            tools: summary.tools,
            feature: summary.feature,
            source: summary.source,
            full_background_json: full_json,
        }
    }
}

impl From<&CatalogBackground> for BackgroundSummary {
    fn from(bg: &CatalogBackground) -> Self {
        Self {
            name: bg.name.clone(),
            source: bg.source.clone(),
            skills: bg.skills.clone(),
            languages: bg.languages.clone(),
            tools: bg.tools.clone(),
            feature: bg.feature.clone(),
        }
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct BackgroundFilters {
    pub search_pattern: Option<String>,
    pub sources: Option<Vec<String>>,
    pub has_tools: Option<bool>,
}
