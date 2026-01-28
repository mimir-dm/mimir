//! Campaign Models
//!
//! Models for campaign management including campaigns, modules, sources, assets, documents, characters, maps, module entities, and map overlays.

mod campaign;
mod campaign_asset;
mod campaign_source;
mod character;
mod character_class;
mod character_feat;
mod character_feature;
mod character_inventory;
mod character_proficiency;
mod character_source;
mod character_spell;
mod document;
mod fog;
mod light_source;
mod map;
mod map_poi;
mod map_trap;
mod module;
mod module_monster;
mod module_npc;
mod token_placement;

pub use campaign::{Campaign, NewCampaign, UpdateCampaign};
pub use campaign_asset::{
    extension_for_mime_type, is_allowed_mime_type, CampaignAsset, NewCampaignAsset,
    ALLOWED_MIME_TYPES,
};
pub use campaign_source::{CampaignSource, NewCampaignSource};
pub use character::{Character, CharacterResponse, NewCharacter, UpdateCharacter};
pub use character_class::{CharacterClass, NewCharacterClass, UpdateCharacterClass};
pub use character_feat::{CharacterFeat, FeatSourceType, NewCharacterFeat};
pub use character_feature::{CharacterFeature, FeatureType, NewCharacterFeature};
pub use character_inventory::{CharacterInventory, NewCharacterInventory, UpdateCharacterInventory};
pub use character_proficiency::{
    CharacterProficiency, NewCharacterProficiency, ProficiencyType, UpdateCharacterProficiency,
};
pub use character_source::{CharacterSource, NewCharacterSource};
pub use character_spell::{CharacterSpell, NewCharacterSpell, UpdateCharacterSpell};
pub use document::{Document, NewDocument, UpdateDocument};
pub use fog::{FogRevealedArea, FogState, NewFogRevealedArea};
pub use light_source::{presets as light_presets, LightSource, NewLightSource, UpdateLightSource};
pub use map::{LightingMode, Map, NewMap, UpdateMap};
pub use map_trap::{MapTrap, NewMapTrap, UpdateMapTrap};
pub use map_poi::{MapPoi, NewMapPoi, UpdateMapPoi};
pub use module::{Module, NewModule, UpdateModule};
pub use module_monster::{ModuleMonster, NewModuleMonster, UpdateModuleMonster};
pub use module_npc::{ModuleNpc, NewModuleNpc, UpdateModuleNpc};
pub use token_placement::{NewTokenPlacement, TokenPlacement, UpdateTokenPlacement};
