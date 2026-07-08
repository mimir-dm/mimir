//! Campaign Commands
//!
//! Tauri commands for campaign CRUD operations.

use mimir_core::models::campaign::{Campaign, CampaignSource};
use mimir_core::services::{
    CampaignService, CreateCampaignInput, SourceService, UpdateCampaignInput,
};
use tauri::State;

use crate::state::AppState;
use super::{to_api_response, ApiResponse};

/// List all campaigns.
///
/// # Arguments
/// * `include_archived` - If true, includes archived campaigns in the list.
#[tauri::command]
pub fn list_campaigns(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> ApiResponse<Vec<Campaign>> {
    let include_archived = include_archived.unwrap_or(false);

    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CampaignService::new(&mut db).list(include_archived);
    to_api_response(result)
}

/// List only archived campaigns.
#[tauri::command]
pub fn list_archived_campaigns(state: State<'_, AppState>) -> ApiResponse<Vec<Campaign>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get all campaigns including archived, then filter to only archived
    let result = CampaignService::new(&mut db)
        .list(true)
        .map(|campaigns| campaigns.into_iter().filter(|c| c.is_archived()).collect());

    to_api_response(result)
}

/// Get a campaign by ID.
#[tauri::command]
pub fn get_campaign(state: State<'_, AppState>, id: String) -> ApiResponse<Campaign> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CampaignService::new(&mut db).get(&id);
    match result {
        Ok(Some(campaign)) => ApiResponse::ok(campaign),
        Ok(None) => ApiResponse::err(format!("Campaign not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a new campaign.
#[derive(Debug, serde::Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Create a new campaign.
#[tauri::command]
pub fn create_campaign(
    state: State<'_, AppState>,
    request: CreateCampaignRequest,
) -> ApiResponse<Campaign> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut input = CreateCampaignInput::new(&request.name);
    if let Some(desc) = request.description {
        input = input.with_description(desc);
    }

    let result = CampaignService::new(&mut db).create(input);
    to_api_response(result)
}

/// Request for updating a campaign.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

/// Update a campaign.
#[tauri::command]
pub fn update_campaign(
    state: State<'_, AppState>,
    id: String,
    request: UpdateCampaignRequest,
) -> ApiResponse<Campaign> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let input = UpdateCampaignInput {
        name: request.name,
        description: request.description,
    };

    let result = CampaignService::new(&mut db).update(&id, input);
    to_api_response(result)
}

/// Archive a campaign (soft delete).
#[tauri::command]
pub fn archive_campaign(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Campaign> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = CampaignService::new(&mut db);

    // Archive the campaign
    if let Err(e) = service.archive(&campaign_id) {
        return ApiResponse::err(e.to_string());
    }

    // Return the updated campaign
    match service.get(&campaign_id) {
        Ok(Some(campaign)) => ApiResponse::ok(campaign),
        Ok(None) => ApiResponse::err(format!("Campaign not found: {}", campaign_id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Unarchive a campaign.
#[tauri::command]
pub fn unarchive_campaign(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Campaign> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let mut service = CampaignService::new(&mut db);

    // Unarchive the campaign
    if let Err(e) = service.unarchive(&campaign_id) {
        return ApiResponse::err(e.to_string());
    }

    // Return the updated campaign
    match service.get(&campaign_id) {
        Ok(Some(campaign)) => ApiResponse::ok(campaign),
        Ok(None) => ApiResponse::err(format!("Campaign not found: {}", campaign_id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for deleting a campaign.
#[derive(Debug, serde::Deserialize)]
pub struct DeleteCampaignRequest {
    pub campaign_id: String,
    #[serde(default)]
    pub delete_files: bool,
}

/// Delete a campaign permanently.
#[tauri::command]
pub fn delete_campaign(
    state: State<'_, AppState>,
    request: DeleteCampaignRequest,
) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Note: delete_files is not implemented yet - campaigns don't have external files in v0.5
    let result = CampaignService::new(&mut db).delete(&request.campaign_id);
    to_api_response(result)
}

// =============================================================================
// Campaign Sources
// =============================================================================

/// List all source codes enabled for a campaign.
#[tauri::command]
pub fn list_campaign_sources(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(SourceService::new(&mut db).list_campaign_sources(&campaign_id))
}

/// Add a source to a campaign's allowed sources.
#[tauri::command]
pub fn add_campaign_source(
    state: State<'_, AppState>,
    campaign_id: String,
    source_code: String,
) -> ApiResponse<CampaignSource> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(SourceService::new(&mut db).add_campaign_source(&campaign_id, &source_code))
}

/// Remove a source from a campaign's allowed sources.
#[tauri::command]
pub fn remove_campaign_source(
    state: State<'_, AppState>,
    campaign_id: String,
    source_code: String,
) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(SourceService::new(&mut db).remove_campaign_source(&campaign_id, &source_code))
}

/// Set all sources for a campaign (replaces existing, atomically).
#[tauri::command]
pub fn set_campaign_sources(
    state: State<'_, AppState>,
    campaign_id: String,
    source_codes: Vec<String>,
) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(SourceService::new(&mut db).set_campaign_sources(&campaign_id, &source_codes))
}
