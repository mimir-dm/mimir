//! Template frontmatter schema and parsing

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Frontmatter structure for all template documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFrontmatter {
    /// Unique identifier for this template
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Template type (campaign_pitch, module_overview, etc.)
    #[serde(rename = "type")]
    pub template_type: String,

    /// Document level (campaign, module, session, handout)
    pub level: String,

    /// Brief description of the template's purpose
    pub purpose: String,

    /// List of template variables used in this document
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,

    /// Author of the template
    #[serde(default = "default_author")]
    pub author: String,
}

fn default_author() -> String {
    "Mimir Team".to_string()
}

/// Template variable definition with required default value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name (without delimiters)
    pub name: String,

    /// Data type (string, number, boolean, list, object, etc.)
    #[serde(rename = "type")]
    pub var_type: String,

    /// Description of what this variable represents
    pub description: String,

    /// Default value for this variable (required)
    pub default: JsonValue,

    /// Whether this variable is required
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// Example frontmatter YAML structure:
/// ```yaml
/// ---
/// id: campaign-pitch
/// title: Campaign Pitch Template
/// type: campaign_pitch
/// level: campaign
/// purpose: Create a one-page pitch to excite players about your campaign concept
/// author: Mimir Team
/// variables:
///   - name: campaign_name
///     type: string
///     description: The name of your campaign
///     default: "[Campaign Name]"
///     required: true
///   - name: genre
///     type: string
///     description: Primary genre and tone
///     default: "Fantasy Adventure"
///     required: true
///   - name: pillars
///     type: object
///     description: Campaign pillar ratings (1-5)
///     default:
///       combat: 3
///       exploration: 3
///       social: 3
///       mystery: 3
///     required: true
/// ---
/// ```
impl TemplateFrontmatter {
    /// Parse frontmatter from a markdown document using gray_matter
    pub fn parse_from_markdown(content: &str) -> Option<Self> {
        // Parse the markdown with gray_matter
        // We parse directly into serde_yaml::Value
        let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
        let parsed = matter.parse::<serde_yaml::Value>(content).ok()?;

        // If there's no frontmatter data, return None
        parsed.data.as_ref()?;

        // Convert the parsed YAML Value to our TemplateFrontmatter struct
        let yaml_value = parsed.data?;
        serde_yaml::from_value(yaml_value).ok()
    }

    /// Extract the content after frontmatter using gray_matter
    pub fn extract_content(markdown: &str) -> String {
        // Parse the markdown with gray_matter
        // We use serde_yaml::Value as the type parameter since we only care about content
        let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
        match matter.parse::<serde_yaml::Value>(markdown) {
            Ok(parsed) => parsed.content,
            Err(_) => markdown.to_string(), // If parsing fails, return original content
        }
    }

    /// Convert to JSON for database storage
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Create variables schema JSON
    pub fn variables_schema(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.variables)
    }

    /// Extract defaults as a map from variables
    pub fn defaults_map(&self) -> serde_json::Map<String, JsonValue> {
        let mut defaults = serde_json::Map::new();
        for var in &self.variables {
            defaults.insert(var.name.clone(), var.default.clone());
        }
        defaults
    }
}
