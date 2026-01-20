//! Module management service.
//!
//! Provides business logic for managing adventure modules within campaigns.
//! Modules represent discrete story arcs or adventures with their own workflow
//! stages, documents, and session tracking.

use crate::connection::DbConnection;
use crate::dal::campaign::documents::DocumentRepository;
use crate::dal::campaign::modules::ModuleRepository;
use crate::domain::{BoardCompletionStatus, BoardRegistry};
use crate::error::Result;
use crate::models::campaign::{
    documents::{Document, NewDocument},
    modules::{Module, NewModule, UpdateModule},
};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// Service for managing modules
pub struct ModuleService<'a> {
    conn: &'a mut DbConnection,
}

impl<'a> ModuleService<'a> {
    /// Create a new module service.
    ///
    /// # Arguments
    /// * `conn` - Mutable reference to the database connection
    pub fn new(conn: &'a mut DbConnection) -> Self {
        Self { conn }
    }

    /// Create a new module for a campaign.
    ///
    /// Creates the module record with auto-assigned module number. Does not
    /// create any documents or directories - use `create_module_with_documents`
    /// for that.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the parent campaign
    /// * `name` - Module name/title
    /// * `expected_sessions` - Estimated number of sessions to complete
    ///
    /// # Returns
    /// * `Ok(Module)` - The created module record
    pub fn create_module(
        &mut self,
        campaign_id: i32,
        name: String,
        expected_sessions: i32,
    ) -> Result<Module> {
        let mut repo = ModuleRepository::new(self.conn);

        // Get the next module number
        let module_number = repo.get_next_module_number(campaign_id)?;

        let new_module = NewModule {
            campaign_id,
            name,
            module_number,
            status: "planning".to_string(),
            expected_sessions,
        };

        repo.create(new_module)
    }

    /// Create a new module with its directory and initial documents.
    ///
    /// Creates the module record, sets up the module directory on disk, and
    /// generates the initial module overview document from a template. The
    /// template used depends on the module_type.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the parent campaign
    /// * `name` - Module name/title
    /// * `expected_sessions` - Estimated number of sessions to complete
    /// * `module_type` - Optional type ("mystery", "dungeon", "heist", etc.)
    ///
    /// # Returns
    /// * `Ok(Module)` - The created module record
    /// * `Err` - If campaign not found or file operations fail
    pub fn create_module_with_documents(
        &mut self,
        campaign_id: i32,
        name: String,
        expected_sessions: i32,
        module_type: Option<String>,
    ) -> Result<Module> {
        // Get the campaign to find directory path
        use crate::dal::campaign::campaigns::CampaignRepository;
        let mut campaign_repo = CampaignRepository::new(self.conn);
        let campaign = campaign_repo
            .find_by_id(campaign_id)?
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        // Create the module record
        let module = self.create_module(campaign_id, name.clone(), expected_sessions)?;

        // Create the module directory
        let module_dir = PathBuf::from(&campaign.directory_path)
            .join("modules")
            .join(format!("module_{:02}", module.module_number));

        if !module_dir.exists() {
            fs::create_dir_all(&module_dir)?;
        }

        // Prepare variables for templates
        let mut variables = std::collections::HashMap::new();
        variables.insert("module_name".to_string(), json!(name));
        variables.insert("module_number".to_string(), json!(module.module_number));

        // Determine which template to use based on module type
        let template_id = if let Some(ref mt) = module_type {
            match mt.as_str() {
                "mystery" => "module_mystery",
                "dungeon" => "module_dungeon",
                "heist" => "module_heist",
                "horror" => "module_horror",
                "political" => "module_political",
                _ => "module_overview", // Default to generic overview
            }
        } else {
            "module_overview" // Default if no type specified
        };

        let overview_file_path = module_dir.join("module-overview.md");

        // Get the template and use its create_context method for defaults
        use crate::dal::campaign::template_documents::TemplateRepository;
        let template = TemplateRepository::get_latest(self.conn, template_id)?;

        // Create context with template defaults
        let mut context = template.create_context();

        // Add our custom variables (these will override defaults if they have the same key)
        for (key, value) in variables {
            context.insert(&key, &value);
        }

        // Render the template
        let mut tera = tera::Tera::default();
        tera.add_raw_template(&template.document_id, &template.document_content)
            .map_err(|e| {
                diesel::result::Error::QueryBuilderError(
                    format!("Failed to add template: {}", e).into(),
                )
            })?;

        let template_content = tera.render(&template.document_id, &context).map_err(|e| {
            diesel::result::Error::QueryBuilderError(
                format!("Failed to render template: {}", e).into(),
            )
        })?;

        // Add YAML frontmatter with title and type
        let title = format!("{} - Module Overview", name);
        let content_with_frontmatter = format!(
            "---\ntitle: \"{}\"\ntype: module_overview\n---\n\n{}",
            title, template_content
        );

        // Write the file with frontmatter
        fs::write(&overview_file_path, content_with_frontmatter)?;

        // Create database record for overview
        let overview_doc = NewDocument {
            campaign_id,
            module_id: Some(module.id),
            session_id: None,
            template_id: "module_overview".to_string(), // Always use module_overview as the template_id
            document_type: "module_overview".to_string(),
            title,
            file_path: overview_file_path.to_string_lossy().to_string(),
            file_type: "markdown".to_string(),
            is_user_created: false,
        };

        DocumentRepository::create(self.conn, overview_doc)?;

        Ok(module)
    }

