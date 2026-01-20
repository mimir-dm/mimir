//! Template information domain types
//!
//! This module contains domain types for representing template metadata
//! in a structured, user-friendly format.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Structured information about a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Unique identifier for the template.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Description of the template's purpose.
    pub purpose: String,
    /// Board level (campaign, module, session).
    pub level: String,
    /// Type classification of the template.
    pub template_type: String,
    /// Variables that can be customized.
    pub variables: Vec<TemplateVariable>,
}

/// Template variable definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name used in the template.
    pub name: String,
    /// Expected type (string, number, boolean, etc.).
    pub var_type: String,
    /// Description of what this variable controls.
    pub description: String,
    /// Default value if not provided.
    pub default: JsonValue,
    /// Whether this variable must be provided.
    pub required: bool,
}
