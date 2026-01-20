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
    pub hd: Option<HitDice>,
    /// Saving throw proficiencies (array of ability abbreviations like "str", "dex", etc.)
    #[serde(default)]
    pub proficiency: Option<Vec<String>>,
    /// Class feature references at each level
    #[serde(default)]
    pub class_features: Option<Vec<ClassFeatureRef>>,
    #[serde(default)]
    pub starting_proficiencies: Option<StartingProficiencies>,
    #[serde(default)]
    pub multiclassing: Option<Multiclassing>,
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
    pub class_table_groups: Option<Vec<ClassTableGroup>>,
    #[serde(default)]
    pub starting_equipment: Option<StartingEquipment>,
    #[serde(default)]
    pub optionalfeature_progression: Option<Vec<OptionalFeatureProgression>>,
    #[serde(default)]
    pub fluff: Option<ClassFluff>,
}

/// Hit dice specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitDice {
    pub number: u8,
    pub faces: u8,
}

/// A class feature reference - can be a string or object with metadata.
///
/// String format: "FeatureName|ClassName|ClassSource|Level|OptionalSource"
/// Object format: { "classFeature": "...", "gainSubclassFeature": true, "tableDisplayName": "..." }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClassFeatureRef {
    /// Simple string reference (e.g., "Spellcasting|Cleric||1")
    Simple(String),
    /// Object with metadata
    Object(ClassFeatureRefObject),
}

/// Class feature reference object with optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassFeatureRefObject {
    pub class_feature: String,
    #[serde(default)]
    pub gain_subclass_feature: Option<bool>,
    #[serde(default)]
    pub table_display_name: Option<String>,
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
    /// Ability score requirements (e.g., {"str": 13, "cha": 13})
    #[serde(default)]
    pub requirements: Option<MulticlassingRequirements>,
    #[serde(default)]
    pub proficiencies_gained: Option<MulticlassingProficiencies>,
}

/// Multiclassing requirements - ability score minimums.
///
/// Can be simple ability requirements (str: 13) or OR requirements
/// that are satisfied if any one is met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MulticlassingRequirements {
    #[serde(default)]
    pub str: Option<i32>,
    #[serde(default)]
    pub dex: Option<i32>,
    #[serde(default)]
    pub con: Option<i32>,
    #[serde(default)]
    pub int: Option<i32>,
    #[serde(default)]
    pub wis: Option<i32>,
    #[serde(default)]
    pub cha: Option<i32>,
    /// Alternative requirements (OR condition)
    #[serde(default)]
    pub or: Option<Vec<MulticlassingRequirements>>,
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
    /// Structured equipment data for each choice
    #[serde(default, rename = "defaultData")]
    pub default_data: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub gold_alternative: Option<String>,
}

/// A class table group defining columns and per-level data.
///
/// Table groups are used to display class progression tables.
/// Rows contain values that may include 5etools tags for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassTableGroup {
    /// Column labels (may contain 5etools filter tags)
    #[serde(default)]
    pub col_labels: Option<Vec<String>>,
    /// Column styling hints
    #[serde(default)]
    pub col_styles: Option<Vec<String>>,
    /// Row data (20 rows for levels 1-20)
    /// Values can be numbers or strings with 5etools tags
    #[serde(default)]
    pub rows: Option<Vec<Vec<serde_json::Value>>>,
    /// Subheader rows displayed between data rows
    #[serde(default)]
    pub row_sub_headers: Option<Vec<serde_json::Value>>,
    /// Title for this table group
    #[serde(default)]
    pub title: Option<String>,
}

/// Optional feature progression (e.g., Warlock invocations, Metamagic).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalFeatureProgression {
    /// Feature type codes (e.g., "EI" for Eldritch Invocations)
    #[serde(default)]
    pub feature_type: Option<Vec<String>>,
    /// Display name for the feature
    #[serde(default)]
    pub name: Option<String>,
    /// Progression - either array of counts per level or object with level keys
    #[serde(default)]
    pub progression: Option<OptionalFeatureProgressionValue>,
}