    /// Get a module by ID.
    ///
    /// # Arguments
    /// * `id` - Database ID of the module
    ///
    /// # Returns
    /// * `Ok(Some(Module))` - If found
    /// * `Ok(None)` - If no module exists with that ID
    pub fn get_module(&mut self, id: i32) -> Result<Option<Module>> {
        let mut repo = ModuleRepository::new(self.conn);
        repo.find_by_id(id)
    }

    /// List all modules for a campaign.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Vec<Module>)` - All modules in the campaign, ordered by module_number
    pub fn list_campaign_modules(&mut self, campaign_id: i32) -> Result<Vec<Module>> {
        let mut repo = ModuleRepository::new(self.conn);
        repo.list_by_campaign(campaign_id)
    }

    /// List modules by status for a campaign.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    /// * `status` - Module status to filter by (e.g., "planning", "active")
    ///
    /// # Returns
    /// * `Ok(Vec<Module>)` - Modules matching the status
    pub fn list_modules_by_status(
        &mut self,
        campaign_id: i32,
        status: &str,
    ) -> Result<Vec<Module>> {
        let mut repo = ModuleRepository::new(self.conn);
        repo.list_by_campaign_and_status(campaign_id, status)
    }

    /// Update module details.
    ///
    /// # Arguments
    /// * `id` - Database ID of the module
    /// * `update` - Fields to update (None fields are left unchanged)
    ///
    /// # Returns
    /// * `Ok(Module)` - The updated module record
    pub fn update_module(&mut self, id: i32, update: UpdateModule) -> Result<Module> {
        let mut repo = ModuleRepository::new(self.conn);
        repo.update(id, update)
    }

    /// Transition a module to a new stage.
    ///
    /// Validates that the transition is allowed per the module board definition
    /// before updating the status.
    ///
    /// # Arguments
    /// * `id` - Database ID of the module
    /// * `new_stage` - Target stage (e.g., "preparation", "running", "completed")
    ///
    /// # Returns
    /// * `Ok(Module)` - The updated module with new status
    /// * `Err` - If transition is not allowed or module not found
    pub fn transition_module_stage(&mut self, id: i32, new_stage: &str) -> Result<Module> {
        let mut repo = ModuleRepository::new(self.conn);

        // Validate transition with board definition
        let board_registry = BoardRegistry::new();
        let board = board_registry
            .get("module")
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        let module = repo
            .find_by_id(id)?
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        if !board.can_transition(&module.status, new_stage) {
            return Err(diesel::result::Error::QueryBuilderError(
                format!("Cannot transition from {} to {}", module.status, new_stage).into(),
            )
            .into());
        }

        repo.transition_status(id, new_stage)
    }

