use super::types::{Entry, Image};
use crate::schema::catalog_languages;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,

    #[serde(rename = "type")]
    pub language_type: Option<String>, // standard, exotic, secret, etc.

    pub script: Option<String>,

    #[serde(rename = "typicalSpeakers")]
    pub typical_speakers: Option<Vec<String>>,

    #[serde(default)]
    pub entries: Vec<Entry>,

    #[serde(rename = "basicRules")]
    pub basic_rules: Option<bool>,

    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,

    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,

    // Additional fields from expanded sources
    pub fonts: Option<Vec<String>>,
    pub dialects: Option<Vec<String>>,
}

// Summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSummary {
    pub name: String,
    pub source: String,
    pub language_type: String,
    pub script: String,
    pub typical_speakers: String,
}

impl From<&Language> for LanguageSummary {
    fn from(lang: &Language) -> Self {
        Self {
            name: lang.name.clone(),
            source: lang.source.clone(),
            language_type: format_language_type(&lang.language_type),
            script: lang.script.clone().unwrap_or_else(|| "—".to_string()),
            typical_speakers: format_speakers(&lang.typical_speakers),
        }
    }
}

fn format_language_type(lang_type: &Option<String>) -> String {
    match lang_type.as_deref() {
        Some("standard") => "Standard".to_string(),
        Some("exotic") => "Exotic".to_string(),
        Some("secret") => "Secret".to_string(),
        Some("dead") => "Dead".to_string(),
        Some("primordial") => "Primordial Dialect".to_string(),
        Some(other) => other.to_string(),
        None => "Standard".to_string(),
    }
}

fn format_speakers(speakers: &Option<Vec<String>>) -> String {
    if let Some(speakers) = speakers {
        if speakers.is_empty() {
            return "—".to_string();
        }
        // For the summary view, just extract readable names
        speakers
            .iter()
            .map(|s| {
                if s.starts_with("{@filter ") && s.ends_with("}") {
                    // Extract display name from filter tags
                    s.trim_start_matches("{@filter ")
                        .split('|')
                        .next()
                        .unwrap_or(s)
                        .to_string()
                } else if s.starts_with("{@race ") && s.ends_with("}") {
                    // Extract race name
                    s.trim_start_matches("{@race ")
                        .trim_end_matches("}")
                        .to_string()
                } else if s.starts_with("{@creature ") && s.ends_with("}") {
                    // Extract creature name
                    s.trim_start_matches("{@creature ")
                        .split('|')
                        .next()
                        .unwrap_or(s)
                        .trim_end_matches("}")
                        .to_string()
                } else if s.starts_with("{@class ") && s.ends_with("}") {
                    // Extract class name
                    s.trim_start_matches("{@class ")
                        .split('|')
                        .next()
                        .unwrap_or(s)
                        .trim_end_matches("}")
                        .to_string()
                } else if s.starts_with("{@item ") && s.ends_with("}") {
                    // Extract item name
                    s.trim_start_matches("{@item ")
                        .split('|')
                        .next()
                        .unwrap_or(s)
                        .trim_end_matches("}")
                        .to_string()
                } else {
                    s.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "—".to_string()
    }
}

// Container for JSON parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageData {
    pub language: Option<Vec<Language>>,
}

// Fluff data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageFluffData {
    #[serde(rename = "languageFluff")]
    pub language_fluff: Option<Vec<LanguageFluff>>,
}

// Database models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = catalog_languages)]
pub struct CatalogLanguage {
    pub id: i32,
    pub name: String,
    pub language_type: String,
    pub script: String,
    pub typical_speakers: String,
    pub source: String,
    pub full_language_json: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = catalog_languages)]
pub struct NewCatalogLanguage {
    pub name: String,
    pub language_type: String,
    pub script: String,
    pub typical_speakers: String,
    pub source: String,
    pub full_language_json: String,
}

impl From<Language> for NewCatalogLanguage {
    fn from(language: Language) -> Self {
        let summary = LanguageSummary::from(&language);
        let json = serde_json::to_string(&language).unwrap_or_default();

        Self {
            name: summary.name,
            language_type: summary.language_type,
            script: summary.script,
            typical_speakers: summary.typical_speakers,
            source: summary.source,
            full_language_json: json,
        }
    }
}

// Filter struct for search operations
#[derive(Debug, Default)]
pub struct LanguageFilters {
    pub name: Option<String>,
    pub search: Option<String>,
    pub language_types: Option<Vec<String>>,
    pub scripts: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
}
