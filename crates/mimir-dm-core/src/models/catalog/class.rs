//! Class catalog models

use super::types::{Entry, Image, ProficiencyItem};
use serde::{Deserialize, Serialize};

/// A D&D 5e character class
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub srd: Option<bool>,
    #[serde(default)]
    pub basic_rules: Option<bool>,
    #[serde(default)]
    pub hd: Option<serde_json::Value>, // Can be HitDice object or something else
    #[serde(default)]
    pub proficiency: Option<serde_json::Value>, // Can be array or object
    #[serde(default)]
    pub class_features: Option<Vec<serde_json::Value>>, // Can be strings or objects
    #[serde(default)]
    pub starting_proficiencies: Option<serde_json::Value>,
    #[serde(default)]
    pub multiclassing: Option<serde_json::Value>,
    #[serde(default)]
    pub subclass_title: Option<String>,
    #[serde(default)]
    pub caster_progression: Option<String>, // "full", "1/2", "1/3", "pact"
    #[serde(default)]
    pub cantrip_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub spells_known_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub prepared_spells: Option<String>, // Formula for prepared spells
    #[serde(default)]
    pub spellcasting_ability: Option<String>, // "int", "wis", "cha"
    #[serde(default)]
    pub class_table_groups: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub starting_equipment: Option<serde_json::Value>,
    #[serde(default)]
    pub optionalfeature_progression: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub fluff: Option<ClassFluff>,
}

/// Hit dice specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitDice {
    pub number: u8,
    pub faces: u8,
}

/// Starting proficiencies for a class
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartingProficiencies {
    #[serde(default)]
    pub armor: Option<Vec<String>>,
    #[serde(default)]
    pub weapons: Option<Vec<String>>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default)]
    pub skills: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub saving_throws: Option<Vec<String>>,
}

/// Multiclassing requirements and benefits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Multiclassing {
    #[serde(default)]
    pub requirements: Option<serde_json::Value>, // Complex requirement objects
    #[serde(default)]
    pub proficiencies_gained: Option<MulticlassingProficiencies>,
}

/// Proficiencies gained from multiclassing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassingProficiencies {
    #[serde(default)]
    pub armor: Option<Vec<String>>,
    #[serde(default)]
    pub weapons: Option<Vec<String>>,
    #[serde(default)]
    pub tools: Option<Vec<String>>,
    #[serde(default)]
    pub skills: Option<Vec<ProficiencyItem>>,
}

/// Starting equipment options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartingEquipment {
    #[serde(default)]
    pub additional_from_background: Option<bool>,
    #[serde(default)]
    pub default: Option<Vec<String>>,
    #[serde(default)]
    pub gold_alternative: Option<String>,
}

/// Container for class data from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassData {
    #[serde(default, rename = "class")]
    pub classes: Vec<Class>,
    #[serde(default)]
    pub subclass: Option<Vec<Subclass>>,
    #[serde(default, rename = "classFeature")]
    pub class_features: Option<Vec<ClassFeature>>,
    #[serde(default, rename = "subclassFeature")]
    pub subclass_features: Option<Vec<SubclassFeature>>,
}

/// A character subclass
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subclass {
    pub name: String,
    #[serde(default)]
    pub short_name: Option<String>,
    pub source: String,
    pub class_name: String,
    pub class_source: String,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub subclass_features: Option<serde_json::Value>, // Can be array of strings or objects
    #[serde(default)]
    pub subclass_table_groups: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub caster_progression: Option<String>,
    #[serde(default)]
    pub spellcasting_ability: Option<String>,
    #[serde(default)]
    pub cantrip_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub spells_known_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub additional_spells: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub fluff: Option<SubclassFluff>,
    #[serde(default)]
    pub intro_description: Option<String>,
}

/// Class feature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassFeature {
    pub name: String,
    pub source: String,
    pub class_name: String,
    pub class_source: String,
    pub level: u8,
    #[serde(default)]
    pub entries: Vec<Entry>,
    pub page: Option<u32>,
    #[serde(default)]
    pub srd: Option<bool>,
}

/// Subclass feature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubclassFeature {
    pub name: String,
    pub source: String,
    pub class_name: String,
    pub class_source: String,
    #[serde(default)]
    pub subclass_short_name: Option<String>,
    pub subclass_source: String,
    pub level: u8,
    #[serde(default)]
    pub entries: Vec<Entry>,
    pub page: Option<u32>,
}