    /// Initialize documents for a module stage.
    ///
    /// Creates the required documents for the module's current stage from
    /// templates. Skips documents that already exist.
    ///
    /// # Arguments
    /// * `module_id` - Database ID of the module
    /// * `campaign_directory` - Path to the campaign's root directory
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of created document file names
    /// * `Err` - If module not found or file operations fail
    pub fn initialize_module_documents(
        &mut self,
        module_id: i32,
        campaign_directory: &str,
    ) -> Result<Vec<String>> {
        let mut module_repo = ModuleRepository::new(self.conn);
        let module = module_repo
            .find_by_id(module_id)?
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        // Get board configuration
        let board_registry = BoardRegistry::new();
        let board = board_registry
            .get("module")
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        // Get required documents for current stage
        let required_docs = board.required_documents(&module.status);

        // Create module directory in modules folder if it doesn't exist
        let module_dir = PathBuf::from(campaign_directory)
            .join("modules")
            .join(format!("module_{:02}", module.module_number));

        if !module_dir.exists() {
            fs::create_dir_all(&module_dir)?;
        }

        let mut created_files = Vec::new();

        // Initialize required documents using templates
        for doc_template_id in required_docs {
            let file_name = format!("{}.md", doc_template_id.replace('_', "-"));
            let file_path = module_dir.join(&file_name);

            // Check if document already exists in database
            let existing = DocumentRepository::find_by_module_and_template(
                self.conn,
                module_id,
                doc_template_id,
            )?;

            if existing.is_none() && !file_path.exists() {
                // Get the template and use its create_context method for defaults
                use crate::dal::campaign::template_documents::TemplateRepository;
                let template = TemplateRepository::get_latest(self.conn, doc_template_id)?;

                // Create context with template defaults
                let mut context = template.create_context();

                // Add module-specific variables
                context.insert("module_name", &json!(module.name));
                context.insert("module_number", &json!(module.module_number));

                // Render the template
                let mut tera = tera::Tera::default();
                tera.add_raw_template(&template.document_id, &template.document_content)
                    .map_err(|e| {
                        diesel::result::Error::QueryBuilderError(
                            format!("Failed to add template: {}", e).into(),
                        )
                    })?;

                let content = tera.render(&template.document_id, &context).map_err(|e| {
                    diesel::result::Error::QueryBuilderError(
                        format!("Failed to render template: {}", e).into(),
                    )
                })?;

                // Write the file
                fs::write(&file_path, content)?;

                // Create database record
                let new_doc = NewDocument {
                    campaign_id: module.campaign_id,
                    module_id: Some(module_id),
                    session_id: None,
                    template_id: doc_template_id.to_string(),
                    document_type: doc_template_id.to_string(),
                    title: doc_template_id
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
                    file_path: file_path.to_string_lossy().to_string(),
                    file_type: "markdown".to_string(),
                    is_user_created: false,
                };

                DocumentRepository::create(self.conn, new_doc)?;
                created_files.push(file_name);
            }
        }

        Ok(created_files)
    }

    /// Get module documents.
    ///
    /// # Arguments
    /// * `module_id` - Database ID of the module
    ///
    /// # Returns
    /// * `Ok(Vec<Document>)` - All documents associated with the module
    pub fn get_module_documents(&mut self, module_id: i32) -> Result<Vec<Document>> {
        DocumentRepository::list_by_module(self.conn, module_id)
    }

