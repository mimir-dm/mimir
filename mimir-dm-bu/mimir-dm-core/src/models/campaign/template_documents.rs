//! Template document database models and operations

use crate::schema::template_documents;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Database model for template documents
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = template_documents)]
pub struct TemplateDocument {
    pub document_id: String,
    pub version_number: i32,
    pub document_content: String,
    pub content_hash: String,
    pub document_type: Option<String>,
    pub document_level: Option<String>,
    pub purpose: Option<String>,
    pub variables_schema: Option<String>,
    pub default_values: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
    pub metadata: Option<String>,
}

/// New template document for insertion
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = template_documents)]
pub struct NewTemplateDocument {
    pub document_id: String,
    pub version_number: Option<i32>, // Will default to 1 if not specified
    pub document_content: String,
    pub content_hash: Option<String>, // Will be computed if not provided
    pub document_type: Option<String>,
    pub document_level: Option<String>,
    pub purpose: Option<String>,
    pub variables_schema: Option<String>,
    pub default_values: Option<String>,
    pub is_active: Option<bool>, // Will default to true
    pub metadata: Option<String>,
}

/// Template document update structure
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = template_documents)]
pub struct UpdateTemplateDocument {
    pub document_content: Option<String>,
    pub document_type: Option<Option<String>>,
    pub document_level: Option<Option<String>>,
    pub purpose: Option<Option<String>>,
    pub variables_schema: Option<Option<String>>,
    pub default_values: Option<Option<String>>,
    pub updated_at: Option<String>,
    pub is_active: Option<bool>,
    pub metadata: Option<Option<String>>,
}

impl TemplateDocument {
    /// Parse variables schema as JSON
    pub fn parse_variables_schema(&self) -> Option<JsonValue> {
        self.variables_schema
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
    }

    /// Parse default values as JSON
    pub fn parse_default_values(&self) -> Option<JsonValue> {
        self.default_values
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
    }

    /// Parse metadata as JSON
    pub fn parse_metadata(&self) -> Option<JsonValue> {
        self.metadata
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
    }

    /// Create a Tera context from default values
    pub fn create_context(&self) -> tera::Context {
        let mut context = tera::Context::new();

        // Add default values if available
        if let Some(defaults) = self.parse_default_values() {
            if let Some(obj) = defaults.as_object() {
                for (key, value) in obj {
                    context.insert(key, value);
                }
            }
        }

        context
    }
}

/// Document level for categorizing templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentLevel {
    Campaign,
    Module,
    Session,
    Handout,
}

impl DocumentLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Campaign => "campaign",
            Self::Module => "module",
            Self::Session => "session",
            Self::Handout => "handout",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "campaign" => Some(Self::Campaign),
            "module" => Some(Self::Module),
            "session" => Some(Self::Session),
            "handout" => Some(Self::Handout),
            _ => None,
        }
    }
}

/// Template types based on our template library
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateType {
    // Campaign Level Templates
    CampaignBible,
    CampaignPitch,
    StartingScenario,
    QuickStartKit,

    // Module Templates
    ModuleOverview,
    ModuleDungeon,
    ModuleHeist,
    ModuleHorror,
    ModuleMystery,
    ModulePolitical,

    // Character & NPC Templates
    CharacterIntegration,
    MajorNpcTracker,
    QuickNpcReference,
    PcArcTracker,

    // World Building Templates
    WorldOverview,
    RegionOverview,
    FactionTemplate,

    // Session Management Templates
    SessionOutline,
    ClueTracker,
    DocumentTracker,
}