/// Container for class feature data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassFeatureData {
    #[serde(default)]
    pub class_feature: Option<Vec<ClassFeature>>,
    #[serde(default)]
    pub subclass_feature: Option<Vec<SubclassFeature>>,
}

/// Class fluff (descriptive text)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassFluff {
    pub name: String,
    pub source: String,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub images: Vec<Image>,
}

/// Subclass fluff (descriptive text)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubclassFluff {
    pub name: String,
    #[serde(default)]
    pub short_name: Option<String>,
    pub source: String,
    pub class_name: String,
    pub class_source: String,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub images: Vec<Image>,
}

/// Container for class fluff data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassFluffData {
    #[serde(default)]
    pub class_fluff: Option<Vec<ClassFluff>>,
    #[serde(default)]
    pub subclass_fluff: Option<Vec<SubclassFluff>>,
}

/// Simplified class for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassSummary {
    pub name: String,
    pub source: String,
    pub page: Option<u32>,
    pub hit_dice: String,
    pub proficiency: String,
    pub primary_ability: String,
    pub spellcasting_ability: Option<String>,
    pub table_groups: Option<Vec<serde_json::Value>>,
    pub subclass_title: Option<String>,
    pub description: String,
    // New fields for unified class/subclass rows
    pub subclass_name: Option<String>,
    pub row_type: String, // "base" or "subclass"
}

impl From<&Class> for ClassSummary {
    fn from(class: &Class) -> Self {
        // Format hit die
        let hit_dice = if let Some(hd) = &class.hd {
            if let Some(obj) = hd.as_object() {
                if let (Some(number), Some(faces)) = (obj.get("number"), obj.get("faces")) {
                    format!(
                        "{}d{}",
                        number.as_u64().unwrap_or(1),
                        faces.as_u64().unwrap_or(6)
                    )
                } else {
                    "1d6".to_string()
                }
            } else {
                "1d6".to_string()
            }
        } else {
            "1d6".to_string()
        };

        // Format proficiency - extract saving throws from startingProficiencies
        let proficiency = if let Some(start_prof) = &class.starting_proficiencies {
            if let Some(obj) = start_prof.as_object() {
                if let Some(saves_val) = obj.get("savingThrows") {
                    if let Some(saves_arr) = saves_val.as_array() {
                        let saves: Vec<String> = saves_arr
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| match s {
                                "str" => "STR",
                                "dex" => "DEX",
                                "con" => "CON",
                                "int" => "INT",
                                "wis" => "WIS",
                                "cha" => "CHA",
                                _ => s,
                            })
                            .map(|s| s.to_string())
                            .collect();
                        if !saves.is_empty() {
                            saves.join(", ")
                        } else {
                            "None".to_string()
                        }
                    } else {
                        "None".to_string()
                    }
                } else {
                    "None".to_string()
                }
            } else {
                "None".to_string()
            }
        } else {
            "None".to_string()
        };

        // Determine primary ability based on class name
        let primary_ability = match class.name.to_lowercase().as_str() {
            "barbarian" | "fighter" => "Strength".to_string(),
            "rogue" | "ranger" | "monk" => "Dexterity".to_string(),
            "wizard" => "Intelligence".to_string(),
            "cleric" | "druid" => "Wisdom".to_string(),
            "bard" | "paladin" | "sorcerer" | "warlock" => "Charisma".to_string(),
            _ => "Various".to_string(),
        };

        // Get a simple description from class features if available
        let description = if let Some(features) = &class.class_features {
            features
                .first()
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .chars()
                .take(200)
                .collect::<String>()
        } else {
            format!("A {} class", class.name)
        };

        ClassSummary {
            name: class.name.clone(),
            source: class.source.clone(),
            page: class.page,
            hit_dice,
            proficiency,
            primary_ability,
            spellcasting_ability: class.spellcasting_ability.clone(),
            table_groups: class.class_table_groups.clone(),
            subclass_title: class.subclass_title.clone(),
            description,
            subclass_name: None,
            row_type: "base".to_string(),
        }
    }
}

// Database models
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::catalog_classes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogClass {
    pub id: i32,
    pub name: String,
    pub hit_dice: Option<String>,
    pub primary_ability: Option<String>,
    pub proficiency: Option<String>,
    pub spellcasting_ability: Option<String>,
    pub subclass_title: Option<String>,
    pub caster_progression: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_class_json: String,
    pub fluff_json: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::catalog_classes)]
