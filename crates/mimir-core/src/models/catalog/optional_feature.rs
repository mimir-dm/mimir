//! OptionalFeature Model
//!
//! Represents optional features like Eldritch Invocations, Metamagic Options,
//! Fighting Styles, Maneuvers, etc.

use crate::schema::optional_features;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// An optional feature from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = optional_features)]
#[diesel(primary_key(id))]
pub struct OptionalFeature {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    /// Feature type code (EI = Eldritch Invocation, MM = Metamagic, etc.)
    pub feature_type: Option<String>,
    pub data: String,
}

impl OptionalFeature {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Check if this is an Eldritch Invocation.
    pub fn is_eldritch_invocation(&self) -> bool {
        self.feature_type.as_ref().map_or(false, |t| t == "EI")
    }

    /// Check if this is a Metamagic option.
    pub fn is_metamagic(&self) -> bool {
        self.feature_type.as_ref().map_or(false, |t| t == "MM")
    }

    /// Check if this is a Fighting Style.
    pub fn is_fighting_style(&self) -> bool {
        self.feature_type.as_ref().map_or(false, |t| t == "FS")
    }

    /// Check if this is a Battle Master Maneuver.
    pub fn is_maneuver(&self) -> bool {
        self.feature_type.as_ref().map_or(false, |t| t == "MV")
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = optional_features)]
pub struct NewOptionalFeature<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub feature_type: Option<&'a str>,
    pub data: &'a str,
}

impl<'a> NewOptionalFeature<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self {
            name,
            source,
            feature_type: None,
            data,
        }
    }

    pub fn with_feature_type(mut self, feature_type: &'a str) -> Self {
        self.feature_type = Some(feature_type);
        self
    }
}

/// Filters for searching optional features.
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct OptionalFeatureFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
    pub sources: Option<Vec<String>>,
    pub feature_type: Option<String>,
}

impl OptionalFeatureFilter {
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
    fn test_new_optional_feature() {
        let feature = NewOptionalFeature::new("Agonizing Blast", "PHB", r#"{"name":"Agonizing Blast"}"#)
            .with_feature_type("EI");
        assert_eq!(feature.name, "Agonizing Blast");
        assert_eq!(feature.feature_type, Some("EI"));
    }
}
