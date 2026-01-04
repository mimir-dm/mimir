use super::types::{Entry, Image};
use crate::schema::catalog_conditions;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(rename = "basicRules")]
    pub basic_rules: Option<bool>,
    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
    #[serde(rename = "otherSources", default)]
    pub other_sources: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disease {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,
    #[serde(default)]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConditionData {
    #[serde(default)]
    pub condition: Option<Vec<Condition>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiseaseData {
    #[serde(default)]
    pub disease: Option<Vec<Disease>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConditionFluffData {
    #[serde(rename = "conditionFluff", default)]
    pub condition_fluff: Option<Vec<ConditionFluff>>,
}

// Combined enum for both conditions and diseases
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConditionOrDisease {
    Condition(Condition),
    Disease(Disease),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionSummary {
    pub name: String,
    pub source: String,
    pub item_type: String, // "Condition" or "Disease"
    pub description: String,
}

impl From<&Condition> for ConditionSummary {
    fn from(condition: &Condition) -> Self {
        // Extract first entry as description
        let description = extract_description(&condition.entries);

        Self {
            name: condition.name.clone(),
            source: condition.source.clone(),
            item_type: "Condition".to_string(),
            description,
        }
    }
}

impl From<&Disease> for ConditionSummary {
    fn from(disease: &Disease) -> Self {
        // Extract first entry as description
        let description = extract_description(&disease.entries);

        Self {
            name: disease.name.clone(),
            source: disease.source.clone(),
            item_type: "Disease".to_string(),
            description,
        }
    }
}

fn extract_description(entries: &[Entry]) -> String {
    use super::types::EntryObject;

    entries
        .first()
        .and_then(|entry| match entry {
            Entry::Text(s) => Some(s.clone()),
            Entry::Object(obj) => match obj {
                EntryObject::List { items, .. } => items
                    .first()
                    .and_then(|item| match item {
                        Entry::Text(s) => Some(s.clone()),
                        _ => None,
                    }),
                _ => None,
            },
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionWithDetails {
    #[serde(flatten)]
    pub item: ConditionOrDisease,
    pub fluff: Option<ConditionFluff>,
}

/// Database model for catalog_conditions table
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = catalog_conditions)]
pub struct CatalogCondition {
    pub id: i32,
    pub name: String,
    pub item_type: String,   // "Condition" or "Disease"
    pub description: String, // First entry as description
    pub source: String,
    pub full_condition_json: String, // Complete condition/disease JSON
}

/// For inserting new conditions into the database
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = catalog_conditions)]
pub struct NewCatalogCondition {
    pub name: String,
    pub item_type: String,
    pub description: String,
    pub source: String,
    pub full_condition_json: String,
}

/// Condition search filters for database queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConditionFilters {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub item_types: Option<Vec<String>>, // Filter by "Condition", "Disease"
    #[serde(default)]
    pub sources: Option<Vec<String>>,
    #[serde(default)]
    pub search: Option<String>, // General search term
}

impl From<CatalogCondition> for ConditionSummary {
    fn from(catalog_condition: CatalogCondition) -> Self {
        Self {
            name: catalog_condition.name,
            source: catalog_condition.source,
            item_type: catalog_condition.item_type,
            description: catalog_condition.description,
        }
    }
}

impl From<Condition> for NewCatalogCondition {
    fn from(condition: Condition) -> Self {
        let description = extract_description(&condition.entries);

        Self {
            name: condition.name.clone(),
            item_type: "Condition".to_string(),
            description,
            source: condition.source.clone(),
            full_condition_json: serde_json::to_string(&condition).unwrap_or_default(),
        }
    }
}

impl From<Disease> for NewCatalogCondition {
    fn from(disease: Disease) -> Self {
        let description = extract_description(&disease.entries);

        Self {
            name: disease.name.clone(),
            item_type: "Disease".to_string(),
            description,
            source: disease.source.clone(),
            full_condition_json: serde_json::to_string(&disease).unwrap_or_default(),
        }
    }
}
