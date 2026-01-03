//! Campaign Management Models
//!
//! Models for organizing and running campaigns and modules.
//! These represent the story management layer, not game mechanics.

pub mod campaigns;
pub mod documents;
pub mod fog;
pub mod light_sources;
pub mod maps;
pub mod module_frontmatter;
pub mod module_monsters;
pub mod modules;
pub mod template_documents;
pub mod template_frontmatter;
pub mod tokens;
pub mod workflow_cards;

// Re-export commonly used types
pub use campaigns::{Campaign, NewCampaign};
pub use documents::{Document, NewDocument};
pub use fog::{FogRevealedArea, FogRevealedAreaSummary, NewFogRevealedArea};
pub use light_sources::{LightSource, LightSourceSummary, LightType, NewLightSource, UpdateLightSource};
pub use maps::{AmbientLight, GridType, Map, MapSummary, NewMap, UpdateMap};
pub use module_frontmatter::{
    ItemReference, ModuleFrontmatter, ModuleVariable, MonsterReference, NpcReference,
};
pub use module_monsters::{
    EncounterGroup, ModuleMonster, ModuleMonsterWithData, NewModuleMonster, UpdateModuleMonster,
};
pub use modules::{Module, NewModule};
pub use template_documents::TemplateDocument;
pub use template_frontmatter::TemplateFrontmatter;
pub use tokens::{NewToken, Token, TokenSize, TokenSummary, TokenType, UpdateToken, VisionType};
pub use workflow_cards::{NewWorkflowCard, WorkflowCard};
