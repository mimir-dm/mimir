use super::types::Entry;
use crate::schema::catalog_variant_rules;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantRule {
    pub name: String,
    pub source: String,

    #[serde(rename = "ruleType")]
    pub rule_type: Option<String>,

    pub page: Option<i32>,

    #[serde(default)]
    pub entries: Vec<Entry>,

    #[serde(flatten)]
    pub other_fields: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantRuleData {
    pub variantrule: Option<Vec<VariantRule>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantRuleSummary {
    pub name: String,
    pub source: String,
    pub rule_type: Option<String>,
    pub page: Option<i32>,
}

impl From<&VariantRule> for VariantRuleSummary {
    fn from(rule: &VariantRule) -> Self {
        VariantRuleSummary {
            name: rule.name.clone(),
            source: rule.source.clone(),
            rule_type: rule.rule_type.clone(),
            page: rule.page,
        }
    }
}

// Database models for catalog_variant_rules table
#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = catalog_variant_rules)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogVariantRule {
    pub id: i32,
    pub name: String,
    pub rule_type: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_variant_rule_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = catalog_variant_rules)]
pub struct NewCatalogVariantRule {
    pub name: String,
    pub rule_type: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_variant_rule_json: String,
}

#[derive(Debug, Default)]
pub struct VariantRuleFilters {
    pub name: Option<String>,
    pub rule_types: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
}

impl From<&VariantRule> for NewCatalogVariantRule {
    fn from(rule: &VariantRule) -> Self {
        NewCatalogVariantRule {
            name: rule.name.clone(),
            rule_type: rule.rule_type.clone(),
            source: rule.source.clone(),
            page: rule.page,
            full_variant_rule_json: serde_json::to_string(rule).unwrap_or_default(),
        }
    }
}

impl From<&CatalogVariantRule> for VariantRuleSummary {
    fn from(catalog_rule: &CatalogVariantRule) -> Self {
        VariantRuleSummary {
            name: catalog_rule.name.clone(),
            source: catalog_rule.source.clone(),
            rule_type: catalog_rule.rule_type.clone(),
            page: catalog_rule.page,
        }
    }
}
