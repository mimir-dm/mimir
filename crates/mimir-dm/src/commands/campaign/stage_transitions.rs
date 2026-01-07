//! Stage transition and document initialization commands.
//!
//! Provides Tauri commands for initializing stage-specific documents and
//! managing campaign workflow transitions.

use crate::state::AppState;
use crate::types::ApiResponse;
use anyhow::Result;
use mimir_dm_core::{
    dal::campaign::{
        campaigns::CampaignRepository, documents::DocumentRepository,
        template_documents::TemplateRepository,
    },
    models::campaign::{
        campaigns::Campaign, documents::NewDocument, template_documents::TemplateDocument,
    },
    DbConnection,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use tera::Tera;
use tracing::warn;

/// Stage document templates mapping
fn get_stage_documents(stage: &str) -> Vec<(&'static str, &'static str)> {
    match stage {
        "concept" => vec![
            ("campaign_pitch", "Campaign Pitch"),
            // The other concept documents (sparks, big three, first adventure)
            // are part of the creative process but not formal templates
        ],
        "session_zero" => vec![
            ("starting_scenario", "Starting Scenario"),
            ("world_primer", "World Primer"),
            ("character_guidelines", "Character Guidelines"),
            ("table_expectations", "Table Expectations"),
            ("character_integration", "Character Integration Forms"),
        ],
        "integration" => vec![
            ("campaign_bible", "Campaign Bible"),
            ("major_npc_tracker", "Major NPCs"),
        ],
        _ => vec![],
    }
}

/// Create documents for a campaign stage
pub fn create_stage_documents(
    conn: &mut DbConnection,
    campaign: &Campaign,
    stage: &str,
) -> Result<Vec<String>, anyhow::Error> {
    let documents = get_stage_documents(stage);
    let mut created = Vec::new();

    for (template_id, title) in documents {
        // Check if document already exists
        let existing = DocumentRepository::find_by_campaign(conn, campaign.id)?;
        let exists = existing.iter().any(|d| d.template_id == template_id);

        if !exists {
            // Get the template content
            let template = match TemplateRepository::get_latest(conn, template_id) {
                Ok(t) => t,
                Err(_) => {
                    warn!(template_id = %template_id, "Template not found, skipping document creation");
                    continue;
                }
            };

            // Create the document file
            let file_name = format!("{}.md", template_id);
            let file_path = PathBuf::from(&campaign.directory_path).join(&file_name);

            // Process template with default values only (no overrides for now)
            let content = match process_template_content(&template, None) {
                Ok(c) => c,
                Err(e) => {
                    warn!(template_id = %template_id, error = %e, "Failed to render template");
                    continue;
                }
            };

            // Write file to disk
            if let Err(e) = fs::write(&file_path, content) {
                warn!(path = %file_path.display(), error = %e, "Failed to write document file");
                continue;
            }

            // Create database record
            let new_doc = NewDocument {
                campaign_id: campaign.id,
                module_id: None,
                session_id: None,
                template_id: template_id.to_string(),
                document_type: template_id.replace('-', "_"),
                title: title.to_string(),
                file_path: file_path.to_string_lossy().to_string(),
                file_type: "markdown".to_string(),
                is_user_created: false,
            };

            DocumentRepository::create(conn, new_doc)?;
            created.push(title.to_string());
        }
    }

    Ok(created)
}

/// Process template content using Tera with default values and optional overrides
fn process_template_content(
    template: &TemplateDocument,
    provided_values: Option<&HashMap<String, JsonValue>>,
) -> Result<String> {
    // Step 1: Create context from template defaults
    let mut context = template.create_context();

    // Step 2: Override defaults with any provided values
    if let Some(values) = provided_values {
        for (key, value) in values {
            context.insert(key, value);
        }
    }

    // Step 3: Render using Tera
    let mut tera = Tera::default();
    tera.add_raw_template(&template.document_id, &template.document_content)?;

    let rendered = tera.render(&template.document_id, &context)?;
    Ok(rendered)
}

/// Initialize documents for current campaign stage.
///
/// Creates the required documents for the campaign's current workflow stage.
/// Documents are created from templates based on the stage (concept, session_zero,
/// integration, etc.).
///
/// # Parameters
/// - `campaign_id` - The database ID of the campaign
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing a vector of created document titles.
///
/// # Errors
/// Returns an error string if the campaign is not found or document creation fails.
#[tauri::command]
pub async fn initialize_stage_documents(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<String>>, String> {
    let mut conn = state.db.get_connection().map_err(|e| e.to_string())?;

    // Get the campaign
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = match campaign_repo.find_by_id(campaign_id) {
        Ok(Some(c)) => c,
        Ok(None) => return Ok(ApiResponse::error("Campaign not found".to_string())),
        Err(e) => return Ok(ApiResponse::error(format!("Database error: {}", e))),
    };

    // Create documents for the current stage
    let created_docs = create_stage_documents(&mut conn, &campaign, &campaign.status)
        .map_err(|e| format!("Failed to create stage documents: {}", e))?;

    Ok(ApiResponse::success(created_docs))
}