pub struct NewCatalogClass {
    pub name: String,
    pub hit_dice: Option<String>,
    pub primary_ability: Option<String>,
    pub proficiency: Option<String>,
    pub spellcasting_ability: Option<String>,
    pub subclass_title: Option<String>,
    pub caster_progression: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_class_json: String,
    pub fluff_json: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::catalog_subclasses)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogSubclass {
    pub id: i32,
    pub name: String,
    pub short_name: Option<String>,
    pub class_name: String,
    pub class_source: String,
    pub source: String,
    pub page: Option<i32>,
    pub caster_progression: Option<String>,
    pub spellcasting_ability: Option<String>,
    pub full_subclass_json: String,
    pub fluff_json: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::catalog_subclasses)]
pub struct NewCatalogSubclass {
    pub name: String,
    pub short_name: Option<String>,
    pub class_name: String,
    pub class_source: String,
    pub source: String,
    pub page: Option<i32>,
    pub caster_progression: Option<String>,
    pub spellcasting_ability: Option<String>,
    pub full_subclass_json: String,
    pub fluff_json: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::catalog_class_features)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogClassFeature {
    pub id: i32,
    pub name: String,
    pub class_name: String,
    pub class_source: String,
    pub level: i32,
    pub source: String,
    pub page: Option<i32>,
    pub full_feature_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::catalog_class_features)]
pub struct NewCatalogClassFeature {
    pub name: String,
    pub class_name: String,
    pub class_source: String,
    pub level: i32,
    pub source: String,
    pub page: Option<i32>,
    pub full_feature_json: String,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::catalog_subclass_features)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogSubclassFeature {
    pub id: i32,
    pub name: String,
    pub class_name: String,
    pub class_source: String,
    pub subclass_short_name: Option<String>,
    pub subclass_source: String,
    pub level: i32,
    pub source: String,
    pub page: Option<i32>,
    pub full_feature_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::catalog_subclass_features)]
pub struct NewCatalogSubclassFeature {
    pub name: String,
    pub class_name: String,
    pub class_source: String,
    pub subclass_short_name: Option<String>,
    pub subclass_source: String,
    pub level: i32,
    pub source: String,
    pub page: Option<i32>,
    pub full_feature_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClassFilters {
    pub name: Option<String>,
    pub sources: Option<Vec<String>>,
    pub has_spellcasting: Option<bool>,
    pub primary_abilities: Option<Vec<String>>,
}

// Conversion from Class to NewCatalogClass
impl From<&Class> for NewCatalogClass {
    fn from(class: &Class) -> Self {
        // Format hit die
        let hit_dice = if let Some(hd) = &class.hd {
            if let Some(obj) = hd.as_object() {
                if let (Some(number), Some(faces)) = (obj.get("number"), obj.get("faces")) {
                    Some(format!(
                        "{}d{}",
                        number.as_u64().unwrap_or(1),
                        faces.as_u64().unwrap_or(6)
                    ))
                } else {
                    Some("1d6".to_string())
                }
            } else {
                Some("1d6".to_string())
            }
        } else {
            Some("1d6".to_string())
        };

        // Format proficiency - extract saving throws from startingProficiencies
        let proficiency = if let Some(start_prof) = &class.starting_proficiencies {
            if let Some(obj) = start_prof.as_object() {
                if let Some(saves_val) = obj.get("savingThrows") {
                    if let Some(saves_arr) = saves_val.as_array() {
                        let saves: Vec<String> = saves_arr
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| match s {
                                "str" => "STR",
                                "dex" => "DEX",
                                "con" => "CON",
                                "int" => "INT",
                                "wis" => "WIS",
                                "cha" => "CHA",
                                _ => s,
                            })
                            .map(|s| s.to_string())
                            .collect();
                        if !saves.is_empty() {
                            Some(saves.join(", "))
                        } else {
                            Some("None".to_string())
                        }
                    } else {
                        Some("None".to_string())
                    }
                } else {
                    Some("None".to_string())
                }
            } else {
                Some("None".to_string())
            }
        } else {
            Some("None".to_string())
        };

        // Determine primary ability based on class name
        let primary_ability = match class.name.to_lowercase().as_str() {
            "barbarian" | "fighter" => Some("Strength".to_string()),
            "rogue" | "ranger" | "monk" => Some("Dexterity".to_string()),
            "wizard" => Some("Intelligence".to_string()),
            "cleric" | "druid" => Some("Wisdom".to_string()),
            "bard" | "paladin" | "sorcerer" | "warlock" => Some("Charisma".to_string()),
            _ => Some("Various".to_string()),
        };

