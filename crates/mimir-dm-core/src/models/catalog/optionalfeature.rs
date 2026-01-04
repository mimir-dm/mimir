use super::types::Entry;
use crate::schema::catalog_optional_features;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalFeature {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,

    #[serde(rename = "featureType")]
    pub feature_type: Vec<String>, // EI, MM, FS:F, etc.

    pub prerequisite: Option<Vec<Prerequisite>>,
    pub entries: Vec<Entry>,

    #[serde(rename = "isClassFeatureVariant")]
    pub is_class_feature_variant: Option<bool>,

    pub consumes: Option<Consumes>,

    #[serde(rename = "additionalSpells")]
    pub additional_spells: Option<Vec<serde_json::Value>>,

    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    pub level: Option<PrerequisiteLevel>,
    pub spell: Option<Vec<String>>,
    pub pact: Option<String>,
    pub patron: Option<String>,
    pub feature: Option<Vec<String>>,
    pub item: Option<Vec<String>>,
    pub other_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrerequisiteLevel {
    Simple(i32),
    Complex {
        level: i32,
        class: Option<ClassRef>,
        subclass: Option<SubclassRef>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassRef {
    pub name: String,
    pub source: Option<String>, // Make source optional
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassRef {
    pub name: String,
    pub source: Option<String>, // Make source optional
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumes {
    pub name: String,
    pub amount: Option<i32>,
}

// Summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalFeatureSummary {
    pub name: String,
    pub source: String,
    pub feature_types: Vec<String>,
    pub feature_type_full: String, // Formatted display name
    pub prerequisite_text: String,
    pub grants_spells: bool,
}

impl From<&OptionalFeature> for OptionalFeatureSummary {
    fn from(opt: &OptionalFeature) -> Self {
        Self {
            name: opt.name.clone(),
            source: opt.source.clone(),
            feature_types: opt.feature_type.clone(),
            feature_type_full: format_feature_types(&opt.feature_type),
            prerequisite_text: format_prerequisites(&opt.prerequisite),
            grants_spells: opt.additional_spells.is_some(),
        }
    }
}

fn format_feature_types(types: &[String]) -> String {
    let formatted: Vec<String> = types
        .iter()
        .map(|t| {
            {
                match t.as_str() {
                    "AI" => "Artificer Infusion",
                    "ED" => "Elemental Discipline",
                    "EI" => "Eldritch Invocation",
                    "MM" => "Metamagic",
                    "MV" => "Maneuver",
                    "MV:B" => "Maneuver (Battle Master)",
                    "MV:C2-UA" => "Maneuver (Cavalier V2 UA)",
                    "AS:V1-UA" => "Arcane Shot (V1 UA)",
                    "AS:V2-UA" => "Arcane Shot (V2 UA)",
                    "AS" => "Arcane Shot",
                    "OTH" => "Other",
                    "FS:F" => "Fighting Style (Fighter)",
                    "FS:B" => "Fighting Style (Bard)",
                    "FS:P" => "Fighting Style (Paladin)",
                    "FS:R" => "Fighting Style (Ranger)",
                    "PB" => "Pact Boon",
                    "OR" => "Onomancy Resonant",
                    "RN" => "Rune Knight Rune",
                    "AF" => "Alchemical Formula",
                    "TT" => "Traveler's Trick",
                    _ => t,
                }
            }
            .to_string()
        })
        .collect();

    formatted.join(", ")
}

fn format_prerequisites(prereqs: &Option<Vec<Prerequisite>>) -> String {
    if let Some(prereqs) = prereqs {
        let parts: Vec<String> = prereqs
            .iter()
            .filter_map(|p| {
                if let Some(level) = &p.level {
                    match level {
                        PrerequisiteLevel::Simple(lvl) => Some(format!("Level {}", lvl)),
                        PrerequisiteLevel::Complex {
                            level,
                            class,
                            subclass,
                        } => {
                            let mut text = String::new();
                            if let Some(cls) = class {
                                text.push_str(&cls.name);
                                if let Some(sub) = subclass {
                                    text.push_str(&format!(" ({})", sub.name));
                                }
                                text.push_str(&format!(" Level {}", level));
                            } else {
                                text.push_str(&format!("Level {}", level));
                            }
                            Some(text)
                        }
                    }
                } else if let Some(pact) = &p.pact {
                    Some(format!("Pact of the {}", pact))
                } else if let Some(patron) = &p.patron {
                    Some(format!("{} Patron", patron))
                } else if let Some(spells) = &p.spell {
                    if !spells.is_empty() {
                        Some(format!("{} cantrip", spells[0].replace("#c", "")))
                    } else {
                        None
                    }
                } else if let Some(features) = &p.feature {
                    if !features.is_empty() {
                        Some(features.join(", "))
                    } else {
                        None
                    }
                } else {
                    p.other_summary.clone()
                }
            })
            .collect();

        if parts.is_empty() {
            String::new()
        } else {
            parts.join("; ")
        }
    } else {
        String::new()
    }
}

// Container for JSON parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionalFeatureData {
    #[serde(rename = "optionalfeature")]
    pub optional_features: Option<Vec<OptionalFeature>>,
}

// Database models
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = catalog_optional_features)]
pub struct CatalogOptionalFeature {
    pub id: i32,
    pub name: String,
    pub feature_types: Option<String>,
    pub feature_type_full: Option<String>,
    pub prerequisite_text: Option<String>,
    pub grants_spells: Option<bool>,
    pub source: String,
    pub page: Option<i32>,
    pub full_optional_feature_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = catalog_optional_features)]
pub struct NewCatalogOptionalFeature {
    pub name: String,
    pub feature_types: Option<String>,
    pub feature_type_full: Option<String>,
    pub prerequisite_text: Option<String>,
    pub grants_spells: Option<bool>,
    pub source: String,
    pub page: Option<i32>,
    pub full_optional_feature_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalFeatureFilters {
    pub name: Option<String>,
    pub feature_types: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub grants_spells: Option<bool>,
}

impl From<&CatalogOptionalFeature> for OptionalFeatureSummary {
    fn from(catalog: &CatalogOptionalFeature) -> Self {
        Self {
            name: catalog.name.clone(),
            source: catalog.source.clone(),
            feature_types: catalog
                .feature_types
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            feature_type_full: catalog.feature_type_full.clone().unwrap_or_default(),
            prerequisite_text: catalog.prerequisite_text.clone().unwrap_or_default(),
            grants_spells: catalog.grants_spells.unwrap_or(false),
        }
    }
}

impl From<&OptionalFeature> for NewCatalogOptionalFeature {
    fn from(opt: &OptionalFeature) -> Self {
        Self {
            name: opt.name.clone(),
            feature_types: serde_json::to_string(&opt.feature_type).ok(),
            feature_type_full: Some(format_feature_types(&opt.feature_type)),
            prerequisite_text: Some(format_prerequisites(&opt.prerequisite)),
            grants_spells: Some(opt.additional_spells.is_some()),
            source: opt.source.clone(),
            page: opt.page,
            full_optional_feature_json: serde_json::to_string(opt).unwrap_or_default(),
        }
    }
}