impl TemplateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Campaign Level
            Self::CampaignBible => "campaign_bible",
            Self::CampaignPitch => "campaign_pitch",
            Self::StartingScenario => "starting_scenario",
            Self::QuickStartKit => "quick_start_kit",

            // Module Types
            Self::ModuleOverview => "module_overview",
            Self::ModuleDungeon => "module_dungeon",
            Self::ModuleHeist => "module_heist",
            Self::ModuleHorror => "module_horror",
            Self::ModuleMystery => "module_mystery",
            Self::ModulePolitical => "module_political",

            // Character & NPC
            Self::CharacterIntegration => "character_integration",
            Self::MajorNpcTracker => "major_npc_tracker",
            Self::QuickNpcReference => "quick_npc_reference",
            Self::PcArcTracker => "pc_arc_tracker",

            // World Building
            Self::WorldOverview => "world_overview",
            Self::RegionOverview => "region_overview",
            Self::FactionTemplate => "faction_template",

            // Session Management
            Self::SessionOutline => "session_outline",
            Self::ClueTracker => "clue_tracker",
            Self::DocumentTracker => "document_tracker",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // Campaign Level
            "campaign_bible" => Some(Self::CampaignBible),
            "campaign_pitch" => Some(Self::CampaignPitch),
            "starting_scenario" => Some(Self::StartingScenario),
            "quick_start_kit" => Some(Self::QuickStartKit),

            // Module Types
            "module_overview" => Some(Self::ModuleOverview),
            "module_dungeon" => Some(Self::ModuleDungeon),
            "module_heist" => Some(Self::ModuleHeist),
            "module_horror" => Some(Self::ModuleHorror),
            "module_mystery" => Some(Self::ModuleMystery),
            "module_political" => Some(Self::ModulePolitical),

            // Character & NPC
            "character_integration" => Some(Self::CharacterIntegration),
            "major_npc_tracker" => Some(Self::MajorNpcTracker),
            "quick_npc_reference" => Some(Self::QuickNpcReference),
            "pc_arc_tracker" => Some(Self::PcArcTracker),

            // World Building
            "world_overview" => Some(Self::WorldOverview),
            "region_overview" => Some(Self::RegionOverview),
            "faction_template" => Some(Self::FactionTemplate),

            // Session Management
            "session_outline" => Some(Self::SessionOutline),
            "clue_tracker" => Some(Self::ClueTracker),
            "document_tracker" => Some(Self::DocumentTracker),

            _ => None,
        }
    }

    /// Get the file name for this template type
    pub fn file_name(&self) -> &'static str {
        match self {
            // Convert enum variant to kebab-case filename
            Self::CampaignBible => "campaign-bible.md",
            Self::CampaignPitch => "campaign-pitch.md",
            Self::StartingScenario => "starting-scenario.md",
            Self::QuickStartKit => "quick-start-kit.md",

            Self::ModuleOverview => "module-overview.md",
            Self::ModuleDungeon => "module-dungeon.md",
            Self::ModuleHeist => "module-heist.md",
            Self::ModuleHorror => "module-horror.md",
            Self::ModuleMystery => "module-mystery.md",
            Self::ModulePolitical => "module-political.md",

            Self::CharacterIntegration => "character-integration.md",
            Self::MajorNpcTracker => "major-npc-tracker.md",
            Self::QuickNpcReference => "quick-npc-reference.md",
            Self::PcArcTracker => "pc-arc-tracker.md",

            Self::WorldOverview => "world-overview.md",
            Self::RegionOverview => "region-overview.md",
            Self::FactionTemplate => "faction-template.md",

            Self::SessionOutline => "session-outline.md",
            Self::ClueTracker => "clue-tracker.md",
            Self::DocumentTracker => "document-tracker.md",
        }
    }

    /// Get all template types
    pub fn all() -> Vec<Self> {
        vec![
            // Campaign Level
            Self::CampaignBible,
            Self::CampaignPitch,
            Self::StartingScenario,
            Self::QuickStartKit,
            // Module Types
            Self::ModuleOverview,
            Self::ModuleDungeon,
            Self::ModuleHeist,
            Self::ModuleHorror,
            Self::ModuleMystery,
            Self::ModulePolitical,
            // Character & NPC
            Self::CharacterIntegration,
            Self::MajorNpcTracker,
            Self::QuickNpcReference,
            Self::PcArcTracker,
            // World Building
            Self::WorldOverview,
            Self::RegionOverview,
            Self::FactionTemplate,
            // Session Management
            Self::SessionOutline,
            Self::ClueTracker,
            Self::DocumentTracker,
        ]
    }

    /// Get the document level for this template type
    pub fn document_level(&self) -> DocumentLevel {
        match self {
            // Campaign Level Templates
            Self::CampaignBible
            | Self::CampaignPitch
            | Self::StartingScenario
            | Self::QuickStartKit
            | Self::WorldOverview
            | Self::RegionOverview
            | Self::FactionTemplate => DocumentLevel::Campaign,

            // Module Templates
            Self::ModuleOverview
            | Self::ModuleDungeon
            | Self::ModuleHeist
            | Self::ModuleHorror
            | Self::ModuleMystery
            | Self::ModulePolitical => DocumentLevel::Module,

            // Module Templates (session prep)
            Self::SessionOutline | Self::ClueTracker => DocumentLevel::Module,

            // Handout Templates
            Self::CharacterIntegration
            | Self::MajorNpcTracker
            | Self::QuickNpcReference
            | Self::PcArcTracker
            | Self::DocumentTracker => DocumentLevel::Handout,
        }
    }
}
