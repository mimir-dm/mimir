//! Campaign management commands.
//!
//! Provides Tauri commands for creating, managing, and archiving campaigns.
//! Campaigns are the top-level organizational unit containing modules, sessions,
//! characters, and documents.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::{
    domain::{BoardCompletionStatus, TemplateInfo},
    models::campaign::campaigns::Campaign as DbCampaign,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tauri::State;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: i32,
    pub name: String,
    pub status: String,
    pub directory_path: String,
    pub created_at: String,
}

impl From<DbCampaign> for Campaign {
    fn from(db_campaign: DbCampaign) -> Self {
        Self {
            id: db_campaign.id,
            name: db_campaign.name,
            status: db_campaign.status,
            directory_path: db_campaign.directory_path,
            created_at: db_campaign.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub description: Option<String>,
    pub directory_location: String, // Base directory where campaign folder will be created
}

/// List all active (non-archived) campaigns.
///
/// # Parameters
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing a vector of `Campaign` objects.
///
/// # Errors
/// Returns an error response if database operations fail.
#[tauri::command]
pub async fn list_campaigns(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Campaign>>, ApiError> {
    info!("Listing campaigns");

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.list_active_campaigns() {
        Ok(campaigns) => {
            let campaigns: Vec<Campaign> = campaigns.into_iter().map(Campaign::from).collect();
            info!("Found {} campaigns", campaigns.len());
            debug!("Campaign details: {:?}", campaigns);
            let response = ApiResponse::success(campaigns);
            debug!("Returning success response");
            Ok(response)
        }
        Err(e) => {
            error!("Failed to list campaigns: {}", e);
            let response = ApiResponse::error(format!("Failed to list campaigns: {}", e));
            debug!("Returning error response: {:?}", response);
            Ok(response)
        }
    }
}

/// Create a new campaign.
///
/// Creates the campaign record and directory structure.
///
/// # Parameters
/// - `request` - Campaign name, description, and base directory location
///
/// # Returns
/// `ApiResponse` containing the created `Campaign`.
#[tauri::command]
pub async fn create_campaign(
    request: CreateCampaignRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Campaign>, ApiError> {
    info!(
        "Creating new campaign: {} at location: {}",
        request.name, request.directory_location
    );

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.create_campaign(
        &request.name,
        request.description,
        &request.directory_location,
    ) {
        Ok(campaign) => {
            info!(
                "Created campaign: {} with directory: {}",
                campaign.name, campaign.directory_path
            );
            Ok(ApiResponse::success(Campaign::from(campaign)))
        }
        Err(e) => {
            error!("Failed to create campaign '{}': {}", request.name, e);
            Ok(ApiResponse::error(format!(
                "Failed to create campaign: {}",
                e
            )))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateDocumentRequest {
    pub campaign_id: i32,
    pub template_id: String,
    pub variables: HashMap<String, JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedDocument {
    pub file_path: String,
    pub template_id: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Generate a campaign document from a template
#[tauri::command]
pub async fn generate_campaign_document(
    request: GenerateDocumentRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<GeneratedDocument>, ApiError> {
    info!(
        "Generating document from template '{}' for campaign {}",
        request.template_id, request.campaign_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::TemplateService::new(&mut conn);

    match service.generate_document(request.campaign_id, &request.template_id, request.variables) {
        Ok(file_path) => {
            info!("Generated document at: {}", file_path);
            Ok(ApiResponse::success(GeneratedDocument {
                file_path: file_path.clone(),
                template_id: request.template_id,
                success: true,
                error_message: None,
            }))
        }
        Err(e) => {
            error!("Failed to generate document: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to generate document: {}",
                e
            )))
        }
    }
}

/// List all available templates
#[tauri::command]
pub async fn list_templates(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<TemplateInfo>>, ApiError> {
    info!("Listing available templates");

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::TemplateService::new(&mut conn);

    match service.list_templates_with_details() {
        Ok(template_infos) => {
            info!("Found {} templates", template_infos.len());
            Ok(ApiResponse::success(template_infos))
        }
        Err(e) => {
            error!("Failed to list templates: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list templates: {}",
                e
            )))
        }
    }
}

/// Get a campaign by ID.
///
/// # Parameters
/// - `id` - The database ID of the campaign
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing the `Campaign` if found.
///
/// # Errors
/// Returns an error response if the campaign is not found or database operations fail.
#[tauri::command]
pub async fn get_campaign(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Campaign>, ApiError> {
    info!("Getting campaign with id: {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.get_campaign(id) {
        Ok(Some(campaign)) => {
            info!("Found campaign: {}", campaign.name);
            Ok(ApiResponse::success(Campaign::from(campaign)))
        }
        Ok(None) => {
            error!("Campaign {} not found", id);
            Ok(ApiResponse::error(format!(
                "Campaign with id {} not found",
                id
            )))
        }
        Err(e) => {
            error!("Failed to find campaign {}: {}", id, e);
            Ok(ApiResponse::error(format!("Database error: {}", e)))
        }
    }
}

/// Check campaign stage completion status
#[tauri::command]
pub async fn check_campaign_stage_completion(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<BoardCompletionStatus>, ApiError> {
    info!("Checking stage completion for campaign {}", campaign_id);

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.check_stage_completion(campaign_id) {
        Ok(status) => {
            info!("Stage completion status: {:?}", status);
            Ok(ApiResponse::success(status))
        }
        Err(e) => {
            error!("Failed to check stage completion: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to check stage completion: {}",
                e
            )))
        }
    }
}

/// Transition campaign to the next stage
#[tauri::command]
pub async fn transition_campaign_stage(
    campaign_id: i32,
    new_stage: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Campaign>, ApiError> {
    info!(
        "Transitioning campaign {} to stage {}",
        campaign_id, new_stage
    );

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.transition_campaign_stage(campaign_id, &new_stage) {
        Ok(updated_campaign) => {
            info!("Successfully transitioned campaign to {}", new_stage);
            Ok(ApiResponse::success(Campaign::from(updated_campaign)))
        }
        Err(e) => {
            error!("Failed to transition campaign: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to transition campaign: {}",
                e
            )))
        }
    }
}

/// Archive a campaign
#[tauri::command]
pub async fn archive_campaign(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Campaign>, ApiError> {
    info!("Archiving campaign {}", campaign_id);

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.archive_campaign(campaign_id) {
        Ok(archived_campaign) => {
            info!("Successfully archived campaign {}", campaign_id);
            Ok(ApiResponse::success(Campaign::from(archived_campaign)))
        }
        Err(e) => {
            error!("Failed to archive campaign: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to archive campaign: {}",
                e
            )))
        }
    }
}

/// Unarchive a campaign
#[tauri::command]
pub async fn unarchive_campaign(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Campaign>, ApiError> {
    info!("Unarchiving campaign {}", campaign_id);

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.unarchive_campaign(campaign_id) {
        Ok(unarchived_campaign) => {
            info!("Successfully unarchived campaign {}", campaign_id);
            Ok(ApiResponse::success(Campaign::from(unarchived_campaign)))
        }
        Err(e) => {
            error!("Failed to unarchive campaign: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to unarchive campaign: {}",
                e
            )))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteCampaignRequest {
    pub campaign_id: i32,
    pub delete_files: bool,
}

/// Delete a campaign (hard delete - only allowed for archived campaigns)
#[tauri::command]
pub async fn delete_campaign(
    request: DeleteCampaignRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!(
        "Deleting campaign {} (delete_files: {})",
        request.campaign_id, request.delete_files
    );

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.delete_campaign(request.campaign_id, request.delete_files) {
        Ok(()) => {
            info!("Successfully deleted campaign {}", request.campaign_id);
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to delete campaign: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to delete campaign: {}",
                e
            )))
        }
    }
}

/// List archived campaigns
#[tauri::command]
pub async fn list_archived_campaigns(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Campaign>>, ApiError> {
    info!("Listing archived campaigns");

    let mut conn = state.db.get_connection()?;
    let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);

    match service.list_archived_campaigns() {
        Ok(campaigns) => {
            let campaigns: Vec<Campaign> = campaigns.into_iter().map(Campaign::from).collect();
            info!("Found {} archived campaigns", campaigns.len());
            Ok(ApiResponse::success(campaigns))
        }
        Err(e) => {
            error!("Failed to list archived campaigns: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list archived campaigns: {}",
                e
            )))
        }
    }
}

/// Response for campaign summary operations
#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignSummaryResponse {
    /// The summary text (null if no summary exists)
    pub summary: Option<String>,
    /// ISO8601 timestamp when summary was last generated (null if no summary)
    pub last_updated: Option<String>,
}

/// Get the cached campaign summary and its timestamp
///
/// Returns the current cached summary if one exists, along with when it was generated.
/// Returns null values if no summary has been generated yet.
#[tauri::command]
pub async fn get_campaign_summary(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<CampaignSummaryResponse>, ApiError> {
    debug!("Getting campaign summary for campaign {}", campaign_id);

    // Get campaign directory
    let campaign_dir = {
        let mut conn = state.db.get_connection()?;
        let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);
        match service.get_campaign(campaign_id)? {
            Some(campaign) => campaign.directory_path,
            None => {
                return Ok(ApiResponse::error(format!(
                    "Campaign {} not found",
                    campaign_id
                )));
            }
        }
    };

    // Get cached summary
    let mut conn = state.db.get_connection()?;
    let service = mimir_dm_core::services::CampaignSummaryService::new(&mut conn);

    match service.get_cached_summary(&campaign_dir) {
        Some(summary) => {
            debug!("Found cached summary for campaign {}", campaign_id);
            Ok(ApiResponse::success(CampaignSummaryResponse {
                summary: Some(summary.summary),
                last_updated: Some(summary.generated_at),
            }))
        }
        None => {
            debug!("No cached summary for campaign {}", campaign_id);
            Ok(ApiResponse::success(CampaignSummaryResponse {
                summary: None,
                last_updated: None,
            }))
        }
    }
}

/// Refresh (regenerate) the campaign summary using LLM
///
/// Gathers all session notes and module information, then uses the configured
/// LLM provider to generate a new story summary. The result is cached.
#[tauri::command]
pub async fn refresh_campaign_summary(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<CampaignSummaryResponse>, ApiError> {
    use mimir_dm_core::services::{
        format_source_for_llm, CampaignSummary, CampaignSummaryService,
    };
    use mimir_dm_llm::{LlmProvider, Message};

    info!("Refreshing campaign summary for campaign {}", campaign_id);

    // Get campaign directory
    let campaign_dir = {
        let mut conn = state.db.get_connection()?;
        let mut service = mimir_dm_core::services::CampaignService::new(&mut conn);
        match service.get_campaign(campaign_id)? {
            Some(campaign) => campaign.directory_path,
            None => {
                return Ok(ApiResponse::error(format!(
                    "Campaign {} not found",
                    campaign_id
                )));
            }
        }
    };

    // Gather source materials
    let source = {
        let mut conn = state.db.get_connection()?;
        let mut service = CampaignSummaryService::new(&mut conn);
        match service.gather_source_materials(campaign_id, &campaign_dir) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to gather source materials: {}", e);
                return Ok(ApiResponse::error(format!(
                    "Failed to gather source materials: {}",
                    e
                )));
            }
        }
    };

    // Check if we have content to summarize
    if source.session_notes.is_empty() && source.modules.is_empty() {
        return Ok(ApiResponse::error(
            "No session notes or modules found to summarize".to_string(),
        ));
    }

    // Get LLM provider
    let llm_guard = state.llm.lock().await;
    let llm_service = match llm_guard.as_ref() {
        Some(llm) => llm,
        None => {
            return Ok(ApiResponse::error(
                "LLM service not initialized. Please configure a provider in settings.".to_string(),
            ));
        }
    };

    // Generate summary using LLM
    let prompt = format_source_for_llm(&source);
    let messages = vec![Message {
        role: "user".to_string(),
        content: prompt,
        tool_call_id: None,
    }];

    info!(
        "Calling LLM to generate summary ({} notes, {} modules)",
        source.session_notes.len(),
        source.modules.len()
    );

    let provider = llm_service.provider();
    let response = match provider
        .chat(messages, None, None, None, None, None, None, None)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("LLM call failed: {}", e);
            return Ok(ApiResponse::error(format!(
                "Failed to generate summary: {}",
                e
            )));
        }
    };

    let summary_text = response.content;
    let generated_at = chrono::Utc::now().to_rfc3339();

    // Save to cache
    let source_hash = CampaignSummaryService::calculate_source_hash(&source);
    let summary = CampaignSummary {
        summary: summary_text.clone(),
        generated_at: generated_at.clone(),
        source_hash,
        campaign_id,
    };

    {
        let mut conn = state.db.get_connection()?;
        let service = CampaignSummaryService::new(&mut conn);
        if let Err(e) = service.save_summary(&campaign_dir, &summary) {
            error!("Failed to cache summary: {}", e);
            // Continue anyway - we have the summary even if caching failed
        }
    }

    info!("Campaign summary refreshed successfully");

    Ok(ApiResponse::success(CampaignSummaryResponse {
        summary: Some(summary_text),
        last_updated: Some(generated_at),
    }))
}