    /// Check module completion status.
    ///
    /// Evaluates the current stage's required and optional documents to determine
    /// if the stage is complete and the module can progress.
    ///
    /// # Arguments
    /// * `module_id` - Database ID of the module
    ///
    /// # Returns
    /// * `Ok(BoardCompletionStatus)` - Completion metrics
    /// * `Err` - If module not found
    pub fn check_module_completion(&mut self, module_id: i32) -> Result<BoardCompletionStatus> {
        let mut module_repo = ModuleRepository::new(self.conn);
        let module = module_repo
            .find_by_id(module_id)?
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        let documents = DocumentRepository::list_by_module(self.conn, module_id)?;

        // Get board configuration
        let board_registry = BoardRegistry::new();
        let board = board_registry
            .get("module")
            .ok_or_else(|| diesel::result::Error::NotFound)?;

        let required_docs = board.required_documents(&module.status);
        let optional_docs = board.optional_documents(&module.status);
        let no_completion_docs = board.no_completion_required_documents(&module.status);

        // Filter required docs to exclude those that don't need completion
        let completion_required_docs: Vec<&str> = required_docs
            .iter()
            .filter(|doc| !no_completion_docs.contains(doc))
            .copied()
            .collect();

        // Count completed documents (only for those that require completion)
        let completed_required = documents
            .iter()
            .filter(|d| {
                completion_required_docs.contains(&d.template_id.as_str())
                    && d.completed_at.is_some()
            })
            .count();

        let completed_optional = documents
            .iter()
            .filter(|d| optional_docs.contains(&d.template_id.as_str()) && d.completed_at.is_some())
            .count();

        // Find missing required documents (but exclude no-completion docs from the check)
        let mut missing_required = Vec::new();
        for req_doc in &completion_required_docs {
            let doc = documents.iter().find(|d| d.template_id == *req_doc);
            match doc {
                None => missing_required.push(req_doc.to_string()),
                Some(d) if d.completed_at.is_none() => missing_required.push(req_doc.to_string()),
                _ => {}
            }
        }

        let is_stage_complete = missing_required.is_empty();

        let next_stage = board.next_stage(&module.status).map(|s| s.to_string());
        let can_progress = is_stage_complete && next_stage.is_some();

        Ok(BoardCompletionStatus {
            board_type: "module".to_string(),
            current_stage: module.status.clone(),
            total_required_documents: completion_required_docs.len(),
            completed_required_documents: completed_required,
            total_optional_documents: optional_docs.len(),
            completed_optional_documents: completed_optional,
            missing_required_documents: missing_required,
            is_stage_complete,
            can_progress,
            next_stage,
            stage_metadata: board.stage_metadata(&module.status),
        })
    }

    /// Check if any modules need next module planning (60% complete).
    ///
    /// Returns modules that are nearing completion so the DM can start
    /// planning the next adventure.
    ///
    /// # Arguments
    /// * `campaign_id` - Database ID of the campaign
    ///
    /// # Returns
    /// * `Ok(Vec<Module>)` - Modules needing attention
    pub fn find_modules_needing_next(&mut self, campaign_id: i32) -> Result<Vec<Module>> {
        let mut repo = ModuleRepository::new(self.conn);
        repo.find_modules_needing_next(campaign_id)
    }

    /// Increment session count for a module.
    ///
    /// Call after each session to track actual vs expected sessions.
    ///
    /// # Arguments
    /// * `module_id` - Database ID of the module
    ///
    /// # Returns
    /// * `Ok(Module)` - The updated module with incremented actual_sessions
    pub fn increment_module_sessions(&mut self, module_id: i32) -> Result<Module> {
        let mut repo = ModuleRepository::new(self.conn);
        repo.increment_sessions(module_id)
    }

    /// Delete a module.
    ///
    /// Permanently removes the module record. Does not delete associated
    /// documents or files on disk.
    ///
    /// # Arguments
    /// * `id` - Database ID of the module to delete
    ///
    /// # Returns
    /// * `Ok(())` - If deletion succeeds
    pub fn delete_module(&mut self, id: i32) -> Result<()> {
        let mut repo = ModuleRepository::new(self.conn);
        repo.delete(id)
    }
}
