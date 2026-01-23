//! Service Layer
//!
//! Business logic services that sit between consumers (MCP, Tauri) and the DAL layer.
//! Services encapsulate validation, transactions, and orchestration of database operations.

mod asset;
mod campaign;
pub mod catalog;
mod character;
mod document;
mod map;
mod module;

use thiserror::Error;

pub use asset::{AssetService, UploadAssetInput};
pub use campaign::{CampaignService, CreateCampaignInput, UpdateCampaignInput};
pub use character::{
    AddInventoryInput, CharacterService, CreateCharacterInput, UpdateCharacterInput,
};
pub use document::{CreateDocumentInput, DocumentService, UpdateDocumentInput};
pub use map::{CreateMapInput, MapService, UpdateMapInput};
pub use module::{CreateModuleInput, ModuleService, ModuleType, UpdateModuleInput};
pub use catalog::{
    ActionService, BackgroundService, CatalogEntityService, CatalogTableService,
    ClassFeatureService, ClassService, ConditionService, CultService, DeityService, FeatService,
    HazardService, ItemService, LanguageService, MonsterService, ObjectService,
    OptionalFeatureService, PsionicService, RaceService, RewardService, SpellService,
    SubclassFeatureService, SubclassService, TrapService, VariantRuleService, VehicleService,
};

/// Default query limit to prevent memory issues on large result sets.
pub const DEFAULT_QUERY_LIMIT: i64 = 1000;

/// Service layer error type.
///
/// All services return `ServiceResult<T>` which uses this error type.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Entity not found.
    #[error("Not found: {entity_type} with id {id}")]
    NotFound {
        /// The type of entity (e.g., "Campaign", "Monster").
        entity_type: String,
        /// The ID that was searched for.
        id: String,
    },

    /// Validation error (invalid input, business rule violation).
    #[error("Validation error: {0}")]
    Validation(String),

    /// Database error from Diesel.
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    /// IO error (file operations).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for service operations.
pub type ServiceResult<T> = Result<T, ServiceError>;

impl ServiceError {
    /// Create a NotFound error.
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Create a Validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = ServiceError::not_found("Campaign", "123");
        assert_eq!(err.to_string(), "Not found: Campaign with id 123");
    }

    #[test]
    fn test_validation_error() {
        let err = ServiceError::validation("Name cannot be empty");
        assert_eq!(err.to_string(), "Validation error: Name cannot be empty");
    }

    #[test]
    fn test_database_error_conversion() {
        let diesel_err = diesel::result::Error::NotFound;
        let err: ServiceError = diesel_err.into();
        assert!(matches!(err, ServiceError::Database(_)));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ServiceError = io_err.into();
        assert!(matches!(err, ServiceError::Io(_)));
    }
}
