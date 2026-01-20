use super::types::{
    AbilityBonus, AdditionalSpells, DamageModifier, Entry, OtherSource, Prerequisite,
    ProficiencyItem,
};
use crate::schema::catalog_feats;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Feat from D&D 5e
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feat {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub prerequisite: Option<Vec<Prerequisite>>,
    #[serde(default)]
    pub ability: Option<Vec<AbilityBonus>>,
    #[serde(default)]
    pub skill_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub language_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub tool_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub weapon_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub armor_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub saving_throw_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub expertise: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub resist: Option<Vec<DamageModifier>>,
    #[serde(default)]
    pub immune: Option<Vec<DamageModifier>>,
    #[serde(default)]
    pub senses: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub additional_spells: Option<Vec<AdditionalSpells>>,
    #[serde(default)]
    pub other_sources: Option<Vec<OtherSource>>,
}

/// Container for feat data from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatData {
    #[serde(default)]
    pub feat: Option<Vec<Feat>>,
}

/// Summary of a feat for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatSummary {
    pub name: String,
    pub source: String,
    pub page: Option<u32>,
    pub prerequisites: Option<String>,
    pub brief: Option<String>,
}

impl From<&Feat> for FeatSummary {
    fn from(feat: &Feat) -> Self {
        use super::types::{Prerequisite, PrerequisiteLevel};

        // Extract prerequisites as a simple string
        let prerequisites = feat.prerequisite.as_ref().and_then(|prereqs| {
            if prereqs.is_empty() {
                None
            } else {
                let mut prereq_parts = Vec::new();
                for prereq in prereqs {
                    match prereq {
                        Prerequisite::Text(s) => prereq_parts.push(s.clone()),
                        Prerequisite::Object(obj) => {
                            // Extract ability requirements
                            if let Some(abilities) = &obj.ability {
                                for ab in abilities {
                                    for (stat, val) in ab {
                                        prereq_parts
                                            .push(format!("{} {}", stat.to_uppercase(), val));
                                    }
                                }
                            }
                            // Extract race requirements
                            if let Some(races) = &obj.race {
                                for r in races {
                                    prereq_parts.push(r.name.clone());
                                }
                            }
                            // Extract level requirements
                            if let Some(level) = &obj.level {
                                match level {
                                    PrerequisiteLevel::Number(n) => {
                                        prereq_parts.push(format!("Level {}", n))
                                    }
                                    PrerequisiteLevel::WithClass { level, class } => {
                                        if let Some(c) = class {
                                            prereq_parts.push(format!("{} {}", c.name, level));
                                        } else {
                                            prereq_parts.push(format!("Level {}", level));
                                        }
                                    }
                                }
                            }
                            // Extract spellcasting requirement
                            if obj.spellcasting == Some(true) {
                                prereq_parts.push("Spellcasting".to_string());
                            }
                        }
                    }
                }
                if !prereq_parts.is_empty() {
                    Some(prereq_parts.join(", "))
                } else {
                    None
                }
            }
        });

        // Extract a brief description from the first entry
        let brief = feat.entries.first().and_then(|entry| match entry {
            Entry::Text(text) => {
                let truncated = if text.len() > 100 {
                    let end = text
                        .char_indices()
                        .take_while(|(i, _)| *i < 100)
                        .map(|(i, _)| i)
                        .last()
                        .unwrap_or(100);
                    format!("{}...", &text[..end])
                } else {
                    text.clone()
                };
                Some(truncated)
            }
            Entry::Object(_) => None, // Complex entry - skip
        });

        FeatSummary {
            name: feat.name.clone(),
            source: feat.source.clone(),
            page: feat.page,
            prerequisites,
            brief,
        }
    }
}

/// Database model for catalog_feats table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = catalog_feats)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogFeat {
    pub id: i32,
    pub name: String,
    pub prerequisites: Option<String>,
    pub brief: Option<String>,
    pub source: String,
    pub full_feat_json: String,
    pub created_at: Option<String>,
}

/// Model for inserting new feats into the database
#[derive(Insertable, Debug)]
#[diesel(table_name = catalog_feats)]
pub struct NewCatalogFeat {
    pub name: String,
    pub prerequisites: Option<String>,
    pub brief: Option<String>,
    pub source: String,
    pub full_feat_json: String,
}

/// Filter parameters for feat search
#[derive(Debug, Clone, Default)]
pub struct FeatFilters {
    pub search_pattern: Option<String>,
    pub sources: Option<Vec<String>>,
    pub has_prerequisites: Option<bool>,
}

impl From<&CatalogFeat> for FeatSummary {
    fn from(feat: &CatalogFeat) -> Self {
        FeatSummary {
            name: feat.name.clone(),
            source: feat.source.clone(),
            page: None, // Page info is in the JSON, would need to parse if needed
            prerequisites: feat.prerequisites.clone(),
            brief: feat.brief.clone(),
        }
    }
}

impl From<&Feat> for NewCatalogFeat {
    fn from(feat: &Feat) -> Self {
        // Extract prerequisites as a simple string (reuse logic from FeatSummary)
        let feat_summary = FeatSummary::from(feat);

        NewCatalogFeat {
            name: feat.name.clone(),
            prerequisites: feat_summary.prerequisites,
            brief: feat_summary.brief,
            source: feat.source.clone(),
            full_feat_json: serde_json::to_string(feat).unwrap_or_default(),
        }
    }
}
