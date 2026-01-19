//! MCP tool definitions
//!
//! This module contains all tool definitions for the Mimir MCP server.

pub mod campaign;
pub mod catalog;
pub mod character;
pub mod document;

// Re-export commonly used types
pub use campaign::{
    AddItemToModuleInput, AddModuleItemResponse, AddMonsterResponse, AddMonsterToModuleInput,
    CampaignDetailsResponse, CampaignListItem, CreateModuleInput, CreateModuleResponse,
    GetCampaignDetailsInput, GetModuleDetailsInput, ListCampaignsInput, ListModulesInput,
    ModuleDetailsResponse, ModuleListItem, ModuleSummary, SetActiveCampaignInput,
    SetActiveCampaignResponse, StageCompletionInfo, UpdateModuleMonsterInput, UpdateMonsterResponse,
};

pub use character::{
    AddCharacterItemResponse, AddItemToCharacterInput, AssignNpcResponse, AssignNpcToModuleInput,
    CharacterListItem, CharacterVersionSummary, CreateCharacterInput, CreateCharacterResponse,
    EditCharacterInput, EditCharacterResponse, GetCharacterInput, GetCharacterResponse,
    ListCharactersInput, UpdateCharacterCurrencyInput, UpdateCurrencyResponse,
};

pub use catalog::{
    ItemSearchResult, MonsterSearchResult, SearchItemsInput, SearchMonstersInput,
    SearchTrapsInput, TrapSearchResult,
};

pub use document::{
    CreateUserDocumentInput, DocumentListItem, EditDocumentInput, EditDocumentResponse,
    ListDocumentsInput, ReadDocumentInput, ReadDocumentResponse,
};
