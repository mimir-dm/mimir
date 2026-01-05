//! Service layer for business logic
//!
//! This module contains services that orchestrate business logic,
//! combining DAL operations with domain rules.

/// Default query limit for catalog searches to prevent memory issues
pub const DEFAULT_QUERY_LIMIT: i64 = 1000;

pub mod action_service;
pub mod background_service;
pub mod campaign_archive_service;
pub mod campaign_service;
pub mod campaign_summary_service;
pub mod catalog_trait;
pub mod character;
pub mod class_service;
pub mod condition_service;
pub mod cult_service;
pub mod deity_service;
pub mod document_service;
pub mod feat_service;
pub mod fog_service;
pub mod item_service;
pub mod light_source_service;
pub mod language_service;
pub mod map_service;
pub mod module_frontmatter_service;
pub mod module_item_service;
pub mod module_monster_service;
pub mod module_npc_service;
pub mod module_service;
pub mod monster_renderer;
pub mod monster_service;
pub mod object_service;
pub mod optional_feature_service;
pub mod player_service;
pub mod psionic_service;
pub mod race_service;
pub mod reward_service;
pub mod spell_service;
pub mod table_service;
pub mod template_service;
pub mod token_service;
pub mod trap_service;
pub mod variant_rule_service;
pub mod vehicle_service;
pub mod reference_service;

// Re-export services
pub use action_service::ActionService;
pub use background_service::BackgroundService;
pub use campaign_service::CampaignService;
pub use campaign_summary_service::{
    CampaignSummary, CampaignSummaryService, ModuleSummaryInfo, SessionNoteInfo,
    SummarySourceMaterial, format_source_for_llm,
};
pub use catalog_trait::CatalogService;
pub use character::{
    CharacterProgressionService, CharacterService, CharacterSpellService,
};
pub use class_service::ClassService;
pub use condition_service::ConditionService;
pub use cult_service::CultService;
pub use deity_service::DeityService;
pub use document_service::DocumentService;
pub use feat_service::FeatService;
pub use fog_service::FogOfWarService;
pub use item_service::ItemService;
pub use light_source_service::LightSourceService;
pub use language_service::LanguageService;
pub use map_service::MapService;
pub use module_frontmatter_service::{ModuleFrontmatterService, SyncResult};
pub use module_item_service::ModuleItemService;
pub use module_monster_service::ModuleMonsterService;
pub use module_npc_service::ModuleNpcService;
pub use module_service::ModuleService;
pub use monster_service::MonsterService;
pub use object_service::ObjectService;
pub use optional_feature_service::OptionalFeatureService;
pub use player_service::PlayerService;
pub use psionic_service::PsionicService;
pub use race_service::RaceService;
pub use reward_service::RewardService;
pub use spell_service::{SpellService, SpellServiceStateful};
pub use table_service::TableService;
pub use template_service::TemplateService;
pub use token_service::TokenService;
pub use trap_service::TrapService;
pub use variant_rule_service::VariantRuleService;
pub use vehicle_service::VehicleService;
pub use reference_service::{ReferenceData, ReferenceService};
pub use campaign_archive_service::{
    ArchiveManifest, ArchivePreview, CampaignArchiveService, CatalogReference, ARCHIVE_EXTENSION,
};