        NewCatalogClass {
            name: class.name.clone(),
            hit_dice,
            primary_ability,
            proficiency,
            spellcasting_ability: class.spellcasting_ability.clone(),
            subclass_title: class.subclass_title.clone(),
            caster_progression: class.caster_progression.clone(),
            source: class.source.clone(),
            page: class.page.map(|p| p as i32),
            full_class_json: serde_json::to_string(class).unwrap_or_default(),
            fluff_json: None, // Fluff data will be set separately during import
        }
    }
}

// Conversion from Subclass to NewCatalogSubclass
impl From<&Subclass> for NewCatalogSubclass {
    fn from(subclass: &Subclass) -> Self {
        NewCatalogSubclass {
            name: subclass.name.clone(),
            short_name: subclass.short_name.clone(),
            class_name: subclass.class_name.clone(),
            class_source: subclass.class_source.clone(),
            source: subclass.source.clone(),
            page: subclass.page.map(|p| p as i32),
            caster_progression: subclass.caster_progression.clone(),
            spellcasting_ability: subclass.spellcasting_ability.clone(),
            full_subclass_json: serde_json::to_string(subclass).unwrap_or_default(),
            fluff_json: None, // Fluff data will be set separately during import
        }
    }
}

// Conversion from ClassFeature to NewCatalogClassFeature
impl From<&ClassFeature> for NewCatalogClassFeature {
    fn from(feature: &ClassFeature) -> Self {
        NewCatalogClassFeature {
            name: feature.name.clone(),
            class_name: feature.class_name.clone(),
            class_source: feature.class_source.clone(),
            level: feature.level as i32,
            source: feature.source.clone(),
            page: feature.page.map(|p| p as i32),
            full_feature_json: serde_json::to_string(feature).unwrap_or_default(),
        }
    }
}

// Conversion from SubclassFeature to NewCatalogSubclassFeature
impl From<&SubclassFeature> for NewCatalogSubclassFeature {
    fn from(feature: &SubclassFeature) -> Self {
        NewCatalogSubclassFeature {
            name: feature.name.clone(),
            class_name: feature.class_name.clone(),
            class_source: feature.class_source.clone(),
            subclass_short_name: feature.subclass_short_name.clone(),
            subclass_source: feature.subclass_source.clone(),
            level: feature.level as i32,
            source: feature.source.clone(),
            page: feature.page.map(|p| p as i32),
            full_feature_json: serde_json::to_string(feature).unwrap_or_default(),
        }
    }
}

// Conversion from CatalogClass to ClassSummary for API responses
impl From<&CatalogClass> for ClassSummary {
    fn from(catalog: &CatalogClass) -> Self {
        ClassSummary {
            name: catalog.name.clone(),
            source: catalog.source.clone(),
            page: catalog.page.map(|p| p as u32),
            hit_dice: catalog
                .hit_dice
                .clone()
                .unwrap_or_else(|| "1d6".to_string()),
            proficiency: catalog
                .proficiency
                .clone()
                .unwrap_or_else(|| "None".to_string()),
            primary_ability: catalog
                .primary_ability
                .clone()
                .unwrap_or_else(|| "Various".to_string()),
            spellcasting_ability: catalog.spellcasting_ability.clone(),
            table_groups: None, // This would need to be parsed from full_class_json if needed
            subclass_title: catalog.subclass_title.clone(),
            description: format!("A {} class", catalog.name), // Simplified description
            subclass_name: None,
            row_type: "base".to_string(),
        }
    }
}

// Conversion from CatalogSubclass + base class info to ClassSummary for subclass rows
impl ClassSummary {
    pub fn from_subclass(subclass: &CatalogSubclass, base_class: &CatalogClass) -> Self {
        ClassSummary {
            name: base_class.name.clone(),   // Class name (e.g., "Barbarian")
            source: subclass.source.clone(), // Subclass source
            page: subclass.page.map(|p| p as u32),
            hit_dice: base_class
                .hit_dice
                .clone()
                .unwrap_or_else(|| "1d6".to_string()),
            proficiency: base_class
                .proficiency
                .clone()
                .unwrap_or_else(|| "None".to_string()),
            primary_ability: base_class
                .primary_ability
                .clone()
                .unwrap_or_else(|| "Various".to_string()),
            // Prefer subclass spellcasting ability if available, otherwise use base class
            spellcasting_ability: subclass
                .spellcasting_ability
                .clone()
                .or_else(|| base_class.spellcasting_ability.clone()),
            table_groups: None,
            subclass_title: base_class.subclass_title.clone(),
            description: format!("{} subclass", subclass.name),
            subclass_name: Some(subclass.name.clone()),
            row_type: "subclass".to_string(),
        }
    }
}
