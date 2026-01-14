//! Common repository traits for the Data Access Layer
//!
//! This module provides trait abstractions for repository operations, enabling:
//! - Unit testing with mock implementations
//! - Decoupling services from concrete database types
//! - Clear interface definitions for data access
//!
//! # Usage Pattern
//!
//! Services can be made generic over repository traits for testability:
//!
//! ```ignore
//! // Production: use real repository
//! let service = CampaignService::new(&mut conn);
//!
//! // Tests: use mock repository
//! let mock = MockCampaignRepository::new();
//! let service = CampaignService::with_repo(mock);
//! ```

use crate::error::Result;
use crate::models::campaign::campaigns::{Campaign, NewCampaign, UpdateCampaign};
use crate::models::campaign::documents::{Document, NewDocument, UpdateDocument};
use crate::models::campaign::modules::{Module, NewModule, UpdateModule};

// =============================================================================
// Campaign Repository Trait
// =============================================================================

/// Trait for campaign data access operations.
///
/// Implement this trait to provide campaign persistence. The default implementation
/// is [`CampaignRepository`](super::campaign::campaigns::CampaignRepository) which uses SQLite.
pub trait CampaignRepositoryTrait {
    /// Create a new campaign
    fn create(&mut self, new_campaign: NewCampaign) -> Result<Campaign>;

    /// Find a campaign by ID
    fn find_by_id(&mut self, id: i32) -> Result<Option<Campaign>>;

    /// Update a campaign
    fn update(&mut self, id: i32, update: UpdateCampaign) -> Result<Campaign>;

    /// Transition a campaign to a new status
    fn transition_status(&mut self, id: i32, new_status: &str) -> Result<Campaign>;

    /// Delete a campaign
    fn delete(&mut self, id: i32) -> Result<()>;

    /// List all campaigns
    fn list(&mut self) -> Result<Vec<Campaign>>;

    /// List campaigns by status
    fn list_by_status(&mut self, status: &str) -> Result<Vec<Campaign>>;

    /// List active (non-archived) campaigns
    fn list_active(&mut self) -> Result<Vec<Campaign>>;

    /// List archived campaigns
    fn list_archived(&mut self) -> Result<Vec<Campaign>>;

    /// Archive a campaign
    fn archive(&mut self, id: i32) -> Result<Campaign>;

    /// Unarchive a campaign
    fn unarchive(&mut self, id: i32) -> Result<Campaign>;
}

// =============================================================================
// Module Repository Trait
// =============================================================================

/// Trait for module data access operations.
///
/// Implement this trait to provide module persistence. The default implementation
/// is [`ModuleRepository`](super::campaign::modules::ModuleRepository) which uses SQLite.
pub trait ModuleRepositoryTrait {
    /// Create a new module
    fn create(&mut self, new_module: NewModule) -> Result<Module>;

    /// Find a module by ID
    fn find_by_id(&mut self, id: i32) -> Result<Option<Module>>;

    /// Update a module
    fn update(&mut self, id: i32, update: UpdateModule) -> Result<Module>;

    /// Transition a module to a new status
    fn transition_status(&mut self, id: i32, new_status: &str) -> Result<Module>;

    /// Increment session count for a module
    fn increment_sessions(&mut self, id: i32) -> Result<Module>;

    /// Delete a module
    fn delete(&mut self, id: i32) -> Result<()>;

    /// List all modules for a campaign
    fn list_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Module>>;

    /// List modules by status for a campaign
    fn list_by_campaign_and_status(&mut self, campaign_id: i32, status: &str)
        -> Result<Vec<Module>>;

    /// Find modules that should trigger next module planning
    fn find_modules_needing_next(&mut self, campaign_id: i32) -> Result<Vec<Module>>;

    /// Get the next module number for a campaign
    fn get_next_module_number(&mut self, campaign_id: i32) -> Result<i32>;
}

// =============================================================================
// Document Repository Trait
// =============================================================================

/// Trait for document data access operations.
///
/// Implement this trait to provide document persistence. The default implementation
/// is [`DocumentRepository`](super::campaign::documents::DocumentRepository) which uses SQLite.
pub trait DocumentRepositoryTrait {
    /// Create a new document
    fn create(&mut self, new_document: NewDocument) -> Result<Document>;

    /// Find a document by ID
    fn find_by_id(&mut self, document_id: i32) -> Result<Document>;

    /// Get all documents for a campaign
    fn find_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>>;

    /// Get all documents for a module
    fn find_by_module(&mut self, module_id: i32) -> Result<Vec<Document>>;

    /// Find a document by module and template
    fn find_by_module_and_template(
        &mut self,
        module_id: i32,
        template_id: &str,
    ) -> Result<Option<Document>>;

    /// Get all documents for a session
    fn find_by_session(&mut self, session_id: i32) -> Result<Vec<Document>>;

    /// Get all documents by template ID
    fn find_by_template(&mut self, template_id: &str) -> Result<Vec<Document>>;

    /// Get all incomplete documents for a campaign
    fn find_incomplete_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>>;

    /// Get all completed documents for a campaign
    fn find_completed_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>>;

    /// Update a document
    fn update(&mut self, document_id: i32, update: UpdateDocument) -> Result<Document>;

    /// Mark a document as completed
    fn mark_completed(&mut self, document_id: i32) -> Result<Document>;

    /// Delete a document
    fn delete(&mut self, document_id: i32) -> Result<usize>;

    /// Check if a document exists by file path
    fn exists_by_path(&mut self, file_path: &str) -> Result<bool>;

    /// Get all handout documents for a campaign
    fn find_handouts_by_campaign(&mut self, campaign_id: i32) -> Result<Vec<Document>>;
}

// =============================================================================
// Generic Repository Traits (for future use)
// =============================================================================

/// Generic synchronous repository trait for basic CRUD operations.
///
/// This trait is provided for entities that follow a standard CRUD pattern.
/// For domain-specific operations, use the dedicated traits above.
pub trait Repository<T> {
    /// Create a new entity
    fn create(&mut self, entity: T) -> Result<T>;

    /// Find an entity by ID
    fn find_by_id(&mut self, id: &str) -> Result<Option<T>>;

    /// Update an entity
    fn update(&mut self, id: &str, entity: T) -> Result<T>;

    /// Delete an entity
    fn delete(&mut self, id: &str) -> Result<()>;

    /// List all entities
    fn list(&mut self) -> Result<Vec<T>>;
}

/// Trait for batch operations
pub trait BatchOperations<T> {
    /// Insert multiple entities in a single transaction
    fn batch_insert(&mut self, entities: Vec<T>) -> Result<usize>;

    /// Delete multiple entities by IDs
    fn batch_delete(&mut self, ids: Vec<String>) -> Result<usize>;
}

/// Trait for entities that belong to a rule system
pub trait RuleSystemScoped {
    /// Find all entities for a specific rule system
    fn find_by_rule_system(&mut self, rule_system_id: &str) -> Result<Vec<Self>>
    where
        Self: Sized;
}

/// Trait for entities that have a source
pub trait SourceScoped {
    /// Find all entities from a specific source
    fn find_by_source(&mut self, source_id: &str) -> Result<Vec<Self>>
    where
        Self: Sized;
}
