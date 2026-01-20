//! Campaign service for business logic operations
//!
//! This service handles all campaign-related business logic including:
//! - Campaign creation with directory structure
//! - Stage transitions with validation
//! - Document generation

use crate::{
    connection::DbConnection,
    dal::campaign::campaigns::CampaignRepository,
    dal::campaign::documents::DocumentRepository,
    dal::campaign::template_documents::TemplateRepository,
    domain::{BoardCompletionStatus, BoardRegistry},
    error::{DbError, Result},
    models::campaign::campaigns::{Campaign, NewCampaign},
    models::campaign::documents::NewDocument,
};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Convert a string to kebab-case for use as a folder name.
/// Examples: "The Frost Architect" -> "the-frost-architect"
///           "My Campaign Name" -> "my-campaign-name"
fn to_kebab_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '_' {
                '-'
            } else {
                // Skip other special characters
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        // Collapse multiple dashes into one
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Service for campaign-related business logic operations.
pub struct CampaignService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> CampaignService<'a> {
    /// Create a new campaign service.
    ///
    /// # Arguments
    /// * `conn` - Mutable reference to the database connection
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new campaign with directory structure.
    ///
    /// Creates a campaign record in the database, sets up the directory structure
    /// on disk (including subdirectories for sessions, modules, characters, etc.),
    /// and generates initial concept stage documents.
    ///
    /// # Arguments
    /// * `name` - Campaign name (cannot be empty or whitespace-only)
    /// * `_description` - Optional campaign description (currently unused)
    /// * `directory_location` - Base path where the campaign folder will be created
    ///
    /// # Returns
    /// * `Ok(Campaign)` - The created campaign record with assigned ID
    /// * `Err(DbError::InvalidData)` - If the name is empty or directory already exists
    /// * `Err(DbError::Io)` - If directory creation fails
    pub fn create_campaign(
        &mut self,
        name: &str,
        _description: Option<String>,
        directory_location: &str,
    ) -> Result<Campaign> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err(DbError::InvalidData(
                "Campaign name cannot be empty".to_string(),
            ));
        }

        // Create directory structure
        let base_path = Path::new(directory_location);
        let campaign_path = self.create_campaign_directory_structure(base_path, name)?;

        // Create database record
        let mut repo = CampaignRepository::new(self.conn);
        let new_campaign = NewCampaign {
            name: name.to_string(),
            status: "concept".to_string(),
            directory_path: campaign_path.to_string_lossy().to_string(),
        };

        // Try to create the campaign
        let campaign = match repo.create(new_campaign) {
            Ok(c) => c,
            Err(e) => {
                // Rollback: try to remove the created directory
                if let Err(remove_err) = fs::remove_dir_all(&campaign_path) {
                    warn!(path = %campaign_path.display(), error = %remove_err, "Failed to cleanup campaign directory after database error");
                }
                return Err(e);
            }
        };

        // Create initial documents for the concept stage
        if let Err(e) = self.create_initial_documents(&campaign) {
            warn!(campaign_id = campaign.id, error = %e, "Failed to create initial documents");
            // Continue anyway - campaign is created, documents can be created later
        }

        Ok(campaign)
    }

    /// Transition a campaign to a new stage.
    ///
    /// Validates that the transition is allowed per the campaign board definition,
    /// updates the campaign status, and creates any required documents for the
    /// new stage.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign to transition
    /// * `new_stage` - Target stage name (e.g., "session_zero", "integration")
    ///
    /// # Returns
    /// * `Ok(Campaign)` - The updated campaign with new status
    /// * `Err(DbError::NotFound)` - If the campaign doesn't exist
    /// * `Err(DbError::InvalidData)` - If the transition is not allowed
    pub fn transition_campaign_stage(
        &mut self,
        campaign_id: i32,
        new_stage: &str,
    ) -> Result<Campaign> {
        // Get the campaign
        let mut repo = CampaignRepository::new(self.conn);
        let campaign = repo
            .find_by_id(campaign_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            })?;

        // Validate transition using board definition
        let board_registry = BoardRegistry::new();
        let board = board_registry.get("campaign").ok_or_else(|| {
            DbError::InvalidData("Campaign board definition not found".to_string())
        })?;

        // Check if transition is allowed
        if !board.can_transition(&campaign.status, new_stage) {
            return Err(DbError::InvalidData(format!(
                "Cannot transition from {} to {}",
                campaign.status, new_stage
            )));
        }

        // Perform the transition
        let updated_campaign = repo.transition_status(campaign_id, new_stage)?;

        // Create stage-specific documents
        if let Err(e) = self.create_stage_documents(&updated_campaign, new_stage) {
            warn!(campaign_id = campaign_id, stage = new_stage, error = %e, "Failed to create stage documents");
            // Continue anyway - transition succeeded
        }

        Ok(updated_campaign)
    }

    /// List all campaigns.
    ///
    /// # Returns
    /// * `Ok(Vec<Campaign>)` - All campaigns (active and archived)
    pub fn list_campaigns(&mut self) -> Result<Vec<Campaign>> {
        let mut repo = CampaignRepository::new(self.conn);
        repo.list()
    }

    /// Get a campaign by ID.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Some(Campaign))` - If found
    /// * `Ok(None)` - If no campaign exists with that ID
    pub fn get_campaign(&mut self, campaign_id: i32) -> Result<Option<Campaign>> {
        let mut repo = CampaignRepository::new(self.conn);
        repo.find_by_id(campaign_id)
    }

    /// Archive a campaign.
    ///
    /// Sets the archived_at timestamp to mark the campaign as archived.
    /// Archived campaigns are hidden from the active campaigns list.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign to archive
    ///
    /// # Returns
    /// * `Ok(Campaign)` - The updated campaign with archived_at set
    pub fn archive_campaign(&mut self, campaign_id: i32) -> Result<Campaign> {
        let mut repo = CampaignRepository::new(self.conn);
        repo.archive(campaign_id)
    }

    /// Unarchive a campaign.
    ///
    /// Clears the archived_at timestamp to restore the campaign to active status.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign to unarchive
    ///
    /// # Returns
    /// * `Ok(Campaign)` - The updated campaign with archived_at cleared
    pub fn unarchive_campaign(&mut self, campaign_id: i32) -> Result<Campaign> {
        let mut repo = CampaignRepository::new(self.conn);
        repo.unarchive(campaign_id)
    }

    /// Delete a campaign (hard delete).
    ///
    /// Permanently removes the campaign from the database. Optionally deletes
    /// the campaign directory on disk. Campaign must be archived first.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign to delete
    /// * `delete_files` - If true, also delete the campaign directory on disk
    ///
    /// # Returns
    /// * `Ok(())` - If deletion succeeds
    /// * `Err(DbError::NotFound)` - If the campaign doesn't exist
    /// * `Err(DbError::InvalidData)` - If the campaign is not archived
    pub fn delete_campaign(&mut self, campaign_id: i32, delete_files: bool) -> Result<()> {
        // Get campaign info for directory path
        let mut repo = CampaignRepository::new(self.conn);
        let campaign =
            repo.find_by_id(campaign_id)?
                .ok_or_else(|| crate::error::DbError::NotFound {
                    entity_type: "Campaign".to_string(),
                    id: campaign_id.to_string(),
                })?;

        // Check if campaign is archived (safety check)
        if campaign.archived_at.is_none() {
            return Err(crate::error::DbError::InvalidData(
                "Cannot delete active campaign. Archive it first.".to_string(),
            ));
        }

        // Delete from database first
        repo.delete(campaign_id)?;

        // Optionally delete campaign directory
        if delete_files {
            if let Err(e) = std::fs::remove_dir_all(&campaign.directory_path) {
                warn!(path = %campaign.directory_path, error = %e, "Failed to delete campaign directory");
                // Continue - database deletion succeeded
            }
        }

        Ok(())
    }

    /// List active campaigns (not archived).
    ///
    /// # Returns
    /// * `Ok(Vec<Campaign>)` - All campaigns that have not been archived
    pub fn list_active_campaigns(&mut self) -> Result<Vec<Campaign>> {
        let mut repo = CampaignRepository::new(self.conn);
        repo.list_active()
    }

    /// List archived campaigns.
    ///
    /// # Returns
    /// * `Ok(Vec<Campaign>)` - All campaigns that have been archived
    pub fn list_archived_campaigns(&mut self) -> Result<Vec<Campaign>> {
        let mut repo = CampaignRepository::new(self.conn);
        repo.list_archived()
    }

    /// Check campaign stage completion status.
    ///
    /// Evaluates the current stage's required and optional documents to determine
    /// if the stage is complete and the campaign can progress to the next stage.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign to check
    ///
    /// # Returns
    /// * `Ok(BoardCompletionStatus)` - Completion metrics including documents
    ///   completed, missing, and whether progression is allowed
    /// * `Err(DbError::NotFound)` - If the campaign doesn't exist
    pub fn check_stage_completion(&mut self, campaign_id: i32) -> Result<BoardCompletionStatus> {
        // Get the campaign
        let mut repo = CampaignRepository::new(self.conn);
        let campaign = repo
            .find_by_id(campaign_id)?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Campaign".to_string(),
                id: campaign_id.to_string(),
            })?;

        // Get the board definition
        let board_registry = BoardRegistry::new();
        let board = board_registry.get("campaign").ok_or_else(|| {
            DbError::InvalidData("Campaign board definition not found".to_string())
        })?;

        let current_stage = &campaign.status;

        // Get required and optional documents for current stage
        let required_docs = board.required_documents(current_stage);
        let optional_docs = board.optional_documents(current_stage);

        // Get all documents for this campaign
        let all_documents = DocumentRepository::find_by_campaign(self.conn, campaign_id)?;

        // Count completed required documents
        let mut completed_required = 0;
        let mut missing_required = Vec::new();

        for doc_type in &required_docs {
            if let Some(doc) = all_documents.iter().find(|d| d.document_type == *doc_type) {
                if doc.completed_at.is_some() {
                    completed_required += 1;
                }
            } else {
                missing_required.push(doc_type.to_string());
            }
        }

        // Count completed optional documents
        let mut completed_optional = 0;
        for doc_type in &optional_docs {
            if let Some(doc) = all_documents.iter().find(|d| d.document_type == *doc_type) {
                if doc.completed_at.is_some() {
                    completed_optional += 1;
                }
            }
        }

        let is_stage_complete =
            required_docs.len() == completed_required && missing_required.is_empty();
        let next_stage = board.next_stage(current_stage).map(|s| s.to_string());
        let can_progress = is_stage_complete && next_stage.is_some();

        Ok(BoardCompletionStatus {
            board_type: board.board_type().to_string(),
            current_stage: current_stage.clone(),
            total_required_documents: required_docs.len(),
            completed_required_documents: completed_required,
            total_optional_documents: optional_docs.len(),
            completed_optional_documents: completed_optional,
            missing_required_documents: missing_required,
            is_stage_complete,
            can_progress,
            next_stage,
            stage_metadata: board.stage_metadata(current_stage),
        })
    }

    /// Create the campaign directory structure
    fn create_campaign_directory_structure(
        &self,
        base_path: &Path,
        campaign_name: &str,
    ) -> Result<PathBuf> {
        // Convert campaign name to kebab-case for folder name
        let folder_name = to_kebab_case(campaign_name);
        let campaign_path = base_path.join(&folder_name);

        // Check if campaign directory already exists
        if campaign_path.exists() {
            return Err(DbError::InvalidData(format!(
                "Campaign directory '{}' already exists",
                campaign_path.display()
            )));
        }

        debug!(path = %campaign_path.display(), "Creating campaign directory structure");

        // Create main campaign directory
        fs::create_dir_all(&campaign_path)?;

        // Create all the required directories
        let directories = [
            "session_zero",
            "world",
            "world/factions",
            "regions",
            "modules",
            "sessions",
            "characters",
            "npcs",
            "npcs/recurring",
            "resources",
            "resources/maps",
            "resources/handouts",
            "resources/references",
        ];

        for dir in directories {
            let dir_path = campaign_path.join(dir);
            fs::create_dir_all(&dir_path)?;
            debug!(path = %dir_path.display(), "Created directory");
        }

        // Create initial README
        let readme_content = format!(
            "# {}\n\nCampaign created on {}\n\nUse the Mimir application to generate additional campaign documents as needed.",
            campaign_name,
            chrono::Local::now().format("%Y-%m-%d")
        );

        fs::write(campaign_path.join("README.md"), readme_content)?;

        debug!("Successfully created campaign directory structure");
        Ok(campaign_path)
    }

    /// Create initial documents for a new campaign
    ///
    /// Creates ALL campaign documents upfront (both required and optional from all stages).
    /// Users can delete documents they don't need.
    fn create_initial_documents(&mut self, campaign: &Campaign) -> Result<()> {
        // Get the board definition to know what documents are needed
        let board_registry = BoardRegistry::new();
        let board = board_registry.get("campaign").ok_or_else(|| {
            DbError::InvalidData("Campaign board definition not found".to_string())
        })?;

        // Collect ALL documents from ALL stages (both required and optional)
        let mut all_docs: Vec<&str> = Vec::new();
        for stage in board.stages() {
            for doc in board.required_documents(stage) {
                if !all_docs.contains(&doc) {
                    all_docs.push(doc);
                }
            }
            for doc in board.optional_documents(stage) {
                if !all_docs.contains(&doc) {
                    all_docs.push(doc);
                }
            }
        }

        for doc_type in all_docs {
            // Use doc_type directly as template_id (both use snake_case now)
            let template_id = doc_type.to_string();
            let file_path = format!("{}/{}.md", campaign.directory_path, template_id);

            // Create directory if needed
            let full_path = std::path::Path::new(&file_path);
            if let Some(parent) = full_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    warn!(document_type = %doc_type, error = %e, "Failed to create directory for document");
                    continue;
                }
            }

            // Get the template from the database - skip if not found
            let template = match TemplateRepository::get_latest(self.conn, &template_id) {
                Ok(t) => t,
                Err(e) => {
                    warn!(template_id = %template_id, error = %e, "Template not found, skipping document creation");
                    continue;
                }
            };

            // Render the template with its default context
            let context = template.create_context();
            let mut tera = tera::Tera::default();
            if let Err(e) = tera.add_raw_template(&template.document_id, &template.document_content)
            {
                warn!(document_type = %doc_type, error = %e, "Failed to parse template");
                continue;
            }

            let rendered_content = match tera.render(&template.document_id, &context) {
                Ok(c) => c,
                Err(e) => {
                    warn!(document_type = %doc_type, error = %e, "Failed to render template");
                    continue;
                }
            };

            // Generate title from doc_type (e.g., "campaign_pitch" -> "Campaign Pitch")
            let title: String = doc_type
                .replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            // Add YAML frontmatter with title and type
            let content_with_frontmatter = format!(
                "---\ntitle: \"{}\"\ntype: {}\n---\n\n{}",
                title, doc_type, rendered_content
            );

            // Write the rendered template content to the file with frontmatter
            if let Err(e) = fs::write(full_path, &content_with_frontmatter) {
                warn!(document_type = %doc_type, error = %e, "Failed to write document file");
                continue;
            }

            let new_doc = NewDocument {
                campaign_id: campaign.id,
                module_id: None,
                session_id: None,
                template_id,
                document_type: doc_type.to_string(),
                title,
                file_path,
                file_type: "markdown".to_string(),
                is_user_created: false,
            };

            if let Err(e) = DocumentRepository::create(self.conn, new_doc) {
                warn!(document_type = %doc_type, error = %e, "Failed to create document record");
            }
        }

        Ok(())
    }

    /// Create stage-specific documents when transitioning
    fn create_stage_documents(&mut self, campaign: &Campaign, stage: &str) -> Result<()> {
        // Get the board definition
        let board_registry = BoardRegistry::new();
        let board = board_registry.get("campaign").ok_or_else(|| {
            DbError::InvalidData("Campaign board definition not found".to_string())
        })?;

        let required_docs = board.required_documents(stage);

        // Check which documents already exist
        let existing_docs = DocumentRepository::find_by_campaign(self.conn, campaign.id)?;
        let existing_template_ids: Vec<String> = existing_docs
            .iter()
            .map(|d| d.template_id.clone())
            .collect();

        // Create missing required documents
        for doc_type in required_docs {
            // Use doc_type directly as template_id (both use snake_case now)
            let template_id = doc_type.to_string();

            // Skip if document already exists
            if existing_template_ids.contains(&template_id) {
                continue;
            }

            let file_path = format!("{}/{}.md", campaign.directory_path, template_id);

            // Create directory if needed
            let full_path = std::path::Path::new(&file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Generate title from doc_type (e.g., "campaign_pitch" -> "Campaign Pitch")
            let title: String = doc_type
                .replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            // Try to get the template from the database and render it
            let rendered_content = match TemplateRepository::get_latest(self.conn, &template_id) {
                Ok(template) => {
                    // Render the template with its default context
                    let context = template.create_context();
                    let mut tera = tera::Tera::default();
                    tera.add_raw_template(&template.document_id, &template.document_content)
                        .map_err(|e| {
                            DbError::InvalidData(format!("Failed to add template: {}", e))
                        })?;

                    tera.render(&template.document_id, &context).map_err(|e| {
                        DbError::InvalidData(format!("Failed to render template: {}", e))
                    })?
                }
                Err(_) => {
                    // If template doesn't exist, create a basic markdown file
                    format!("# {}\n\n*This document will be created for the {} stage.*\n\n## Overview\n\n[Document content will be added here]\n",
                        title,
                        stage
                    )
                }
            };

            // Add YAML frontmatter with title and type
            let content_with_frontmatter = format!(
                "---\ntitle: \"{}\"\ntype: {}\n---\n\n{}",
                title, doc_type, rendered_content
            );

            // Write the content to the file with frontmatter
            fs::write(full_path, content_with_frontmatter)?;

            // Create database record
            let new_doc = NewDocument {
                campaign_id: campaign.id,
                module_id: None,
                session_id: None,
                template_id: template_id.clone(),
                document_type: doc_type.to_string(),
                title: doc_type
                    .replace('_', " ")
                    .split_whitespace()
                    .map(|w| {
                        let mut chars = w.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
                file_path,
                file_type: "markdown".to_string(),
                is_user_created: false,
            };

            if let Err(e) = DocumentRepository::create(self.conn, new_doc) {
                warn!(document_type = %doc_type, error = %e, "Failed to create document");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::establish_connection;
    use tempfile::TempDir;

    #[test]
    fn test_create_campaign_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let service = CampaignService::new(&mut conn);

        let campaign_path = service
            .create_campaign_directory_structure(temp_dir.path(), "Test Campaign")
            .unwrap();

        // Verify directories were created
        assert!(campaign_path.exists());
        assert!(campaign_path.join("session_zero").exists());
        assert!(campaign_path.join("world/factions").exists());
        assert!(campaign_path.join("README.md").exists());
    }

    #[test]
    fn test_create_campaign_with_empty_name() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        let result = service.create_campaign("", None, "/tmp");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidData(_)));
    }

    #[test]
    fn test_create_campaign_with_whitespace_name() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        let result = service.create_campaign("   ", None, "/tmp");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidData(_)));
    }

    #[test]
    fn test_create_campaign_success() {
        let temp_dir = TempDir::new().unwrap();
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        let campaign = service
            .create_campaign(
                "My Test Campaign",
                Some("A test campaign description".to_string()),
                temp_dir.path().to_str().unwrap(),
            )
            .unwrap();

        assert_eq!(campaign.name, "My Test Campaign");
        assert_eq!(campaign.status, "concept");
        assert!(campaign.directory_path.contains("my-test-campaign"));

        // Verify directory was created
        let campaign_dir = Path::new(&campaign.directory_path);
        assert!(campaign_dir.exists());
        assert!(campaign_dir.join("session_zero").exists());
    }

    #[test]
    fn test_create_duplicate_campaign_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        // Create first campaign
        let campaign1 = service
            .create_campaign("Duplicate Test", None, temp_dir.path().to_str().unwrap())
            .unwrap();

        assert_eq!(campaign1.name, "Duplicate Test");

        // Try to create second campaign with same name in same location
        let result =
            service.create_campaign("Duplicate Test", None, temp_dir.path().to_str().unwrap());

        assert!(result.is_err());
        // Should fail because directory already exists
    }

    #[test]
    fn test_list_campaigns_empty() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        let campaigns = service.list_campaigns().unwrap();
        assert_eq!(campaigns.len(), 0);
    }

    #[test]
    fn test_list_campaigns_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        // Create multiple campaigns
        service
            .create_campaign("Campaign 1", None, temp_dir.path().to_str().unwrap())
            .unwrap();

        // Create subdirectory for second campaign
        let subdir = temp_dir.path().join("other");
        fs::create_dir_all(&subdir).unwrap();

        service
            .create_campaign("Campaign 2", None, subdir.to_str().unwrap())
            .unwrap();

        let campaigns = service.list_campaigns().unwrap();
        assert_eq!(campaigns.len(), 2);

        let names: Vec<String> = campaigns.iter().map(|c| c.name.clone()).collect();
        assert!(names.contains(&"Campaign 1".to_string()));
        assert!(names.contains(&"Campaign 2".to_string()));
    }

    #[test]
    fn test_get_campaign() {
        let temp_dir = TempDir::new().unwrap();
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        let created = service
            .create_campaign(
                "Get Test Campaign",
                Some("Description".to_string()),
                temp_dir.path().to_str().unwrap(),
            )
            .unwrap();

        // Get existing campaign
        let found = service.get_campaign(created.id).unwrap();
        assert!(found.is_some());

        let campaign = found.unwrap();
        assert_eq!(campaign.id, created.id);
        assert_eq!(campaign.name, "Get Test Campaign");

        // Get non-existent campaign
        let not_found = service.get_campaign(99999).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_transition_campaign_stage() {
        let temp_dir = TempDir::new().unwrap();
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        let campaign = service
            .create_campaign("Transition Test", None, temp_dir.path().to_str().unwrap())
            .unwrap();

        assert_eq!(campaign.status, "concept");

        // Transition to session_zero
        let updated = service
            .transition_campaign_stage(campaign.id, "session_zero")
            .unwrap();

        assert_eq!(updated.status, "session_zero");

        // Verify we can't transition to invalid stage
        let invalid_result = service.transition_campaign_stage(
            campaign.id,
            "completed", // Can't jump from session_zero to completed
        );

        assert!(invalid_result.is_err());
        assert!(matches!(
            invalid_result.unwrap_err(),
            DbError::InvalidData(_)
        ));
    }

    #[test]
    fn test_transition_nonexistent_campaign() {
        let mut conn = establish_connection(":memory:").unwrap();
        crate::run_migrations(&mut conn).unwrap();

        let mut service = CampaignService::new(&mut conn);

        let result = service.transition_campaign_stage(99999, "session_zero");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::NotFound { .. }));
    }
}
