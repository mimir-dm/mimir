//! VariantRule Model
//!
//! Represents optional and variant rules from rulebooks.

use crate::schema::variant_rules;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A variant or optional rule from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = variant_rules)]
#[diesel(primary_key(id))]
pub struct VariantRule {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    /// Rule type (O = Optional, V = Variant)
    pub rule_type: Option<String>,
    pub data: String,
}

impl VariantRule {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Check if this is an optional rule.
    pub fn is_optional(&self) -> bool {
        self.rule_type.as_ref().map_or(false, |t| t == "O")
    }

    /// Check if this is a variant rule.
    pub fn is_variant(&self) -> bool {
        self.rule_type.as_ref().map_or(false, |t| t == "V")
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = variant_rules)]
pub struct NewVariantRule<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub rule_type: Option<&'a str>,
    pub data: &'a str,
}

impl<'a> NewVariantRule<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self {
            name,
            source,
            rule_type: None,
            data,
        }
    }

    pub fn with_type(mut self, rule_type: &'a str) -> Self {
        self.rule_type = Some(rule_type);
        self
    }
}

/// Filters for searching variant rules.
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct VariantRuleFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
    pub sources: Option<Vec<String>>,
    pub rule_type: Option<String>,
}

impl VariantRuleFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_empty_sources_filter(&self) -> bool {
        matches!(&self.sources, Some(sources) if sources.is_empty())
    }

    pub fn effective_sources(&self) -> Option<Vec<String>> {
        match (&self.sources, &self.source) {
            (Some(sources), _) if !sources.is_empty() => Some(sources.clone()),
            (_, Some(source)) => Some(vec![source.clone()]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_variant_rule() {
        let rule = NewVariantRule::new("Flanking", "DMG", r#"{"name":"Flanking"}"#)
            .with_type("O");
        assert_eq!(rule.name, "Flanking");
        assert_eq!(rule.rule_type, Some("O"));
    }
}