/// Progression value - can be array or level-keyed object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalFeatureProgressionValue {
    /// Array of counts for each level (index 0 = level 1)
    Array(Vec<i32>),
    /// Object with level numbers as keys
    Object(std::collections::HashMap<String, i32>),
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
    /// Subclass feature references (strings like "FeatureName|Class||Subclass||Level")
    #[serde(default)]
    pub subclass_features: Option<Vec<String>>,
    #[serde(default)]
    pub subclass_table_groups: Option<Vec<ClassTableGroup>>,
    #[serde(default)]
    pub caster_progression: Option<String>,
    #[serde(default)]
    pub spellcasting_ability: Option<String>,
    #[serde(default)]
    pub cantrip_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub spells_known_progression: Option<Vec<u8>>,
    /// Additional spells granted by the subclass
    /// Contains level-keyed objects with spell lists
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
        // Format hit die using typed HitDice struct
        let hit_dice = if let Some(hd) = &class.hd {
            format!("{}d{}", hd.number, hd.faces)
        } else {
            "1d6".to_string()
        };

        // Format proficiency - extract saving throws from startingProficiencies
        let proficiency = if let Some(start_prof) = &class.starting_proficiencies {
            if let Some(saves) = &start_prof.saving_throws {
                let formatted: Vec<String> = saves
                    .iter()
                    .map(|s| match s.as_str() {
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
                if !formatted.is_empty() {
                    formatted.join(", ")
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
                .map(|f| match f {
                    ClassFeatureRef::Simple(s) => s.clone(),
                    ClassFeatureRef::Object(obj) => obj.class_feature.clone(),
                })
                .unwrap_or_else(|| format!("A {} class", class.name))
                .chars()
                .take(200)
                .collect::<String>()
        } else {
            format!("A {} class", class.name)
        };

        // Serialize table groups for the summary (frontend needs JSON for rendering)
        let table_groups = class.class_table_groups.as_ref().map(|groups| {
            groups
                .iter()
                .filter_map(|g| serde_json::to_value(g).ok())
                .collect()
        });

        ClassSummary {
            name: class.name.clone(),
            source: class.source.clone(),
            page: class.page,
            hit_dice,
            proficiency,
            primary_ability,
            spellcasting_ability: class.spellcasting_ability.clone(),
            table_groups,
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
        // Format hit die using typed HitDice struct
        let hit_dice = if let Some(hd) = &class.hd {
            Some(format!("{}d{}", hd.number, hd.faces))
        } else {
            Some("1d6".to_string())
        };

        // Format proficiency - extract saving throws from startingProficiencies
        let proficiency = if let Some(start_prof) = &class.starting_proficiencies {
            if let Some(saves) = &start_prof.saving_throws {
                let formatted: Vec<String> = saves
                    .iter()
                    .map(|s| match s.as_str() {
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
                if !formatted.is_empty() {
                    Some(formatted.join(", "))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hit_dice() {
        let json = r#"{"number": 1, "faces": 10}"#;
        let hd: HitDice = serde_json::from_str(json).unwrap();
        assert_eq!(hd.number, 1);
        assert_eq!(hd.faces, 10);
    }

    #[test]
    fn test_parse_class_feature_ref_simple() {
        let json = r#""Spellcasting|Cleric||1""#;
        let feature: ClassFeatureRef = serde_json::from_str(json).unwrap();
        match feature {
            ClassFeatureRef::Simple(s) => assert_eq!(s, "Spellcasting|Cleric||1"),
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_parse_class_feature_ref_object() {
        let json = r#"{"classFeature": "Divine Domain|Cleric||1", "gainSubclassFeature": true}"#;
        let feature: ClassFeatureRef = serde_json::from_str(json).unwrap();
        match feature {
            ClassFeatureRef::Object(obj) => {
                assert_eq!(obj.class_feature, "Divine Domain|Cleric||1");
                assert_eq!(obj.gain_subclass_feature, Some(true));
            }
            _ => panic!("Expected Object variant"),
        }
    }

    #[test]
    fn test_parse_multiclassing_requirements() {
        let json = r#"{"str": 13, "cha": 13}"#;
        let reqs: MulticlassingRequirements = serde_json::from_str(json).unwrap();
        assert_eq!(reqs.str, Some(13));
        assert_eq!(reqs.cha, Some(13));
        assert_eq!(reqs.dex, None);
    }

    #[test]
    fn test_parse_optional_feature_progression_array() {
        let json = r#"{"featureType": ["EI"], "name": "Eldritch Invocations", "progression": [0, 2, 2, 2, 3]}"#;
        let prog: OptionalFeatureProgression = serde_json::from_str(json).unwrap();
        assert_eq!(prog.name, Some("Eldritch Invocations".to_string()));
        match prog.progression {
            Some(OptionalFeatureProgressionValue::Array(arr)) => {
                assert_eq!(arr, vec![0, 2, 2, 2, 3]);
            }
            _ => panic!("Expected Array variant"),
        }
    }

    #[test]
    fn test_parse_optional_feature_progression_object() {
        let json = r#"{"featureType": ["PB"], "name": "Pact Boon", "progression": {"3": 1}}"#;
        let prog: OptionalFeatureProgression = serde_json::from_str(json).unwrap();
        assert_eq!(prog.name, Some("Pact Boon".to_string()));
        match prog.progression {
            Some(OptionalFeatureProgressionValue::Object(map)) => {
                assert_eq!(map.get("3"), Some(&1));
            }
            _ => panic!("Expected Object variant"),
        }
    }

    #[test]
    fn test_parse_class_table_group() {
        let json = r#"{
            "colLabels": ["Cantrips Known", "Spells Known"],
            "rows": [[2, 2], [2, 3], [2, 4]]
        }"#;
        let group: ClassTableGroup = serde_json::from_str(json).unwrap();
        assert_eq!(group.col_labels, Some(vec!["Cantrips Known".to_string(), "Spells Known".to_string()]));
        assert!(group.rows.is_some());
        let rows = group.rows.unwrap();
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn test_parse_starting_equipment() {
        let json = r#"{
            "additionalFromBackground": true,
            "default": ["a mace", "leather armor"],
            "goldAlternative": "5d4 x 10 gp"
        }"#;
        let equip: StartingEquipment = serde_json::from_str(json).unwrap();
        assert_eq!(equip.additional_from_background, Some(true));
        assert_eq!(equip.default, Some(vec!["a mace".to_string(), "leather armor".to_string()]));
        assert_eq!(equip.gold_alternative, Some("5d4 x 10 gp".to_string()));
    }

    #[test]
    fn test_parse_minimal_class() {
        let json = r#"{
            "name": "Fighter",
            "source": "PHB",
            "hd": {"number": 1, "faces": 10},
            "proficiency": ["str", "con"]
        }"#;
        let class: Class = serde_json::from_str(json).unwrap();
        assert_eq!(class.name, "Fighter");
        assert_eq!(class.source, "PHB");
        assert!(class.hd.is_some());
        let hd = class.hd.unwrap();
        assert_eq!(hd.faces, 10);
        assert_eq!(class.proficiency, Some(vec!["str".to_string(), "con".to_string()]));
    }

    #[test]
    fn test_parse_class_with_features() {
        let json = r#"{
            "name": "Cleric",
            "source": "PHB",
            "classFeatures": [
                "Spellcasting|Cleric||1",
                {"classFeature": "Divine Domain|Cleric||1", "gainSubclassFeature": true}
            ]
        }"#;
        let class: Class = serde_json::from_str(json).unwrap();
        assert_eq!(class.name, "Cleric");
        let features = class.class_features.unwrap();
        assert_eq!(features.len(), 2);
        match &features[0] {
            ClassFeatureRef::Simple(s) => assert!(s.contains("Spellcasting")),
            _ => panic!("Expected Simple variant for first feature"),
        }
        match &features[1] {
            ClassFeatureRef::Object(obj) => assert!(obj.gain_subclass_feature == Some(true)),
            _ => panic!("Expected Object variant for second feature"),
        }
    }

    #[test]
    fn test_parse_subclass() {
        let json = r#"{
            "name": "Knowledge Domain",
            "source": "PHB",
            "className": "Cleric",
            "classSource": "PHB",
            "subclassFeatures": [
                "Knowledge Domain|Cleric||Knowledge||1",
                "Channel Divinity: Knowledge of the Ages|Cleric||Knowledge||2"
            ]
        }"#;
        let subclass: Subclass = serde_json::from_str(json).unwrap();
        assert_eq!(subclass.name, "Knowledge Domain");
        assert_eq!(subclass.class_name, "Cleric");
        let features = subclass.subclass_features.unwrap();
        assert_eq!(features.len(), 2);
        assert!(features[0].contains("Knowledge Domain"));
    }
}
